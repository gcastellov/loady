use std::time::{Duration, SystemTime};
use std::sync::{Mutex,Arc};
use std::thread;
use std::sync::mpsc;
use std::fmt::Debug;
use std::marker::Sync;
use uuid::Uuid;

pub mod reporting;

pub trait TestContext : Default + Clone + Copy + Send {
    fn new(test_name: &'static str, test_suite: &'static str) -> Self;
    fn add_hit(&mut self, result: bool);
    fn get_hits(&self) -> u64;
    fn get_successful_hits(&self) -> u64;
    fn get_unsuccessful_hits(&self) -> u64;
    fn get_session_id(&self) -> String;
    fn get_current_duration(&self) -> Option<Duration>;
    fn get_current_step_name(&self) -> String;
    fn set_current_step(&mut self, step_name: &'static str, stage_name: &'static str);
    fn set_current_duration(&mut self, duration: Duration);
}

#[derive(Default,Clone,Copy,Debug)]
pub struct TestCaseContext<'a, T> {
    pub session_id: Uuid,
    pub test_name: &'a str,
    pub test_suite: &'a str,
    pub test_step_name: Option<&'a str>,
    pub test_stage_name: Option<&'a str>,
    pub successful_hits: u64,
    pub unsuccessful_hits: u64,
    pub duration: Option<Duration>,
    pub data: T
}

pub struct TestCase<T: TestContext> {
    pub test_name: &'static str,
    pub test_suite: &'static str,
    pub test_context: Option<T>,
    pub test_steps: Vec<TestStep<T>>
}

pub struct TestStep<T> {
    step_name: &'static str,
    action: fn(&Arc::<Mutex::<T>>) -> bool,
    stages: Vec<TestStepStage>
}

pub struct TestStepStage {
    stage_name: &'static str,
    during: Duration,
    interval: Duration,
    rate: u32
}

impl<'a, T> TestContext for TestCaseContext<'a, T> 
    where T: Default + Clone + Copy + Send {

    fn new(test_name: &'static str, test_suite: &'static str) -> Self {
        TestCaseContext {
            session_id: Uuid::new_v4(),
            test_name,
            test_suite,
            test_step_name: None,
            test_stage_name: None,
            successful_hits: 0,
            unsuccessful_hits: 0,
            duration: None,
            data: T::default()
        }
    }

    fn get_hits(&self) -> u64 {
        self.successful_hits + self.unsuccessful_hits
    }

    fn add_hit(&mut self, result: bool) {
        if result {
            self.successful_hits += 1;
        } else {
            self.unsuccessful_hits +=1;
        }
    }

    fn get_session_id(&self) -> String {
        self.session_id.to_string()
    }

    fn set_current_step(&mut self, step_name: &'static str, stage_name: &'static str) {
        self.test_step_name = Some(step_name.clone());
        self.test_stage_name = Some(stage_name.clone());
    }

    fn set_current_duration(&mut self, duration: Duration) {
        self.duration = Some(duration);
    }

    fn get_successful_hits(&self) -> u64 {
        self.successful_hits
    }

    fn get_unsuccessful_hits(&self) -> u64 {
        self.unsuccessful_hits
    }

    fn get_current_duration(&self) -> Option<Duration> {
        self.duration
    }

    fn get_current_step_name(&self) -> String {
        self.test_step_name.unwrap().to_string()
    }
}

impl<T> TestCase<T> 
    where T: TestContext + 'static + Sync + Debug {
    
    pub fn new(test_name: &'static str, test_suite: &'static str) -> Self {        
        TestCase::<T> {
            test_name,
            test_suite,
            test_context : None,
            test_steps: Vec::default()
        }
    }

    pub fn with_step(&mut self, test_step: TestStep<T>) {        
        self.test_steps.push(test_step);
    }

    pub fn run(&mut self, tx_action: &std::sync::mpsc::Sender::<T>, tx_step: &std::sync::mpsc::Sender::<T>) {
        
        let start_time = SystemTime::now();
        let ctx = Arc::new(Mutex::new(T::new(self.test_name, self.test_suite)));
        let mut handles = Vec::default();

        for test_step in &self.test_steps {

            for test_stage in &test_step.stages {               

                let mut step_ctx = ctx.lock().unwrap();
                step_ctx.set_current_step(test_step.step_name, test_stage.stage_name);
                step_ctx.set_current_duration(start_time.elapsed().unwrap());
                drop(step_ctx);

                let stage_start_time = SystemTime::now();

                while stage_start_time.elapsed().unwrap() < test_stage.during {

                    for _ in 0..test_stage.rate {
                        let action_transmitter = mpsc::Sender::clone(tx_action);
                        let t_ctx = Arc::clone(&ctx);
                        let action = test_step.action.clone();
            
                        let handle = thread::spawn(move || {                                 
                            let action_result = action(&t_ctx);
                            let mut inner_ctx = t_ctx.lock().unwrap();
                            inner_ctx.add_hit(action_result);
                            inner_ctx.set_current_duration(start_time.elapsed().unwrap());
                            action_transmitter.send(*inner_ctx).unwrap();
                            drop(inner_ctx);
                        });
            
                        handles.push(handle);
                    }

                    thread::sleep(test_stage.interval);
                }
            }

            let mut step_ctx = ctx.lock().unwrap();
            step_ctx.set_current_duration(start_time.elapsed().unwrap());
            
            let step_transmitter = mpsc::Sender::clone(tx_step);
            step_transmitter.send(*step_ctx).unwrap();
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut context = ctx.lock().unwrap();
        context.set_current_duration(start_time.elapsed().unwrap());
        self.test_context = Some(context.clone());        
    }
}

impl<T: TestContext> TestStep<T> {
    pub fn new(step_name: &'static str, action: fn(&Arc::<Mutex::<T>>) -> bool) -> Self {
        TestStep {
            step_name,
            action,
            stages: Vec::default()
        }
    }

    pub fn with_stage(&mut self, stage_name: &'static str, during: Duration, interval: Duration, rate: u32) {
        let stage = TestStepStage {
            stage_name,
            during,
            interval,
            rate
        };

        self.stages.push(stage);
    }
}