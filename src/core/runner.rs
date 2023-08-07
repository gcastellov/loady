use std::sync::{mpsc,Mutex,Arc};
use std::thread;
use std::time::{Duration,Instant};
use std::fmt::Debug;
use std::marker::Sync;
use std::io::{Error};
use crate::core::reporting::{ReportingSink,DefaultReportingSink};
use crate::core::exporting::{Exporter,FileType};
use crate::core::stats::{TestStatus,StepStatus};
use crate::core::{TestCase,TestContext};

pub struct TestRunner {
    reporting_sinks: Vec<Arc<Box<dyn ReportingSink>>>,
    exporter: Exporter,
    use_summary: bool,
    reporting_frequency: Duration
}

impl TestRunner {

    const DEFAULT_REPORTING_FREQUENCY: Duration = Duration::from_secs(5);

    pub fn new() -> Self {
        TestRunner {
            reporting_sinks: Vec::default(),
            exporter: Exporter::default(),
            use_summary: false,
            reporting_frequency: Self::DEFAULT_REPORTING_FREQUENCY
        }
    }
  
    pub fn run<'a, T, U>(&self, mut test_case: TestCase<T, U>) -> Result<TestStatus, Error>
        where T: TestContext + 'static + Sync + Debug, U: 'static + Clone + Sync + Send {

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

        let action_sinks: Vec<Arc<Box<dyn ReportingSink>>> = self.reporting_sinks.clone();
        let step_sinks: Vec<Arc<Box<dyn ReportingSink>>> = self.reporting_sinks.clone();
        let mut stats_by_step: Arc<Mutex<Vec<StepStatus>>> = Arc::new(Mutex::new(Vec::default()));
        let arc_stats_by_step = Arc::clone(&mut stats_by_step);
        let reporting_frequency = self.reporting_frequency.to_owned();
        
        let t_action_join = thread::spawn(move || { 
            let mut frequency_instant = Instant::now();
            
            while let Ok(inner_ctx) = rx_action.recv() {
                let action_sinks = action_sinks.clone();

                if !action_sinks.is_empty() && frequency_instant.elapsed() > reporting_frequency {

                    let step_status = StepStatus::new(
                        test_case.test_name.to_owned(), 
                        Box::new(inner_ctx));

                    report_step_status(true, step_status, action_sinks);
                    frequency_instant = Instant::now();                    
                }
                
                thread::sleep(Duration::from_millis(25));
            }
        });

        let t_step_join = thread::spawn(move || { 
            while let Ok(inner_ctx) = rx_step.recv() {
                let step_sinks = step_sinks.clone();
                let step_status = StepStatus::new(
                    test_case.test_name.to_owned(), 
                    Box::new(inner_ctx));

                if !step_sinks.is_empty() {
                    report_step_status(false, step_status.to_owned(), step_sinks);
                }

                arc_stats_by_step
                    .lock()
                    .unwrap()
                    .push(step_status);

                thread::sleep(Duration::from_millis(50));
            }
        });

        test_case.run(&tx_action, &tx_step);

        drop(tx_action);
        drop(tx_step);

        t_action_join.join().unwrap();
        t_step_join.join().unwrap();
        

        let by_step: Vec<StepStatus> = stats_by_step.lock().unwrap().clone();
        let test_status = self.report_test_status(&test_case, &by_step)?;
        Ok(test_status)
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
   
    fn report_test_status<T, U>(&self, test_case: &TestCase<T, U>, stats_by_step: &Vec<StepStatus>) -> Result<TestStatus, Error>
        where T: TestContext + 'static + Sync + Debug {

        let ctx = test_case.test_context.clone().unwrap();

        let test_status = TestStatus::new(
            test_case.test_name.to_owned(), 
            Box::new(ctx));

        self.write_to_sinks(test_status.to_owned())?;
        self.exporter.write_output_files(test_status.to_owned(), stats_by_step.to_owned())?;

        if self.use_summary {
            let content = FileType::Txt.get_content(test_status.to_owned(), stats_by_step.to_owned());
            println!("\r\n{}\r\n", content);
        }

        Ok(test_status)
    }

    fn write_to_sinks(&self, test_status: TestStatus) -> Result<(), Error> {

        if !self.reporting_sinks.is_empty() {

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

        Ok(())
    }
}