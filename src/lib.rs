use std::sync::{Arc};
use std::time::{Duration};
use std::fmt::Debug;
use std::marker::Sync;
use crate::core::{TestCase,TestStep,TestStepStage};
use crate::core::context::{TestCaseContext};

pub mod core;

pub struct TestCaseBuilder<'a, T> 
    where T: 'static + Default + Clone + Send + Debug + Sync {
    pub test_case: TestCase<TestCaseContext<'a>, T>
}

impl<T> TestCaseBuilder<'static, T> 
    where T: 'static + Default + Clone + Send + Debug + Sync {
    
    pub fn new(test_name: &'static str, test_suite: &'static str, data: &T) -> Self {
        TestCaseBuilder {
            test_case: TestCase::<TestCaseContext, T>::new(test_name, test_suite, data.to_owned())
        }
    }

    pub fn with_init_step(mut self, action: fn(T) -> Result<T, i32>) -> Self {
        let step = TestStep::as_init(action);
        self.test_case.with_step(step);
        self
    }

    pub fn with_warm_up_step(mut self, action: fn(&Arc::<T>)) -> Self {
        let step = TestStep::as_warm_up(action, Vec::default());
        self.test_case.with_step(step);
        self
    }

    pub fn with_load_step(mut self, name: &'static str, action: fn(&Arc::<T>) -> Result<(), i32>) -> Self {   
        let step = TestStep::as_load(name, action, Vec::default());
        self.test_case.with_step(step);
        self
    }

    pub fn with_clean_up_step(mut self, action: fn(T)) -> Self {
        let step = TestStep::as_clean_up(action);
        self.test_case.with_step(step);
        self
    }

    pub fn with_stage(mut self, stage_name: &'static str, during: Duration, interval: Duration, rate: u32) -> Self {

        if let Some(step) = self.test_case.test_steps.last_mut() {
            let stage = TestStepStage::new(stage_name, during, interval, rate);
            match step {
                TestStep::<T>::Warmup { stages, .. } => stages.push(stage),
                TestStep::<T>::Load { stages, .. } => stages.push(stage),
                _ => panic!("Only 'Warm up' and 'Load' step types can use stages")
            };
        }
        
        self
    }

    pub fn build(self) -> TestCase::<TestCaseContext::<'static>, T> {
        self.test_case
    }
}

