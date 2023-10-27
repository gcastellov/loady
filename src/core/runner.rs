use tokio::time::{Instant,Duration};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use std::fmt::Debug;
use std::marker::Sync;
use std::io::{Error};
use std::sync::Arc;
use crate::core::reporting::{ReportingSink,DefaultReportingSink};
use crate::core::exporting::{Exporter,FileType};
use crate::core::stats::{TestStatus,StepStatus};
use crate::core::context::TestContext;
use crate::core::composition::{TestCase};

pub struct TestRunner {
    reporting_sinks: Vec<Arc<Box<dyn ReportingSink>>>,
    exporter: Exporter,
    use_summary: bool,
    reporting_frequency: Duration
}

impl<'a> TestRunner {

    const DEFAULT_REPORTING_FREQUENCY: Duration = Duration::from_secs(5);

    pub fn new() -> Self {
        TestRunner {
            reporting_sinks: Vec::default(),
            exporter: Exporter::default(),
            use_summary: false,
            reporting_frequency: Self::DEFAULT_REPORTING_FREQUENCY
        }
    }
  
    pub async fn run<T, U>(&self, mut test_case: TestCase<'a, T, U>) -> Result<TestStatus, &str>
        where T: TestContext + 'static + Sync + Debug, U: 'static + Clone + Sync + Send {

        let report_step_status = |is_action: bool, step_status: StepStatus, sinks: Vec<Arc<Box<dyn ReportingSink>>>| async move {
            for sink in sinks {
                let cloned_sink = Arc::clone(&sink);
                let status = step_status.to_owned();

                if is_action {
                    cloned_sink.on_load_action_ended(status).await;
                } else {
                    cloned_sink.on_load_step_ended(status).await;
                }
            }
        };

        let (tx_load_action, mut rx_load_action) = mpsc::channel::<T>(10);
        let (tx_load_step, mut rx_load_step) = mpsc::channel::<T>(10);
        let (tx_internal_step, mut rx_internal_step) = mpsc::channel::<&str>(10);

        let action_sinks: Vec<Arc<Box<dyn ReportingSink>>> = self.reporting_sinks.clone();
        let step_sinks: Vec<Arc<Box<dyn ReportingSink>>> = self.reporting_sinks.clone();
        let internal_step_sinks: Vec<Arc<Box<dyn ReportingSink>>> = self.reporting_sinks.clone();
        
        let mut stats_by_step: Arc<Mutex<Vec<StepStatus>>> = Arc::new(Mutex::new(Vec::default()));
        let arc_stats_by_step = Arc::clone(&mut stats_by_step);
        let reporting_frequency = self.reporting_frequency.to_owned();
        
        let t_action_join = tokio::spawn(async move { 
            let mut frequency_instant = Instant::now();            
            while let Some(inner_ctx) = rx_load_action.recv().await {
                let action_sinks = action_sinks.clone();
                if !action_sinks.is_empty() && frequency_instant.elapsed() > reporting_frequency {

                    let step_status = StepStatus::new(
                        test_case.test_name.to_owned(), 
                        Box::new(inner_ctx));

                    report_step_status(true, step_status, action_sinks).await;
                    frequency_instant = Instant::now();                    
                }
            }
        });

        let t_step_join = tokio::spawn(async move { 
            while let Some(inner_ctx) = rx_load_step.recv().await {
                let step_sinks = step_sinks.clone();
                let step_status = StepStatus::new(
                    test_case.test_name.to_owned(), 
                    Box::new(inner_ctx));

                if !step_sinks.is_empty() {
                    report_step_status(false, step_status.to_owned(), step_sinks).await;
                }

                arc_stats_by_step
                    .lock()
                    .await
                    .push(step_status);
            }
        });

        let t_internal_step_join = tokio::spawn(async move { 
            while let Some(step_name) = rx_internal_step.recv().await {
                let internal_step_sinks = internal_step_sinks.clone();
                if !internal_step_sinks.is_empty() {
                    for sink in internal_step_sinks {
                        let cloned_sink = Arc::clone(&sink);
                        cloned_sink.on_internal_step_ended(step_name).await;
                    }
                }
            }
        });

        test_case.run(&tx_load_action, &tx_load_step, &tx_internal_step).await?;

        drop(tx_load_action);
        drop(tx_load_step);
        drop(tx_internal_step);

        _ = t_action_join.await;
        _ = t_step_join.await;
        _ = t_internal_step_join.await;

        let by_step: Vec<StepStatus> = stats_by_step.lock().await.clone();
        let test_status = self.report_test_status(&test_case, &by_step).await;

        match test_status {
            Ok(status) => Ok(status),
            Err(_) => Err("An error occurred while exporting the test status")
        }
    }

    pub fn with_default_reporting_sink(mut self) -> Self {
        let sink = DefaultReportingSink::default();
        self.with_reporting_sink(sink);
        self
    }

    pub fn with_reporting_sink<T: ReportingSink + 'static>(&mut self, sink: T) -> &Self {
        self.reporting_sinks.push(Arc::new(Box::new(sink)));
        self
    }

    pub fn with_default_output_files(mut self) -> Self {
        self.exporter.with_default_output_files();
        self
    }

    pub fn with_output_file(mut self, file_type: FileType, directory: &str, file_name: &str) -> Self {
        self.exporter.with_output_file(file_type, directory.to_string(), file_name.to_string());
        self
    }

    pub fn with_test_summary_std_out(mut self) -> Self {
        self.use_summary = true;
        self
    }

    pub fn with_reporting_frequency(mut self, seconds: u64) -> Self {
        if Self::DEFAULT_REPORTING_FREQUENCY.as_secs() > seconds {
            panic!("Reporting frequency must be greater than the default value {}", Self::DEFAULT_REPORTING_FREQUENCY.as_secs())
        }

        self.reporting_frequency = Duration::from_secs(seconds);
        self
    }
   
    async fn report_test_status<T, U>(&self, test_case: &TestCase<'a, T, U>, stats_by_step: &Vec<StepStatus>) -> Result<TestStatus, Error>
        where T: TestContext + 'static + Sync + Debug {

        let ctx = test_case.test_context.clone().unwrap_or(T::default());

        let test_status = TestStatus::new(
            test_case.test_name.to_owned(), 
            Box::new(ctx));

        self.write_to_sinks(test_status.to_owned()).await?;
        self.exporter.write_output_files(test_status.to_owned(), stats_by_step.to_owned())?;

        if self.use_summary {
            let content = FileType::Txt.get_content(test_status.to_owned(), stats_by_step.to_owned());
            println!("\r\n{}\r\n", content);
        }

        Ok(test_status)
    }

    async fn write_to_sinks(&self, test_status: TestStatus) -> Result<(), Error> {

        if !self.reporting_sinks.is_empty() {
            let arc_test_status = Arc::new(Mutex::new(test_status));
            for sink in &self.reporting_sinks {
                let status = Arc::clone(&arc_test_status);
                let cloned_sink = Arc::clone(sink);
                let t_status = status.lock().await.clone();
                cloned_sink.on_test_ended(t_status).await;
            }
        }

        Ok(())
    }
}