use std::sync::{mpsc,Mutex,Arc};
use std::thread;
use std::time::{Duration};
use std::fmt::Debug;
use std::marker::Sync;
use std::fs::File;
use std::io::{Write,Error};
use crate::core::reporting::{ReportingSink,DefaultReportingSink,TestStatus,StepStatus};
use crate::core::{TestCase,TestContext};

#[derive(Default)]
pub struct TestRunner {
    reporting_sinks: Vec<Arc<Box<dyn ReportingSink>>>,
    output_dir: Option<String>,
    output_file: Option<String>
}

impl TestRunner {

    const SESSION_ID_PATTERN: &str = "{session-id}";
   
    pub fn run<T>(&self, mut test_case: TestCase<T>) -> Result<(), Error>
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
        let mut stats_by_step: Arc<Mutex<Vec<StepStatus>>> = Arc::new(Mutex::new(Vec::default()));
        let arc_stats_by_step = Arc::clone(&mut stats_by_step);
        
        let t_action_join = thread::spawn(move || { 
            while let Ok(inner_ctx) = rx_action.recv() {
                let action_sinks = action_sinks.clone();
                if action_sinks.is_empty() {
                    continue;
                }

                let step_status = StepStatus::new(
                    test_case.test_name.to_owned(), 
                    Box::new(inner_ctx));

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
                    test_case.test_name.to_owned(), 
                    Box::new(inner_ctx));

                report_step_status(false, step_status.to_owned(), step_sinks);

                arc_stats_by_step
                    .lock()
                    .unwrap()
                    .push(step_status.to_owned());

                thread::sleep(Duration::from_millis(50));
            }
        });

        test_case.run(&tx_action, &tx_step);

        drop(tx_action);
        drop(tx_step);

        t_action_join.join().unwrap();
        t_step_join.join().unwrap();
        

        let by_step: Vec<StepStatus> = stats_by_step.lock().unwrap().clone();
        self.report_test_status(&test_case, &by_step)?;
        Ok(())
    }

    pub fn with_default_reporting_sink(&mut self) {
        let sink = DefaultReportingSink::default();
        self.with_reporting_sink(sink);
    }

    pub fn with_reporting_sink<T: ReportingSink + 'static>(&mut self, sink: T) {
        self.reporting_sinks.push(Arc::new(Box::new(sink)));
    }

    pub fn with_default_output_file(&mut self) {
        self.output_dir = Some(String::from("output"));
        self.output_file = Some(String::from(Self::SESSION_ID_PATTERN.to_owned() + ".txt"));
    }
   
    fn report_test_status<T>(&self, test_case: &TestCase<T>, stats_by_step: &Vec<StepStatus>) -> Result<(), Error>
        where T: TestContext + 'static + Sync + Debug {

        let ctx = test_case.test_context.unwrap();

        let test_status = TestStatus::new(
            test_case.test_name.to_owned(), 
            Box::new(ctx));

        self.write_to_sinks(test_status.to_owned())?;
        self.write_output_file(test_status.to_owned(), stats_by_step.to_owned())?;        

        Ok(())
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

    fn write_output_file(&self, test_status: TestStatus, step_status: Vec<StepStatus>) -> Result<(), Error> {

        const STEP_SEPARATOR: &str = "\r\n\r\n----------------------------------------------------------------------\r\n\r\n";

        if let Some(mut file_name) = self.output_file.clone() {            
            let directory = self.output_dir.clone().unwrap();
            std::fs::create_dir_all(directory.to_owned())?;           
            file_name = file_name.as_str().replace(Self::SESSION_ID_PATTERN, test_status.session_id.as_str());
            file_name = format!("{}/{}", directory, file_name);
            let mut file = File::create(file_name)?;
            let content: String = step_status
                .iter()
                .fold(format!("{}", test_status), |cur, nxt| cur + format!("{}{}", STEP_SEPARATOR, nxt).as_str());

            file.write_all(content.as_bytes())?;           
        }

        Ok(())
    }
}