use loady::core::composition::{TestCase,TestStep,TestStepStage};
use loady::core::context::TestCaseContext;
use loady::core::runner::TestRunner;
use loady::core::stats::Metrics;
use tokio::time::Duration;
use crate::support::*;

mod support;

#[tokio::test]
async fn given_test_with_no_steps_when_running_test_then_returns_error() {
    let test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
    let runner = TestRunner::new();

    let actual = runner.run(test_case).await;

    assert!(actual.is_err());  
}

#[tokio::test]
async fn given_test_with_steps_without_stages_when_running_test_then_returns_error() {
    let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
    let test_step = TestStep::<'static, EmptyData>::as_load(TEST_STEP_1, Box::new(load), Vec::default());
    test_case.with_step(test_step);
    let runner = TestRunner::new();
    
    let actual = runner.run(test_case).await;

    assert!(actual.is_err());    
}

#[tokio::test]
async fn given_test_with_steps_when_running_test_then_returns_metrics() {
    let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
    let stages = vec![
        TestStepStage::new(TEST_STAGE_1, Duration::from_secs(2), Duration::from_millis(500), 2),
        TestStepStage::new(TEST_STAGE_2, Duration::from_secs(3), Duration::from_millis(500), 3)
    ];
        
    let test_step = TestStep::<'static, EmptyData>::as_load(TEST_STEP_1, Box::new(load), stages);
    test_case.with_step(test_step);
    let runner = TestRunner::new();

    let actual = runner.run(test_case).await;

    assert!(actual.is_ok());

    let test_status = actual.unwrap();
    assert_not_blank_metrics(&test_status.metrics);
}


fn assert_not_blank_metrics(metrics: &Metrics) {
    assert!(metrics.test_duration > 0);
    assert!(metrics.mean_time > 0);
    assert!(metrics.min_time > 0);
    assert!(metrics.max_time > 0);
    assert!(metrics.std_dev > 0);
    assert!(metrics.p90_time > 0);
    assert!(metrics.p95_time > 0);
    assert!(metrics.p99_time > 0);
    assert!(metrics.positive_hits > 0);
    assert!(metrics.negative_hits > 0);
    assert!(metrics.all_hits > 0);
    assert!(metrics.errors.len() > 0);
}