use std::time::{Duration, SystemTime};
use std::sync::{Mutex,Arc};
use std::thread;
use std::sync::mpsc;
use std::fmt::Debug;
use std::marker::Sync;
use uuid::Uuid;
use std::collections::{HashMap,BTreeSet};

pub mod stats;
pub mod reporting;
pub mod exporting;
pub mod runner;

pub trait TestContext : Default + Clone + Send {
    fn new(test_name: &'static str, test_suite: &'static str) -> Self;
    fn add_hit(&mut self, result: Result<(), i32>, duration: Duration);
    fn get_hits(&self) -> u128;
    fn get_successful_hits(&self) -> u128;
    fn get_unsuccessful_hits(&self) -> u128;
    fn get_session_id(&self) -> String;
    fn get_current_duration(&self) -> Duration;
    fn get_current_step_name(&self) -> String;
    fn get_current_mean_time(&self) -> u128;
    fn get_current_min_time(&self) -> u128;
    fn get_current_max_time(&self) -> u128;
    fn get_current_percentile_time(&self, percentile: f64) -> u128;
    fn get_current_errors(&self) -> HashMap<i32, u128>;
    fn set_current_step(&mut self, step_name: &'static str, stage_name: &'static str);
    fn set_current_duration(&mut self, duration: Duration);    
}

#[derive(Default,Clone,Debug)]
struct TestCaseMetrics {
    successful_hits: u128,
    unsuccessful_hits: u128,
    test_duration: Duration,
    elapsed_times: BTreeSet<u128>,
    errors: HashMap<i32, u128>
}

#[derive(Default,Clone,Debug)]
pub struct TestCaseContext<'a> {
    pub session_id: Uuid,
    pub test_name: &'a str,
    pub test_suite: &'a str,
    pub test_step_name: Option<&'a str>,
    pub test_stage_name: Option<&'a str>,
    test_metrics: TestCaseMetrics
}

pub struct TestCase<T: TestContext, U> {
    pub test_name: &'static str,
    pub test_suite: &'static str,
    pub test_context: Option<T>,
    pub test_steps: Vec<TestStep<T>>,
    pub data: U
}

pub struct TestStep<T> {
    step_name: &'static str,
    action: fn(&Arc::<Mutex::<T>>) -> Result<(), i32>,
    stages: Vec<TestStepStage>
}

pub struct TestStepStage {
    stage_name: &'static str,
    during: Duration,
    interval: Duration,
    rate: u32
}

impl<'a> TestContext for TestCaseContext<'a> {

    fn new(test_name: &'static str, test_suite: &'static str) -> Self {
        TestCaseContext {
            session_id: Uuid::new_v4(),
            test_name,
            test_suite,
            test_step_name: None,
            test_stage_name: None,
            test_metrics: TestCaseMetrics::default()
        }
    }

    fn get_hits(&self) -> u128 {
        self.test_metrics.successful_hits + self.test_metrics.unsuccessful_hits
    }

    fn add_hit(&mut self, result: Result<(), i32>, duration: Duration) {
        
        if let Err(code) = result {
            self.test_metrics.unsuccessful_hits +=1;
            *self.test_metrics.errors.entry(code).or_insert(0) += 1;
        } else {
            self.test_metrics.successful_hits += 1;
        }

        self.test_metrics.elapsed_times.insert(duration.as_millis());
    }

    fn get_session_id(&self) -> String {
        self.session_id.to_string()
    }

    fn set_current_step(&mut self, step_name: &'static str, stage_name: &'static str) {
        self.test_step_name = Some(step_name.clone());
        self.test_stage_name = Some(stage_name.clone());
    }

    fn set_current_duration(&mut self, duration: Duration) {
        self.test_metrics.test_duration = duration;
    }

    fn get_successful_hits(&self) -> u128 {
        self.test_metrics.successful_hits
    }

    fn get_unsuccessful_hits(&self) -> u128 {
        self.test_metrics.unsuccessful_hits
    }

    fn get_current_duration(&self) -> Duration {
        self.test_metrics.test_duration
    }

    fn get_current_step_name(&self) -> String {
        self.test_step_name.unwrap().to_string()
    }

    fn get_current_mean_time(&self) -> u128 {
        self.test_metrics.elapsed_times.iter().sum::<u128>() / self.test_metrics.elapsed_times.len() as u128
    }

    fn get_current_max_time(&self) -> u128 {
        *self.test_metrics.elapsed_times.last().unwrap_or(&0)
    }

    fn get_current_min_time(&self) -> u128 {
        *self.test_metrics.elapsed_times.first().unwrap_or(&0)
    }

    fn get_current_percentile_time(&self, percentile: f64) -> u128 {
        let index = (self.test_metrics.elapsed_times.len() - 1) as f64 * percentile;
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;
        
        let lowest_value = *self.test_metrics.elapsed_times.iter().nth(lower_index).unwrap_or(&0);
        let highest_value = *self.test_metrics.elapsed_times.iter().nth(upper_index).unwrap_or(&0);

        let interpolated_value = lowest_value as f64 + (index - lower_index as f64) * (highest_value - lowest_value) as f64;
        interpolated_value as u128
    }

    fn get_current_errors(&self) -> HashMap<i32, u128> {        
        self.test_metrics.errors.clone()
    }    
}

impl<'a, T, U> TestCase<T, U> 
    where T: TestContext + 'static + Sync + Debug, U: Default {
    
    pub fn new(test_name: &'static str, test_suite: &'static str, data: U) -> Self {        
        TestCase::<T, U> {
            test_name,
            test_suite,
            test_context : None,
            test_steps: Vec::default(),
            data
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
                            let action_start_time = SystemTime::now();                     
                            let action_result = action(&t_ctx);
                            let mut inner_ctx = t_ctx.lock().unwrap();
                            inner_ctx.add_hit(action_result, action_start_time.elapsed().unwrap());
                            inner_ctx.set_current_duration(start_time.elapsed().unwrap());
                            action_transmitter.send(inner_ctx.to_owned()).unwrap();
                        });
            
                        handles.push(handle);
                    }

                    thread::sleep(test_stage.interval);
                }
            }

            let mut step_ctx = ctx.lock().unwrap();
            step_ctx.set_current_duration(start_time.elapsed().unwrap());
            
            let step_transmitter = mpsc::Sender::clone(tx_step);
            step_transmitter.send(step_ctx.to_owned()).unwrap();
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
    pub fn new(step_name: &'static str, action: fn(&Arc::<Mutex::<T>>) -> Result<(), i32>) -> Self {
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