use async_trait::async_trait;
use chrono::{DateTime, Utc};
use influxdb::Client;
use influxdb::InfluxDbWriteable;
use loady::core::reporting::ReportingSink;
use loady::core::stats::{StepStatus, TestStatus};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct InfluxDbSink {
    pub client: Arc<Mutex<Client>>,
    pub metrics_ts_name: String,
    pub errors_ts_name: String,
}

#[derive(InfluxDbWriteable)]
struct Metric {
    time: DateTime<Utc>,
    mesurement: i64,
    #[influxdb(tag)]
    description: String,
    #[influxdb(tag)]
    session_id: String,
    #[influxdb(tag)]
    test_name: String,
    #[influxdb(tag)]
    step_name: String,
}

trait MeticConverter {
    fn to_metrics(&self, query_name: &str, created_at: DateTime<Utc>) -> Vec<influxdb::WriteQuery>;
    fn to_errors(&self, query_name: &str, created_at: DateTime<Utc>) -> Vec<influxdb::WriteQuery>;
}

impl Metric {
    fn new(
        time: DateTime<Utc>,
        session_id: &str,
        step_name: &str,
        test_name: &str,
        description: &str,
        mesurement: u128,
    ) -> Self {
        Metric {
            time,
            session_id: session_id.to_owned(),
            step_name: step_name.to_owned(),
            test_name: test_name.to_owned(),
            description: description.to_owned(),
            mesurement: mesurement as i64,
        }
    }
}

impl MeticConverter for StepStatus {
    fn to_metrics(&self, query_name: &str, created_at: DateTime<Utc>) -> Vec<influxdb::WriteQuery> {
        let session_id = &self.session_id;
        let step_name = &self.step_name;
        let test_name = &self.test_name;

        let metrics = vec![
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "test_duration",
                self.metrics.test_duration,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "mean_time",
                self.metrics.mean_time,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "max_time",
                self.metrics.max_time,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "min_time",
                self.metrics.min_time,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "std_dev",
                self.metrics.std_dev,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "p90_time",
                self.metrics.p90_time,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "p95_time",
                self.metrics.p95_time,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "p99_time",
                self.metrics.p99_time,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "positive_hits",
                self.metrics.positive_hits,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "negative_hits",
                self.metrics.negative_hits,
            ),
            Metric::new(
                created_at,
                session_id,
                step_name,
                test_name,
                "all_hits",
                self.metrics.all_hits,
            ),
        ];

        metrics
            .into_iter()
            .map(|metric| metric.into_query(query_name))
            .collect::<Vec<influxdb::WriteQuery>>()
    }

    fn to_errors(&self, query_name: &str, created_at: DateTime<Utc>) -> Vec<influxdb::WriteQuery> {
        let session_id = &self.session_id;
        let step_name = &self.step_name;
        let test_name = &self.test_name;

        self.metrics
            .errors
            .iter()
            .map(|(key, value)| {
                Metric::new(
                    created_at,
                    session_id,
                    step_name,
                    test_name,
                    &key.to_string(),
                    *value,
                )
            })
            .map(|metric| metric.into_query(query_name))
            .collect()
    }
}

impl InfluxDbSink {
    async fn insert(&self, metrics: &Vec<influxdb::WriteQuery>) {
        let client = self.client.lock().await;
        client
            .query(metrics)
            .await
            .expect("The request to InfluxDb has failed");
    }

    async fn track_metrics(&self, step_status: &StepStatus) {
        let created_at = SystemTime::now().into();
        let metrics = step_status.to_metrics(self.metrics_ts_name.as_str(), created_at);
        let errors = step_status.to_errors(self.errors_ts_name.as_str(), created_at);
        self.insert(&metrics).await;
        self.insert(&errors).await;
    }
}

#[async_trait]
impl ReportingSink for InfluxDbSink {
    async fn on_test_ended(&self, _: TestStatus) {}

    async fn on_load_step_ended(&self, step_status: StepStatus) {
        self.track_metrics(&step_status).await;
    }

    async fn on_load_action_ended(&self, step_status: StepStatus) {
        self.track_metrics(&step_status).await;
    }

    async fn on_internal_step_ended(&self, _: &str) {}
}
