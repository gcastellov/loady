use loady::core::runner::TestRunner;
use support::Sample;

mod support;

#[tokio::main]
async fn main() {
    let test_case = Sample::build_test_case();
    let runner = TestRunner::default()
        .with_default_output_files()
        .with_test_summary_std_out();

    _ = runner.run(test_case).await;
}
