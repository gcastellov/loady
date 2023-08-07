use loady::core::stats::Metrics;

pub const TEST_NAME: &'static str = "simple sample";
pub const TEST_SUITE: &'static str = "samples";
pub const TEST_STEP_1: &'static str = "first";
pub const TEST_STEP_2: &'static str = "second";
pub const TEST_STAGE_1: &'static str = "warm up";
pub const TEST_STAGE_2: &'static str = "load";

#[derive(Default,Clone,Debug)]
pub struct InnerContext {
    client_id: String,
    secret: String
}

pub fn assert_blank_metrics(metrics: &Metrics) {
    assert_eq!(metrics.test_duration, 0);
    assert_eq!(metrics.mean_time, 0);
    assert_eq!(metrics.min_time, 0);
    assert_eq!(metrics.max_time, 0);
    assert_eq!(metrics.p90_time, 0);
    assert_eq!(metrics.p95_time, 0);
    assert_eq!(metrics.p99_time, 0);
    assert_eq!(metrics.positive_hits, 0);
    assert_eq!(metrics.negative_hits, 0);
    assert_eq!(metrics.all_hits, 0);
    assert_eq!(metrics.errors.len(), 0);
}