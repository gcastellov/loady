use std::sync::{mpsc,Mutex,Arc};
use std::thread;
use std::time::{Duration};
use std::fmt::Debug;
use std::marker::Sync;
use crate::core::reporting::{ReportingSink,DefaultReportingSink,TestStatus,StepStatus};
use crate::core::{TestCase,TestStep,TestContext,TestCaseContext};

pub mod core;

#[derive(Default)]
pub struct Runner {
    reporting_sinks: Vec<Arc<Box<dyn ReportingSink>>>
}

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

    pub fn with_step(mut self, step_name: &'static str, action: fn(&Arc::<Mutex::<TestCaseContext::<T>>>) -> bool) -> Self {
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

impl Runner {
    
    pub fn run<T>(&self, mut test_case: TestCase<T>)
        where T: TestContext + 'static + Sync + Debug {

        let report_step_status = |is_action: bool, step_status: StepStatus, sinks: Vec<Arc<Box<dyn ReportingSink>>>| {
            let mut sink_handles = Vec::default();
            
            for sink in sinks {
                let cloned_sink = Arc::clone(&sink);
                let status = step_status.to_owned();
                let sink_handle = thread::spawn(move || {
                    if is_action {
                        cloned_sink.on_action_ended(status);
                    } else {

                        cloned_sink.on_step_ended(status);
                    }
                });

                sink_handles.push(sink_handle);
            }

            for handle in sink_handles {
                handle.join().unwrap();
            }
        };


        let (tx_action, rx_action) = mpsc::channel::<T>();
        let (tx_step, rx_step) = mpsc::channel::<T>();

        let action_sinks: Vec<Arc<Box<dyn ReportingSink>>> = self.reporting_sinks.iter().map(|sink|Arc::clone(&sink)).collect();
        let step_sinks: Vec<Arc<Box<dyn ReportingSink>>> = self.reporting_sinks.iter().map(|sink|Arc::clone(&sink)).collect();

        let t_action_join = thread::spawn(move || { 
            while let Ok(inner_ctx) = rx_action.recv() {
                let action_sinks = action_sinks.clone();
                if action_sinks.is_empty() {
                    continue;
                }

                let step_status = StepStatus::new(
                    inner_ctx.get_session_id(),
                    test_case.test_name.to_owned(),
                    inner_ctx.get_current_step_name(), 
                    inner_ctx.get_current_duration().unwrap(), 
                    inner_ctx.get_successful_hits(), 
                    inner_ctx.get_unsuccessful_hits());

                report_step_status(true, step_status, action_sinks);
                thread::sleep(Duration::from_millis(50));
            }
        });


        let t_step_join = thread::spawn(move || { 

            while let Ok(inner_ctx) = rx_step.recv() {
                let step_sinks = step_sinks.clone();
                if step_sinks.is_empty() {
                    continue;
                }

                let step_status = StepStatus::new(
                    inner_ctx.get_session_id(),
                    test_case.test_name.to_owned(),
                    inner_ctx.get_current_step_name(), 
                    inner_ctx.get_current_duration().unwrap(), 
                    inner_ctx.get_successful_hits(), 
                    inner_ctx.get_unsuccessful_hits());

                report_step_status(false, step_status, step_sinks);
                thread::sleep(Duration::from_millis(50));
            }
        });

        test_case.run(&tx_action, &tx_step);

        drop(tx_action);
        drop(tx_step);

        t_action_join.join().unwrap();
        t_step_join.join().unwrap();
        
        self.report_test_status(&test_case);        
    }

    fn report_test_status<T>(&self, test_case: &TestCase<T>)
        where T: TestContext + 'static + Sync + Debug {
        if !self.reporting_sinks.is_empty() {
            let ctx = test_case.test_context.unwrap();

            let test_status = TestStatus::new(
                ctx.get_session_id(),
                test_case.test_name.to_owned(), 
                ctx.get_current_duration().unwrap(), 
                ctx.get_successful_hits(), 
                ctx.get_unsuccessful_hits());

            let arc_test_status = Arc::new(Mutex::new(test_status));
            let mut sink_handles = Vec::default();

            for sink in &self.reporting_sinks {

                let status = Arc::clone(&arc_test_status);
                let cloned_sink = Arc::clone(sink);

                let sink_handle = thread::spawn(move || {
                    let t_status = status.lock().unwrap().clone();
                    cloned_sink.on_tests_ended(t_status);
                });

                sink_handles.push(sink_handle);
            }

            for handle in sink_handles {
                handle.join().unwrap();
            }
        }
    }

    pub fn with_default_reporting_sink(&mut self)  {
        let sink = DefaultReportingSink::default();
        self.with_reporting_sink(sink);
    }

    fn with_reporting_sink<T: ReportingSink + 'static>(&mut self, sink: T) {
        self.reporting_sinks.push(Arc::new(Box::new(sink)));
    }
}