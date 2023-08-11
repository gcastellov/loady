use std::time::{Duration};
use std::collections::{HashMap,BTreeSet};
use uuid::Uuid;

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
pub struct TestCaseContext<'a> {
    pub session_id: Uuid,
    pub test_name: &'a str,
    pub test_suite: &'a str,
    pub test_step_name: Option<&'a str>,
    pub test_stage_name: Option<&'a str>,
    test_metrics: TestContextMetrics
}

#[derive(Default,Clone,Debug)]
struct TestContextMetrics {
    successful_hits: u128,
    unsuccessful_hits: u128,
    test_duration: Duration,
    elapsed_times: BTreeSet<u128>,
    errors: HashMap<i32, u128>
}

impl<'a> TestContext for TestCaseContext<'a> {

    fn new(test_name: &'static str, test_suite: &'static str) -> Self {
        TestCaseContext {
            session_id: Uuid::new_v4(),
            test_name,
            test_suite,
            test_step_name: None,
            test_stage_name: None,
            test_metrics: TestContextMetrics::default()
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
        self.test_step_name.unwrap_or("").to_string()
    }

    fn get_current_mean_time(&self) -> u128 {
        let sum = self.test_metrics.elapsed_times.iter().sum::<u128>();
        sum.checked_div(self.test_metrics.elapsed_times.len() as u128).unwrap_or(0)
    }

    fn get_current_max_time(&self) -> u128 {
        *self.test_metrics.elapsed_times.last().unwrap_or(&0)
    }

    fn get_current_min_time(&self) -> u128 {
        *self.test_metrics.elapsed_times.first().unwrap_or(&0)
    }

    fn get_current_percentile_time(&self, percentile: f64) -> u128 {

        let calc_percentile = |value: usize| -> u128 {
            let index = value as f64 * percentile;
            let lower_index = index.floor() as usize;
            let upper_index = index.ceil() as usize;
            
            let lowest_value = *self.test_metrics.elapsed_times.iter().nth(lower_index).unwrap_or(&0);
            let highest_value = *self.test_metrics.elapsed_times.iter().nth(upper_index).unwrap_or(&0);
    
            let interpolated_value = lowest_value as f64 + (index - lower_index as f64) * (highest_value - lowest_value) as f64;
            interpolated_value as u128
        };

        match self.test_metrics.elapsed_times.len().checked_sub(1) {
            Some(value) => calc_percentile(value),
            _ => 0
        }
    }

    fn get_current_errors(&self) -> HashMap<i32, u128> {        
        self.test_metrics.errors.clone()
    }
}