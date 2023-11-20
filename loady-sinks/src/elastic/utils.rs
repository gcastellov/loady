use super::sink::ElasticSink;
use elasticsearch::{auth::Credentials, http::transport::Transport, Elasticsearch};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct ElasticSinkBuilder {
    client: Option<Elasticsearch>,
}

impl ElasticSinkBuilder {
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
