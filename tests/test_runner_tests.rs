use loady::core::runner::{TestRunner};
use loady::core::{TestCase,TestStep};
use loady::core::context::TestCaseContext;
use loady::core::stats::Metrics;
use std::sync::Arc;
use crate::support::*;

mod support;

#[test]
fn given_test_with_no_steps_when_running_test_then_runs_and_returns_blank_results() {
    let test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
    let runner = TestRunner::new();

    let actual = runner.run(test_case);

    assert!(actual.is_ok());    
    let test_status = actual.unwrap();    
    assert_blank_metrics(&test_status.metrics);
}

#[test]
fn given_test_with_steps_without_stages_when_running_test_then_runs_and_returns_blank_results() {
    let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
    let test_step = TestStep::<EmptyData>::as_load(TEST_STEP_1, |_: &Arc<EmptyData>| { Ok(()) }, Vec::default());
    test_case.with_step(test_step);
    let runner = TestRunner::new();
    
    let actual = runner.run(test_case);

    assert!(actual.is_ok());    
    let test_status = actual.unwrap();
    assert_blank_metrics(&test_status.metrics);
}


fn assert_blank_metrics(metrics: &Metrics) {
    assert_eq!(metrics.test_duration, 0);
    assert_eq!(metrics.mean_time, 0);
    assert_eq!(metrics.min_time, 0);
    assert_eq!(metrics.max_time, 0);
    assert_eq!(metrics.p90_time, 0);
    assert_eq!(metrics.p95_time, 0);
    assert_eq!(metrics.p99_time, 0);
    assert_eq!(metrics.positive_hits, 0);
    assert_eq!(metrics.negative_hits, 0);
    assert_eq!(metrics.all_hits, 0);
    assert_eq!(metrics.errors.len(), 0);
}