use crate::core::context::TestContext;
use crate::core::exporting::{Exporter, FileType, Localization};
use crate::core::stats::{StepStatus, TestStatus};
use async_trait::async_trait;
use std::fmt::Debug;
use std::io::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{Duration, Instant};

pub struct Reporter {
    pub exporter: Exporter,
    pub use_summary: bool,
    pub reporting_frequency: Duration,
    stats_by_steps: Arc<Mutex<Vec<StepStatus>>>,
}

#[derive(Default, Clone)]
pub struct DefaultReportingSink;

#[async_trait]
pub trait ReportingSink: Sync + Send {
    async fn on_test_ended(&self, status: TestStatus);
    async fn on_load_step_ended(&self, status: StepStatus);
    async fn on_load_action_ended(&self, step_status: StepStatus);
    async fn on_internal_step_ended(&self, step_name: &str);
}

#[async_trait]
impl ReportingSink for DefaultReportingSink {
    async fn on_test_ended(&self, test_status: TestStatus) {
        let locale = Localization::default();
        print!("\x1B[2J\x1B[1;1H");
        println!("{}", test_status.as_txt(&locale));
    }

    async fn on_load_step_ended(&self, step_status: StepStatus) {
        let locale = Localization::default();
        print!("\x1B[2J\x1B[1;1H");
        println!("{}", step_status.as_txt(&locale));
    }

    async fn on_load_action_ended(&self, step_status: StepStatus) {
        let locale = Localization::default();
        print!("\x1B[2J\x1B[1;1H");
        println!("{}", step_status.as_txt(&locale));
    }

    async fn on_internal_step_ended(&self, _step_name: &str) {}
}

impl Default for Reporter {
    fn default() -> Self {
        Self {
            exporter: Default::default(),
            use_summary: Default::default(),
            reporting_frequency: Self::DEFAULT_REPORTING_FREQUENCY,
            stats_by_steps: Arc::new(Mutex::new(Vec::default())),
        }
    }
}

impl Reporter {
    pub const DEFAULT_REPORTING_FREQUENCY: Duration = Duration::from_secs(5);

    pub fn handle_action_ended<T>(
        &self,
        sinks: &Arc<Vec<Arc<Box<dyn ReportingSink>>>>,
    ) -> (tokio::task::JoinHandle<()>, mpsc::Sender<T>)
    where
        T: TestContext + 'static + Sync + Debug,
    {
        let (sender, mut receiver) = mpsc::channel::<T>(10);
        let sinks = Arc::clone(sinks);
        let reporting_frequency = self.reporting_frequency;

        let t_action_join = tokio::spawn(async move {
            let mut frequency_instant = Instant::now();
            while let Some(inner_ctx) = receiver.recv().await {
                let sinks = Arc::clone(&sinks);
                if !sinks.is_empty() && frequency_instant.elapsed() > reporting_frequency {
                    let step_status = StepStatus::new(inner_ctx.get_test_name(), inner_ctx);

                    for sink in sinks.as_ref() {
                        sink.on_load_action_ended(step_status.to_owned()).await;
                    }

                    frequency_instant = Instant::now();
                }
            }
        });

        (t_action_join, sender)
    }

    pub fn handle_load_step_ended<T>(
        &self,
        sinks: &Arc<Vec<Arc<Box<dyn ReportingSink>>>>,
    ) -> (tokio::task::JoinHandle<()>, mpsc::Sender<T>)
    where
        T: TestContext + 'static + Sync + Debug,
    {
        let (sender, mut receiver) = mpsc::channel::<T>(10);
        let sinks = Arc::clone(sinks);
        let stats_by_step = Arc::clone(&self.stats_by_steps);

        let t_step_join = tokio::spawn(async move {
            while let Some(inner_ctx) = receiver.recv().await {
                let sinks = Arc::clone(&sinks);
                let step_status = StepStatus::new(inner_ctx.get_test_name(), inner_ctx);

                if !sinks.is_empty() {
                    for sink in sinks.as_ref() {
                        sink.on_load_step_ended(step_status.to_owned()).await;
                    }
                }

                stats_by_step.lock().await.push(step_status);
            }
        });

        (t_step_join, sender)
    }

    pub fn handle_internal_events<T>(
        &self,
        sinks: &Arc<Vec<Arc<Box<dyn ReportingSink>>>>,
    ) -> (tokio::task::JoinHandle<()>, mpsc::Sender<T>)
    where
        T: TestContext + 'static + Sync + Debug,
    {
        let (sender, mut receiver) = mpsc::channel::<T>(10);
        let sinks = Arc::clone(sinks);

        let t_internal_step_join = tokio::spawn(async move {
            while let Some(inner_ctx) = receiver.recv().await {
                let step_name = &inner_ctx.get_current_step_name();
                if !sinks.is_empty() {
                    for sink in sinks.as_ref() {
                        let sink = Arc::clone(sink);
                        sink.on_internal_step_ended(step_name).await;
                    }
                }
            }
        });

        (t_internal_step_join, sender)
    }

    pub async fn report_test_status<T>(
        &self,
        sinks: Arc<Vec<Arc<Box<dyn ReportingSink>>>>,
        ctx: T,
    ) -> Result<TestStatus, Error>
    where
        T: TestContext + 'static + Sync + Debug,
    {
        let test_status = TestStatus::new(ctx.get_test_name(), ctx);
        let stats_by_step = self.stats_by_steps.lock().await.to_vec();

        Self::write_to_sinks(sinks, test_status.to_owned()).await?;
        self.exporter
            .write_output_files(test_status.to_owned(), stats_by_step.to_owned())?;

        if self.use_summary {
            let content =
                FileType::Txt.get_content(test_status.to_owned(), stats_by_step.to_owned());

            print!("\x1B[2J\x1B[1;1H");
            println!("\r\n{}\r\n", content);
        }

        Ok(test_status)
    }

    async fn write_to_sinks(
        sinks: Arc<Vec<Arc<Box<dyn ReportingSink>>>>,
        test_status: TestStatus,
    ) -> Result<(), Error> {
        if !sinks.is_empty() {
            let arc_test_status = Arc::new(Mutex::new(test_status));
            for sink in sinks.as_ref() {
                let status = Arc::clone(&arc_test_status);
                let cloned_sink = Arc::clone(sink);
                let t_status = status.lock().await.clone();
                cloned_sink.on_test_ended(t_status).await;
            }
        }

        Ok(())
    }
}
