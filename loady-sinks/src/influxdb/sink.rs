use async_trait::async_trait;
use chrono::{DateTime, Utc};
use influxdb::Client;
use influxdb::InfluxDbWriteable;
use influxdb::Timestamp;
use loady::core::reporting::ReportingSink;
use loady::core::stats::{TestStatus, StepStatus};
use std::time::SystemTime;
use std::sync::Arc;
use tokio::sync::Mutex;


#[derive(InfluxDbWriteable)]
struct Metric {
    time: DateTime<Utc>,
    mesurement: i64,
    #[influxdb(tag)]description: String,
    #[influxdb(tag)]session_id: String,
    #[influxdb(tag)]test_name: String,
    #[influxdb(tag)]step_name: String,
}

struct MetricsWrapper {
    created_at: DateTime<Utc>,
    step_status: StepStatus 
}

impl Metric {
    fn new(time: DateTime<Utc>, session_id: &str, step_name: &str, test_name: &str, description: &str, mesurement: u128) -> Self {
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
impl MetricsWrapper {
    fn new(step_status: StepStatus) -> Self {
        Self {
            created_at: SystemTime::now().into(),
            step_status
        }
    }

    fn to_entries(&self, query_name: &str) -> Vec<influxdb::WriteQuery> {

        let session_id = &self.step_status.session_id;
        let step_name = &self.step_status.step_name;
        let test_name = &self.step_status.test_name;

        let metrics = vec![
            Metric::new(self.created_at, session_id, step_name, test_name, "test_duration", self.step_status.metrics.test_duration),
            Metric::new(self.created_at, session_id, step_name, test_name, "mean_time", self.step_status.metrics.mean_time),
            Metric::new(self.created_at, session_id, step_name, test_name, "max_time", self.step_status.metrics.max_time),
            Metric::new(self.created_at, session_id, step_name, test_name, "min_time", self.step_status.metrics.min_time),
            Metric::new(self.created_at, session_id, step_name, test_name, "std_dev", self.step_status.metrics.std_dev),
            Metric::new(self.created_at, session_id, step_name, test_name, "p90_time", self.step_status.metrics.p90_time),
            Metric::new(self.created_at, session_id, step_name, test_name, "p95_time", self.step_status.metrics.p95_time),
            Metric::new(self.created_at, session_id, step_name, test_name, "p99_time", self.step_status.metrics.p99_time),
            Metric::new(self.created_at, session_id, step_name, test_name, "positive_hits", self.step_status.metrics.positive_hits),
            Metric::new(self.created_at, session_id, step_name, test_name, "negative_hits", self.step_status.metrics.negative_hits),
            Metric::new(self.created_at, session_id, step_name, test_name, "all_hits", self.step_status.metrics.all_hits),
        ];

        metrics.into_iter()
            .map(|metric|metric.into_query(query_name))
            .collect::<Vec<influxdb::WriteQuery>>()
    }
}

#[derive(Clone)]
pub struct InfluxDbSink {
    pub client: Arc<Mutex<Client>>,
    pub timeseries_name: String
}

impl InfluxDbSink {
    async fn insert(&self, metrics: &Vec<influxdb::WriteQuery>) {
        let client = self.client.lock().await;
        client
            .query(metrics)
            .await
            .expect("The request to InfluxDb has failed");
    }
}

#[async_trait]
impl ReportingSink for InfluxDbSink {
    async fn on_test_ended(&self, _: TestStatus) {
    }

    async fn on_load_step_ended(&self, step_status: StepStatus) {
        let metrics = MetricsWrapper::new(step_status).to_entries(self.timeseries_name.as_str());
        self.insert(&metrics).await;        
    }

    async fn on_load_action_ended(&self, step_status: StepStatus) {
        let metrics = MetricsWrapper::new(step_status).to_entries(self.timeseries_name.as_str());
        self.insert(&metrics).await;
    }

    async fn on_internal_step_ended(&self, _: &str) {
    }
}