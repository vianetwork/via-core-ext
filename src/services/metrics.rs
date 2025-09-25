use std::time::Duration;

use vise::{Buckets, Counter, Histogram, Metrics, Unit};

#[derive(Debug, Metrics)]
#[metrics(prefix = "da")]
pub struct DaMetrics {
    /// Number of blobs dispatched
    pub dispatched_blobs: Counter,

    /// Number of inclusion queries
    pub inclusion_queries: Counter,

    /// Dispatch latency in seconds
    #[metrics(buckets = Buckets::LATENCIES, unit = Unit::Seconds)]
    pub dispatch_latency: Histogram<Duration>,
}

#[vise::register]
pub(crate) static DA_METRICS: vise::Global<DaMetrics> = vise::Global::new();
