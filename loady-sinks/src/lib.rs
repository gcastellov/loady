use async_trait::async_trait;
use chrono::prelude::{DateTime, Utc};
use elasticsearch::auth::Credentials;
use elasticsearch::http::transport::Transport;
use elasticsearch::{Elasticsearch, IndexParts};
use loady::core::reporting::ReportingSink;
use loady::core::stats::*;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

#[derive(Default, Clone)]
pub struct ElasticSink {
    client: Arc<Mutex<Elasticsearch>>,
}

#[derive(Default)]
pub struct ElasticSyncBuilder {
    client: Option<Elasticsearch>,
}

#[derive(Serialize)]
struct DocumentDto {
    created_at: String,
    status: StepStatus,
}

impl DocumentDto {
    fn new(step_status: StepStatus) -> Self {
        let now: DateTime<Utc> = SystemTime::now().into();

        Self {
            created_at: now.to_rfc3339(),
            status: step_status,
        }
    }
}

impl ElasticSyncBuilder {
    pub fn with_using_url(mut self, url: &str) -> Self {
        let transport = Transport::single_node(url)
            .expect("The url provided for the Elastic sink is malformed");
        let client = Elasticsearch::new(transport);
        self.client = Some(client);
        self
    }

    pub fn with_using_cloud(mut self, cloud_id: &str, username: &str, password: &str) -> Self {
        let credentials = Credentials::Basic(username.into(), password.into());
        let transport = Transport::cloud(cloud_id, credentials)
            .expect("The provided credentials are malformed");
        let client = Elasticsearch::new(transport);
        self.client = Some(client);
        self
    }

    pub fn build(self) -> ElasticSink {
        let client = self.client.unwrap_or(Elasticsearch::default());
        ElasticSink {
            client: Arc::new(Mutex::new(client)),
        }
    }
}

impl ElasticSink {
    async fn index(&self, step_status: StepStatus) {
        let client = self.client.lock().await;
        let index_name = String::from("reporting-") + &step_status.session_id;
        let doc = DocumentDto::new(step_status);
        let response = client
            .index(IndexParts::Index(&index_name))
            .body(json!(doc))
            .send()
            .await
            .expect("Something went wrong while sending the request to ElasticSearch");

        if !response.status_code().is_success() {
            panic!("The request to ElasticSearch has failed");
        }
    }
}

#[async_trait]
impl ReportingSink for ElasticSink {
    async fn on_test_ended(&self, _: TestStatus) {}

    async fn on_load_step_ended(&self, step_status: StepStatus) {
        self.index(step_status).await;
    }

    async fn on_load_action_ended(&self, step_status: StepStatus) {
        self.index(step_status).await;
    }

    async fn on_internal_step_ended(&self, _: &str) {}
}
