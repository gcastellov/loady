use loady::core::runner::TestRunner;
use loady_sinks::influxdb::utils::InfluxDbSinkBuilder;
use support::Sample;

mod support;

#[tokio::main]
async fn main() {
    let test_case = Sample::build_test_case();
    let influxdb_sink = InfluxDbSinkBuilder::default()
        .with_using_url("http://localhost:8086")
        .with_credentials("influx", "influxdb")
        .with_db_name("db0")
        .with_timeseries_name("step-metics")
        .build();

    let runner = TestRunner::default()
        .with_reporting_sink(influxdb_sink)
        .with_test_summary_std_out();

    _ = runner.run(test_case).await;
}
