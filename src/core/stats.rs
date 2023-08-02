use std::collections::HashMap;
use crate::core::{TestContext};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub test_duration: u128,
    pub mean_time: u128,
    pub max_time: u128,
    pub min_time: u128,
    pub positive_hits: u128,
    pub negative_hits: u128,
    pub all_hits: u128,
    pub errors: HashMap<i32, u128>
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
            test_duration: test_context.get_current_duration().as_millis(),
            positive_hits: test_context.get_successful_hits(),
            negative_hits: test_context.get_unsuccessful_hits(),
            min_time: test_context.get_current_min_time().as_millis(), 
            max_time: test_context.get_current_max_time().as_millis(),
            mean_time: test_context.get_current_mean_time().as_millis(),
            all_hits: test_context.get_successful_hits() + test_context.get_unsuccessful_hits(),
            errors: test_context.get_current_errors()
        }
    }
}
