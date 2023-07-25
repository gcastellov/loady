use std::time::{Duration};
use std::fmt::{Formatter,Result,Display};
use num_format::{Locale, ToFormattedString};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub test_duration: Duration,
    pub mean_time: Duration,
    pub max_time: Duration,
    pub min_time: Duration,
    pub positive_hits: u128,
    pub negative_hits: u128,
    pub all_hits: u128
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
    pub fn new(session_id: String, test_name: String, test_duration: Duration, positive_hits: u128, negative_hits: u128, min_time: Duration, max_time: Duration, mean_time: Duration) -> Self {
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
    pub fn new(session_id: String, test_name: String, step_name: String, test_duration: Duration, positive_hits: u128, negative_hits: u128, min_time: Duration, max_time: Duration, mean_time: Duration) -> Self {
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
    fn new(test_duration: Duration, positive_hits: u128, negative_hits: u128, min_time: Duration, max_time: Duration, mean_time: Duration) -> Self {
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

impl Display for TestStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{: <20}: {}\r\n{: <20}: {}\r\n\r\n{}", 
            "Session ID",
            self.session_id, 
            "Test Case",
            self.test_name,
            self.status)
    }
}

impl Display for StepStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{: <20}: {}\r\n\r\n{}", 
            "Test Step",
            self.step_name,
            self.status)
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {


        let format_number = |num: &u128| -> String {
            num.to_formatted_string(&Locale::en)
        };

        let format_duration = |duration: &Duration| -> String {
            format_number(&duration.as_millis())
        };

        write!(f, "{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n\r\n{: <20}: {}\r\n{: <20}: {}\r\n{: <20}: {}", 
            "Test Duration",
            format_duration(&self.test_duration),
            "Min Time",
            format_duration(&self.min_time),
            "Mean Time", 
            format_duration(&self.mean_time),
            "Max Time",
            format_duration(&self.max_time),
            "All Hits",
            format_number(&self.all_hits),
            "Successful hits",
            format_number(&self.positive_hits),
            "Unsuccessul hits",
            format_number(&self.negative_hits)
        )
    }
}
