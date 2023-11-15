use crate::core::context::TestContext;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
pub struct Metrics {
    pub test_duration: u128,
    pub mean_time: u128,
    pub max_time: u128,
    pub min_time: u128,
    pub std_dev: u128,
    pub p90_time: u128,
    pub p95_time: u128,
    pub p99_time: u128,
    pub positive_hits: u128,
    pub negative_hits: u128,
    pub all_hits: u128,
    pub errors: HashMap<i32, u128>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StepStatus {
    pub session_id: String,
    pub test_name: String,
    pub step_name: String,
    pub metrics: Metrics,
}

#[derive(Clone, Debug, Serialize)]
pub struct TestStatus {
    pub session_id: String,
    pub test_name: String,
    pub metrics: Metrics,
}

impl TestStatus {
    pub fn new(test_name: String, test_context: impl TestContext) -> Self {
        TestStatus {
            test_name,
            session_id: test_context.get_session_id(),
            metrics: Metrics::new(test_context),
        }
    }
}

impl StepStatus {
    pub fn new(test_name: String, test_context: impl TestContext) -> Self {
        StepStatus {
            test_name,
            session_id: test_context.get_session_id(),
            step_name: test_context.get_current_step_name(),
            metrics: Metrics::new(test_context),
        }
    }
}

impl Metrics {
    const P90: f64 = 0.9;
    const P95: f64 = 0.95;
    const P99: f64 = 0.99;

    fn new(test_context: impl TestContext) -> Self {
        Metrics {
            test_duration: test_context.get_current_duration().as_millis(),
            positive_hits: test_context.get_successful_hits(),
            negative_hits: test_context.get_unsuccessful_hits(),
            all_hits: test_context.get_successful_hits() + test_context.get_unsuccessful_hits(),
            min_time: test_context.get_current_min_time(),
            max_time: test_context.get_current_max_time(),
            mean_time: test_context.get_current_mean_time(),
            std_dev: test_context.get_current_std_dev(),
            p90_time: test_context.get_current_percentile_time(Self::P90),
            p95_time: test_context.get_current_percentile_time(Self::P95),
            p99_time: test_context.get_current_percentile_time(Self::P99),
            errors: test_context.get_current_errors(),
        }
    }
}
