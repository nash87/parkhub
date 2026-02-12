//! Email Notifications via SMTP
//!
//! Optional SMTP email sending. Only active if PARKHUB_SMTP_HOST is set.

use anyhow::Result;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
}

impl SmtpConfig {
    pub fn from_env() -> Option<Self> {
        let host = std::env::var("PARKHUB_SMTP_HOST").ok()?;
        Some(Self {
            host,
            port: std::env::var("PARKHUB_SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            user: std::env::var("PARKHUB_SMTP_USER").unwrap_or_default(),
            pass: std::env::var("PARKHUB_SMTP_PASS").unwrap_or_default(),
            from: std::env::var("PARKHUB_SMTP_FROM")
                .unwrap_or_else(|_| "noreply@parkhub.local".to_string()),
        })
    }
}

#[derive(Clone)]
pub struct EmailService {
    config: Arc<SmtpConfig>,
}

impl EmailService {
    pub fn new(config: SmtpConfig) -> Self {
        info!(
            "Email service initialized (SMTP: {}:{})",
            config.host, config.port
        );
        Self {
            config: Arc::new(config),
        }
    }

    async fn send(&self, to: &str, subject: &str, body: String) -> Result<()> {
        let email = Message::builder()
            .from(self.config.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)?;

        let creds = Credentials::new(self.config.user.clone(), self.config.pass.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)?
            .port(self.config.port)
            .credentials(creds)
            .build();

        mailer.send(email).await?;
        info!("Email sent to {}: {}", to, subject);
        Ok(())
    }

    pub async fn send_booking_confirmation(
        &self,
        to: &str,
        user_name: &str,
        slot_number: &str,
        lot_name: &str,
        start_time: &str,
        end_time: &str,
    ) -> Result<()> {
        let body = format!(
            r#"<html><body><h2>Booking Confirmed</h2><p>Hello {},</p><p>Your parking spot has been booked:</p><ul><li><b>Lot:</b> {}</li><li><b>Slot:</b> {}</li><li><b>From:</b> {}</li><li><b>Until:</b> {}</li></ul><p>Don't forget to check in!</p><p>— ParkHub</p></body></html>"#,
            user_name, lot_name, slot_number, start_time, end_time
        );
        self.send(to, "ParkHub: Booking Confirmed", body).await
    }

    pub async fn send_booking_reminder(
        &self,
        to: &str,
        user_name: &str,
        slot_number: &str,
        lot_name: &str,
        start_time: &str,
    ) -> Result<()> {
        let body = format!(
            r#"<html><body><h2>Booking Reminder</h2><p>Hello {},</p><p>Your parking at <b>{}</b> slot <b>{}</b> starts at <b>{}</b>.</p><p>Remember to check in within 15 minutes.</p><p>— ParkHub</p></body></html>"#,
            user_name, lot_name, slot_number, start_time
        );
        self.send(to, "ParkHub: Booking Reminder", body).await
    }

    pub async fn send_auto_release_notification(
        &self,
        to: &str,
        user_name: &str,
        slot_number: &str,
        lot_name: &str,
    ) -> Result<()> {
        let body = format!(
            r#"<html><body><h2>Booking Auto-Released</h2><p>Hello {},</p><p>Your booking for <b>{}</b> at <b>{}</b> was auto-released (no check-in).</p><p>— ParkHub</p></body></html>"#,
            user_name, slot_number, lot_name
        );
        self.send(to, "ParkHub: Booking Auto-Released", body).await
    }

    pub async fn send_waitlist_notification(
        &self,
        to: &str,
        user_name: &str,
        lot_name: &str,
        date: &str,
    ) -> Result<()> {
        let body = format!(
            r#"<html><body><h2>Spot Available!</h2><p>Hello {},</p><p>A spot is now available at <b>{}</b> on <b>{}</b>. Book now!</p><p>— ParkHub</p></body></html>"#,
            user_name, lot_name, date
        );
        self.send(to, "ParkHub: Spot Available", body).await
    }
}
