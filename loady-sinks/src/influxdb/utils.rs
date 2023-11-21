use super::sink::InfluxDbSink;
use influxdb::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct InfluxDbSinkBuilder {
    url: Option<String>,
    db_name: Option<String>,
    metrics_ts_name: Option<String>,
    errors_ts_name: Option<String>,
    credentials: Option<(String, String)>,
}

impl InfluxDbSinkBuilder {
    const DEFAULT_DB_NAME: &str = "loady";
    const DEFAULT_METRICS_MEASUREMENT_NAME: &str = "loady-metrics";
    const DEFAULT_ERRORS_MEASUREMENT_NAME: &str = "loady-errors";

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

    pub fn with_timeseries_names(mut self, metrics_ts_name: &str, errors_ts_name: &str) -> Self {
        self.metrics_ts_name = Some(metrics_ts_name.to_owned());
        self.errors_ts_name = Some(errors_ts_name.to_owned());
        self
    }

    pub fn build(self) -> InfluxDbSink {
        let url = self.url.expect("InfluxDb url not provided");
        let db_name = self.db_name.unwrap_or(String::from(Self::DEFAULT_DB_NAME));
        let metrics_ts_name = self
            .metrics_ts_name
            .unwrap_or(String::from(Self::DEFAULT_METRICS_MEASUREMENT_NAME));
        let errors_ts_name = self
            .errors_ts_name
            .unwrap_or(String::from(Self::DEFAULT_ERRORS_MEASUREMENT_NAME));
        let client = match self.credentials {
            Some((username, password)) => Client::new(url, db_name).with_auth(username, password),
            _ => Client::new(url, db_name),
        };

        InfluxDbSink {
            client: Arc::new(Mutex::new(client)),
            metrics_ts_name,
            errors_ts_name,
        }
    }
}
