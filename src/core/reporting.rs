use std::time::{Duration};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub test_duration: Duration,
    pub mean_time: Duration,
    pub max_time: Duration,
    pub min_time: Duration,
    pub positive_hits: u64,
    pub negative_hits: u64,
    pub all_hits: u64
}

#[derive(Clone, Debug)]
pub struct StepStatus {
    pub session_id: String,
    pub test_name: String,
    pub step_name: String, 
    pub status: Metrics
}

#[derive(Clone, Debug)]
pub struct TestStatus {
    pub session_id: String,
    pub test_name: String,
    pub status: Metrics
}

#[derive(Default, Clone)]
pub struct DefaultReportingSink;

pub trait ReportingSink : Sync + Send {
    fn on_tests_ended(&self, status: TestStatus);
    fn on_step_ended(&self, status: StepStatus);
    fn on_action_ended(&self, step_status: StepStatus);
}

impl ReportingSink for DefaultReportingSink { 
    fn on_tests_ended(&self, test_status: TestStatus) {
        println!("Test has ended: {:?}", test_status);
    }

    fn on_step_ended(&self, step_status: StepStatus) {
        println!("Test step has ended: {:?}", step_status);
    }

    fn on_action_ended(&self, step_status: StepStatus) {
        println!("Test action has ended: {:?}", step_status);
    }
}

impl TestStatus  {
    pub fn new(session_id: String, test_name: String, test_duration: Duration, positive_hits: u64, negative_hits: u64, min_time: Duration, max_time: Duration, mean_time: Duration) -> Self {
        TestStatus {
            session_id: session_id,
            test_name: test_name,
            status: Metrics::new(
                test_duration,
                positive_hits,
                negative_hits,
                min_time, 
                max_time,
                mean_time
            )
        }
    }
}

impl StepStatus  {
    pub fn new(session_id: String, test_name: String, step_name: String, test_duration: Duration, positive_hits: u64, negative_hits: u64, min_time: Duration, max_time: Duration, mean_time: Duration) -> Self {
        StepStatus {
            session_id: session_id,
            test_name: test_name,
            step_name: step_name,
            status: Metrics::new(
                test_duration,
                positive_hits,
                negative_hits,
                min_time, 
                max_time,
                mean_time
            )
        }
    }
}

impl Metrics {
    fn new(test_duration: Duration, positive_hits: u64, negative_hits: u64, min_time: Duration, max_time: Duration, mean_time: Duration) -> Self {
        Metrics {
            test_duration,
            positive_hits,
            negative_hits,
            min_time,
            max_time,
            mean_time,
            all_hits: positive_hits + negative_hits
        }
    }
}
