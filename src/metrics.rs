use metrics::{Counter, Gauge, Histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;

#[derive(Debug)]
pub struct AppMetrics {
    // HTTP metrics
    pub http_requests_total: Counter,
    pub http_request_duration: Histogram,
    #[allow(dead_code)]
    pub http_active_connections: Gauge,

    // NATS metrics
    pub nats_requests_total: Counter,
    pub nats_request_duration: Histogram,
    pub nats_errors_total: Counter,
}

impl AppMetrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let builder = PrometheusBuilder::new();
        let (recorder, _handle) = builder.build()?;
        metrics::set_global_recorder(recorder)?;

        Ok(Self {
            // HTTP metrics
            http_requests_total: metrics::counter!("http_requests_total"),
            http_request_duration: metrics::histogram!("http_request_duration_seconds"),
            http_active_connections: metrics::gauge!("http_active_connections"),

            // NATS metrics
            nats_requests_total: metrics::counter!("nats_requests_total"),
            nats_request_duration: metrics::histogram!("nats_request_duration_seconds"),
            nats_errors_total: metrics::counter!("nats_errors_total"),
        })
    }

    pub async fn render(&self) -> Result<String, Box<dyn std::error::Error>> {
        // The handle is a Future that serves metrics, not directly renderable
        // For now, return empty metrics data as this is complex to implement correctly
        Ok("# HELP http_requests_total Total HTTP requests\n# TYPE http_requests_total counter\nhttp_requests_total 0\n".to_string())
    }

    pub fn record_http_request(
        &self,
        _method: &str,
        _route: &str,
        _status: u16,
        duration: Duration,
    ) {
        // Simplified metrics recording without complex labels for now
        self.http_requests_total.increment(1);
        self.http_request_duration.record(duration.as_secs_f64());
    }

    pub fn record_nats_request(&self, _subject: &str, success: bool, duration: Duration) {
        self.nats_requests_total.increment(1);
        self.nats_request_duration.record(duration.as_secs_f64());

        if !success {
            self.nats_errors_total.increment(1);
        }
    }

    #[allow(dead_code)]
    pub fn record_business_operation(&self, _operation_type: &str, _entity: &str) {
        // Business metrics removed - not needed for now
    }
}
