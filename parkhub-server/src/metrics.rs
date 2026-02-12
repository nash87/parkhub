//! Prometheus Metrics

use metrics::{counter, gauge};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

/// Initialize the Prometheus metrics exporter
pub fn init_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

/// Record authentication events
pub fn record_auth_event(event_type: &str, success: bool) {
    let labels = [
        ("event", event_type.to_string()),
        ("success", success.to_string()),
    ];
    counter!("auth_events_total", &labels).increment(1);
}

/// Record booking events
pub fn record_booking_event(event_type: &str) {
    let labels = [("event", event_type.to_string())];
    counter!("booking_events_total", &labels).increment(1);
}

/// Record parking lot occupancy
#[allow(dead_code)]
pub fn record_lot_occupancy(lot_id: &str, lot_name: &str, total: u64, occupied: u64) {
    let labels = [
        ("lot_id", lot_id.to_string()),
        ("lot_name", lot_name.to_string()),
    ];
    gauge!("parking_lot_total_slots", &labels).set(total as f64);
    gauge!("parking_lot_occupied_slots", &labels).set(occupied as f64);
    if total > 0 {
        gauge!("parking_lot_occupancy_percent", &labels)
            .set((occupied as f64 / total as f64) * 100.0);
    }
}
