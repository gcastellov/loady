use crate::core::composition::TestCase;
use crate::core::context::TestContext;
use crate::core::exporting::FileType;
use crate::core::reporting::{DefaultReportingSink, Reporter, ReportingSink};
use crate::core::stats::TestStatus;
use std::fmt::Debug;
use std::marker::Sync;
use std::sync::Arc;
use tokio::time::Duration;

#[derive(Default)]
pub struct TestRunner {
    reporter: Reporter,
    sinks: Vec<Arc<Box<dyn ReportingSink>>>,
}

impl<'a> TestRunner {
    pub async fn run<T, U>(
        &self,
        mut test_case: TestCase<'static, T, U>,
    ) -> Result<TestStatus, &str>
    where
        T: TestContext + 'static + Sync + Debug,
        U: 'static + Clone + Sync + Send,
    {
        let sinks = Arc::new(self.sinks.to_owned());
        let (action_handle, action_sender) = self.reporter.handle_action_ended(&sinks);
        let (step_handle, load_sender) = self.reporter.handle_load_step_ended(&sinks);
        let (internal_handle, internal_sender) = self.reporter.handle_internal_events(&sinks);

        test_case
            .run(&action_sender, &load_sender, &internal_sender)
            .await?;

        drop(action_sender);
        drop(load_sender);
        drop(internal_sender);

        _ = action_handle.await;
        _ = step_handle.await;
        _ = internal_handle.await;

        let ctx = test_case.test_context.clone().unwrap_or(T::default());
        let test_status = self.reporter.report_test_status(sinks, ctx).await;

        match test_status {
            Ok(status) => Ok(status),
            Err(_) => Err("An error occurred while exporting the test status"),
        }
    }

    pub fn with_default_reporting_sink(mut self) -> Self {
        self.sinks
            .push(Arc::new(Box::<DefaultReportingSink>::default()));
        self
    }

    pub fn with_reporting_sink<T: ReportingSink + 'static>(mut self, sink: T) -> Self {
        self.sinks.push(Arc::new(Box::new(sink)));
        self
    }

    pub fn with_default_output_files(mut self) -> Self {
        self.reporter.exporter.with_default_output_files();
        self
    }

    pub fn with_output_file(
        mut self,
        file_type: FileType,
        directory: &str,
        file_name: &str,
    ) -> Self {
        self.reporter.exporter.with_output_file(
            file_type,
            directory.to_string(),
            file_name.to_string(),
        );
        self
    }

    pub fn with_test_summary_std_out(mut self) -> Self {
        self.reporter.use_summary = true;
        self
    }

    pub fn with_reporting_frequency(mut self, seconds: u64) -> Self {
        if Reporter::DEFAULT_REPORTING_FREQUENCY.as_secs() > seconds {
            panic!(
                "Reporting frequency must be greater than the default value {}",
                Reporter::DEFAULT_REPORTING_FREQUENCY.as_secs()
            )
        }

        self.reporter.reporting_frequency = Duration::from_secs(seconds);
        self
    }
}
