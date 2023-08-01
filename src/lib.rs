use std::sync::{Mutex,Arc};
use std::time::{Duration};
use std::fmt::Debug;
use std::marker::Sync;
use crate::core::{TestCase,TestStep,TestCaseContext};

pub mod core;

pub struct TestCaseBuilder<'a, T> 
    where T: 'static + Default + Clone + Copy + Send + Debug + Sync {
    pub test_case: TestCase<TestCaseContext<'a, T>>
}

impl<T> TestCaseBuilder<'static, T> 
    where T: 'static + Default + Clone + Copy + Send + Debug + Sync {
    
    pub fn new(test_name: &'static str, test_suite: &'static str) -> Self {
        TestCaseBuilder {
            test_case: TestCase::<TestCaseContext<T>>::new(test_name, test_suite)
        }
    }

    pub fn with_step(mut self, step_name: &'static str, action: fn(&Arc::<Mutex::<TestCaseContext::<T>>>) -> Result<(), i32>) -> Self {
        let step = TestStep::new(step_name, action);
        self.test_case.with_step(step);
        self
    }

    pub fn with_stage(mut self, stage_name: &'static str, during: Duration, interval: Duration, rate: u32) -> Self {
        
        if let Some(step) = self.test_case.test_steps.last_mut() {
            step.with_stage(stage_name, during, interval, rate);
        }
        
        self
    }

    pub fn build(self) -> TestCase::<TestCaseContext::<'static , T>> {
        self.test_case
    }
}

