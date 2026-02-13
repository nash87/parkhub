//! Background Jobs - auto-release, reminders, waitlist cleanup

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::email::EmailService;
use crate::AppState;

pub fn start_background_jobs(state: Arc<RwLock<AppState>>, email: Option<EmailService>) {
    let s1 = state.clone();
    let e1 = email.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = run_auto_release(&s1, &e1).await {
                warn!("Auto-release error: {}", e);
            }
        }
    });

    let s2 = state.clone();
    let e2 = email.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            if let Err(e) = run_reminders(&s2, &e2).await {
                warn!("Reminder error: {}", e);
            }
        }
    });

    let s3 = state;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_expired_waitlist(&s3).await {
                warn!("Waitlist cleanup error: {}", e);
            }
        }
    });

    info!("Background jobs started (auto-release, reminders, waitlist cleanup)");
}

async fn run_auto_release(
    state: &Arc<RwLock<AppState>>,
    email: &Option<EmailService>,
) -> anyhow::Result<()> {
    let sg = state.read().await;
    // Read from DB settings first, fall back to env var, then default 0 (disabled)
    let auto_release_minutes: i64 = match sg.db.get_setting("auto_release_minutes").await {
        Ok(Some(val)) => val.parse().unwrap_or(0),
        _ => std::env::var("PARKHUB_AUTO_RELEASE_MINUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    };
    // If 0, auto-release is disabled
    if auto_release_minutes <= 0 {
        return Ok(());
    }
    let bookings = sg.db.list_bookings().await?;
    let now = Utc::now();

    for booking in &bookings {
        if booking.status != parkhub_common::BookingStatus::Confirmed {
            continue;
        }
        if booking.checked_in_at.is_some() {
            continue;
        }
        let elapsed = now.signed_duration_since(booking.start_time).num_minutes();
        if elapsed >= auto_release_minutes {
            let mut updated = booking.clone();
            updated.status = parkhub_common::BookingStatus::AutoReleased;
            updated.updated_at = now;
            sg.db.save_booking(&updated).await?;

            if let Ok(Some(mut slot)) = sg.db.get_parking_slot(&booking.slot_id.to_string()).await {
                slot.status = parkhub_common::SlotStatus::Available;
                let _ = sg.db.save_parking_slot(&slot).await;
            }
            info!(
                "Auto-released booking {} (no check-in after {} min)",
                booking.id, auto_release_minutes
            );

            if let Some(ref es) = email {
                if let Ok(Some(user)) = sg.db.get_user(&booking.user_id.to_string()).await {
                    let _ = es
                        .send_auto_release_notification(
                            &user.email,
                            &user.name,
                            booking.slot_number.as_deref().unwrap_or("?"),
                            booking.lot_name.as_deref().unwrap_or("Lot"),
                        )
                        .await;
                }
            }

            // Notify waitlist
            let date = booking.start_time.format("%Y-%m-%d").to_string();
            let wl = sg
                .db
                .list_waitlist_by_lot(&booking.lot_id.to_string(), Some(&date))
                .await
                .unwrap_or_default();
            if let Some(first) = wl.first() {
                if !first.notified {
                    if let Some(ref es) = email {
                        if let Ok(Some(wu)) = sg.db.get_user(&first.user_id.to_string()).await {
                            let _ = es
                                .send_waitlist_notification(
                                    &wu.email,
                                    &wu.name,
                                    booking.lot_name.as_deref().unwrap_or("Lot"),
                                    &date,
                                )
                                .await;
                        }
                    }
                    let mut ue = first.clone();
                    ue.notified = true;
                    let _ = sg.db.save_waitlist_entry(&ue).await;
                }
            }
        }
    }
    Ok(())
}

async fn run_reminders(
    state: &Arc<RwLock<AppState>>,
    email: &Option<EmailService>,
) -> anyhow::Result<()> {
    let es = match email {
        Some(e) => e,
        None => return Ok(()),
    };
    let sg = state.read().await;
    let bookings = sg.db.list_bookings().await?;
    let now = Utc::now();
    for b in &bookings {
        if b.status != parkhub_common::BookingStatus::Confirmed {
            continue;
        }
        let mins = b.start_time.signed_duration_since(now).num_minutes();
        if (25..=30).contains(&mins) {
            if let Ok(Some(user)) = sg.db.get_user(&b.user_id.to_string()).await {
                let _ = es
                    .send_booking_reminder(
                        &user.email,
                        &user.name,
                        b.slot_number.as_deref().unwrap_or("?"),
                        b.lot_name.as_deref().unwrap_or("Lot"),
                        &b.start_time.to_rfc3339(),
                    )
                    .await;
            }
        }
    }
    Ok(())
}

async fn cleanup_expired_waitlist(state: &Arc<RwLock<AppState>>) -> anyhow::Result<()> {
    let sg = state.read().await;
    let entries = sg.db.list_all_waitlist().await?;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    for e in &entries {
        if e.date < today {
            let _ = sg.db.delete_waitlist_entry(&e.id.to_string()).await;
        }
    }
    Ok(())
}
