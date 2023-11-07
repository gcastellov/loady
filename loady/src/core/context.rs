use tokio::time::Duration;
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
    fn get_current_std_dev(&self) -> u128;
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

    fn get_current_std_dev(&self) -> u128 {
        let mean_time = self.get_current_mean_time() as i128;
        let sum = self.test_metrics.elapsed_times
            .iter()
            .map(|time|(*time as i128 - mean_time).pow(2))
            .sum::<i128>();

        let div = sum.checked_div(self.test_metrics.elapsed_times.len() as i128).unwrap_or(0) as f64;
        f64::sqrt(div).round() as u128
    }

    fn get_current_errors(&self) -> HashMap<i32, u128> {        
        self.test_metrics.errors.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn seed_with_hits(ctx: &mut impl TestContext) {
        let hits = vec![
            (Ok(()), Duration::from_millis(100)), 
            (Ok(()), Duration::from_millis(130)), 
            (Ok(()), Duration::from_millis(80)), 
            (Err(401), Duration::from_millis(200)),
            (Err(402), Duration::from_millis(300)),
            (Ok(()), Duration::from_millis(150)), 
        ];

        for (result, duration) in hits {
            ctx.add_hit(result, duration);
        }
    }

    #[test]
    fn given_default_test_context_when_getting_values_then_returns_defaults() {
        let ctx = TestCaseContext::default();

        assert!(!ctx.get_session_id().is_empty());
        assert_eq!(ctx.get_current_step_name(), String::from(""));
        assert_eq!(ctx.get_current_duration(), Duration::default());
        assert_eq!(ctx.get_current_errors(), HashMap::default());        
        assert_eq!(ctx.get_current_min_time(), 0);
        assert_eq!(ctx.get_current_mean_time(), 0);
        assert_eq!(ctx.get_current_max_time(), 0);
        assert_eq!(ctx.get_current_percentile_time(0.95), 0);
        assert_eq!(ctx.get_current_percentile_time(0.97), 0);
        assert_eq!(ctx.get_current_percentile_time(0.99), 0);
        assert_eq!(ctx.get_hits(), 0);
        assert_eq!(ctx.get_successful_hits(), 0);
        assert_eq!(ctx.get_unsuccessful_hits(), 0);
    }

    #[test]
    fn given_set_of_results_when_getting_min_time_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_current_min_time();

        assert_eq!(actual, 80);
    }

    #[test]
    fn given_set_of_results_when_getting_max_time_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_current_max_time();

        assert_eq!(actual, 300);
    }

    #[test]
    fn given_set_of_results_when_getting_mean_time_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_current_mean_time();

        assert_eq!(actual, 160);
    }

    #[test]
    fn given_set_of_results_when_getting_std_dev_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_current_std_dev();

        assert_eq!(actual, 73);
    }

    #[test]
    fn given_set_of_results_when_getting_successful_hits_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_successful_hits();

        assert_eq!(actual, 4);
    }

    #[test]
    fn given_set_of_results_when_getting_unsuccessful_hits_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_unsuccessful_hits();

        assert_eq!(actual, 2);
    }

    #[test]
    fn given_set_of_results_when_getting_all_hits_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_hits();

        assert_eq!(actual, 6);
    }

    #[test]
    fn given_set_of_results_when_getting_errors_then_returns_exepected_value() {
        let mut ctx = TestCaseContext::default();
        seed_with_hits(&mut ctx);

        let actual = ctx.get_current_errors();

        assert_eq!(actual.len(), 2);
        assert_eq!(actual.get(&401), Some(&1));
        assert_eq!(actual.get(&402), Some(&1));
    }

    #[test]
    fn given_step_name_when_getting_current_step_name_then_returns_expected_value() {
        const STEP_NAME: &str = "STEP NAME";
        const STAGE_NAME: &str  = "STAGE NAME";        
        let mut ctx = TestCaseContext::default();
        ctx.set_current_step(STEP_NAME, STAGE_NAME);

        let actual = ctx.get_current_step_name();

        assert_eq!(actual, STEP_NAME);
    }

    #[test]
    fn given_test_duration_when_getting_current_duration_then_returns_expected_value() {        
        let duration = Duration::from_secs(2);
        let mut ctx = TestCaseContext::default();
        ctx.set_current_duration(duration);

        let actual = ctx.get_current_duration();
        
        assert_eq!(actual, duration);
    }
}