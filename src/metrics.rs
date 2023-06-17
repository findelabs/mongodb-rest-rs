use axum::{http::Request, middleware::Next, response::IntoResponse};
use core::time::Duration;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use metrics_util::MetricKindMask;


pub fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::GAUGE,
            Some(Duration::from_secs(10)),
        )
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

pub async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {

    let _path = req.uri().path().to_owned();
    let _method = req.method().clone();
    let response = next.run(req).await;
    let _status = response.status().as_u16().to_string();
    response
}
