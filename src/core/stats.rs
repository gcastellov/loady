use std::time::{Duration};
use crate::core::{TestContext};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub test_duration: Duration,
    pub mean_time: Duration,
    pub max_time: Duration,
    pub min_time: Duration,
    pub positive_hits: u128,
    pub negative_hits: u128,
    pub all_hits: u128
}

#[derive(Clone, Debug)]
pub struct StepStatus {
    pub session_id: String,
    pub test_name: String,
    pub step_name: String, 
    pub metrics: Metrics
}

#[derive(Clone, Debug)]
pub struct TestStatus {
    pub session_id: String,
    pub test_name: String,
    pub metrics: Metrics
}

impl TestStatus  {
    pub fn new(test_name: String, test_context: Box<impl TestContext>) -> Self {
        TestStatus {
            test_name: test_name,
            session_id: test_context.get_session_id(),
            metrics: Metrics::new(test_context)
        }
    }
}

impl StepStatus  {
    pub fn new(test_name: String, test_context: Box<impl TestContext>) -> Self {
        StepStatus {
            test_name: test_name,
            session_id: test_context.get_session_id(),
            step_name: test_context.get_current_step_name(),
            metrics: Metrics::new(test_context)
        }
    }
}

impl Metrics {
    fn new(test_context: Box<impl TestContext>) -> Self {
        Metrics {
            test_duration: test_context.get_current_duration(),
            positive_hits: test_context.get_successful_hits(),
            negative_hits: test_context.get_unsuccessful_hits(),
            min_time: test_context.get_current_min_time(), 
            max_time: test_context.get_current_max_time(),
            mean_time: test_context.get_current_mean_time(),
            all_hits: test_context.get_successful_hits() + test_context.get_unsuccessful_hits()
        }
    }
}
