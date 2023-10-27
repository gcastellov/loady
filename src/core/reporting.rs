use async_trait::async_trait;
use crate::core::stats::{TestStatus,StepStatus};

#[derive(Default, Clone)]
pub struct DefaultReportingSink;

#[async_trait]
pub trait ReportingSink : Sync + Send {
    async fn on_test_ended(&self, status: TestStatus);
    async fn on_load_step_ended(&self, status: StepStatus);
    async fn on_load_action_ended(&self, step_status: StepStatus);
    async fn on_internal_step_ended(&self, step_name: &str);
}

#[async_trait]
impl ReportingSink for DefaultReportingSink { 
    async fn on_test_ended(&self, test_status: TestStatus) {
        println!("Test has ended: {:?}", test_status);
    }

    async fn on_load_step_ended(&self, step_status: StepStatus) {
        println!("Test step has ended: {:?}", step_status);
    }

    async fn on_load_action_ended(&self, step_status: StepStatus) {
        println!("Test action has ended: {:?}", step_status);
    }

    async fn on_internal_step_ended(&self, step_name: &str) {
        println!("Test step has ended: {}", step_name);
    }
}