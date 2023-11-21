use loady::core::runner::TestRunner;
use loady_sinks::elastic::utils::ElasticSinkBuilder;
use support::Sample;

mod support;

#[tokio::main]
async fn main() {
    let test_case = Sample::build_test_case();
    let elastic_sink = ElasticSinkBuilder::default()
        .with_using_url("http://localhost:9200")
        .build();

    let runner = TestRunner::default()
        .with_reporting_sink(elastic_sink)
        .with_test_summary_std_out();

    _ = runner.run(test_case).await;
}
