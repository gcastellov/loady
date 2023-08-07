use loady::core::runner::{TestRunner};
use loady::core::{TestCase, TestStep, TestCaseContext};
use crate::support::*;
use std::sync::Arc;

mod support;

#[test]
fn given_test_with_no_steps_when_running_test_then_runs_and_returns_blank_results() {
    let test_case = TestCase::<TestCaseContext, InnerContext>::new(TEST_NAME, TEST_SUITE, InnerContext::default());
    let runner = TestRunner::new();

    let actual = runner.run(test_case);

    assert!(actual.is_ok());    
    let test_status = actual.unwrap();    
    assert_blank_metrics(&test_status.metrics);
}

#[test]
fn given_test_with_steps_without_stages_when_running_test_then_runs_and_returns_blank_results() {
    let mut test_case = TestCase::<TestCaseContext, InnerContext>::new(TEST_NAME, TEST_SUITE, InnerContext::default());
    let test_step = TestStep::<InnerContext>::new(TEST_STEP_1, |_: &Arc<InnerContext>| { Ok(()) } );
    test_case.with_step(test_step);
    let runner = TestRunner::new();
    
    let actual = runner.run(test_case);

    assert!(actual.is_ok());    
    let test_status = actual.unwrap();
    assert_blank_metrics(&test_status.metrics);
}
