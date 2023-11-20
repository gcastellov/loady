use std::sync::Arc;
use tokio::sync::Mutex;
use influxdb::Client;
use super::sink::InfluxDbSink;

#[derive(Default)]
pub struct InfluxDbSinkBuilder {
    url: Option<String>,
    db_name: Option<String>,
    timeseries_name: Option<String>,
    credentials: Option<(String, String)>
}

impl InfluxDbSinkBuilder {
    pub fn with_using_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_owned());
        self
    }

    pub fn with_credentials(mut self, username: &str, password: &str) -> Self {
        self.credentials = Some((username.to_owned(), password.to_owned()));
        self
    }

    pub fn with_db_name(mut self, db_name: &str) -> Self {
        self.db_name = Some(db_name.to_owned());
        self
    }

    pub fn with_timeseries_name(mut self, timeseries_name: &str) -> Self {
        self.timeseries_name = Some(timeseries_name.to_owned());
        self
    }

    pub fn build(self) -> InfluxDbSink {
        let url = self.url.expect("InfluxDb url not provided");
        let db_name = self.db_name.unwrap_or(String::from("loady-metrics"));
        let ts_name = self.timeseries_name.unwrap_or(String::from("metrics"));        
        let client = match self.credentials {
            Some((username, password)) => Client::new(url, db_name).with_auth(username, password),
            _ => Client::new(url, db_name)
        };

        InfluxDbSink {
            client: Arc::new(Mutex::new(client)),
            timeseries_name: ts_name,
        }
    }
}

