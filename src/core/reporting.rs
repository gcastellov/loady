use crate::core::stats::{TestStatus,StepStatus};

#[derive(Default, Clone)]
pub struct DefaultReportingSink;

pub trait ReportingSink : Sync + Send {
    fn on_test_ended(&self, status: TestStatus);
    fn on_load_step_ended(&self, status: StepStatus);
    fn on_load_action_ended(&self, step_status: StepStatus);
    fn on_internal_step_ended(&self, step_name: &str);
}

impl ReportingSink for DefaultReportingSink { 
    fn on_test_ended(&self, test_status: TestStatus) {
        println!("Test has ended: {:?}", test_status);
    }

    fn on_load_step_ended(&self, step_status: StepStatus) {
        println!("Test step has ended: {:?}", step_status);
    }

    fn on_load_action_ended(&self, step_status: StepStatus) {
        println!("Test action has ended: {:?}", step_status);
    }

    fn on_internal_step_ended(&self, step_name: &str) {
        println!("Test step has ended: {}", step_name);
    }
}