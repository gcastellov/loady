use loady::core::{TestCase, TestCaseContext};
use crate::support::*;

mod support;

#[test]
fn given_test_info_when_using_ctr_then_build_instance() {
    let data = InnerContext::default();
    let test_case = TestCase::<TestCaseContext, InnerContext>::new(TEST_NAME, TEST_SUITE, data);

    assert_eq!(test_case.test_name, TEST_NAME);
    assert_eq!(test_case.test_suite, TEST_SUITE);
}