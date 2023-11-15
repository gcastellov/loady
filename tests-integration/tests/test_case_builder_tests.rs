use crate::support::*;
use loady::utils::TestCaseBuilder;
use std::time::Duration;

mod support;

#[test]
fn given_test_info_when_using_builder_then_build_test_case() {
    let test_case = TestCaseBuilder::<EmptyData>::new(TEST_NAME, TEST_SUITE, &EmptyData::default())
        .with_load_step(TEST_STEP_1, Box::new(load))
        .with_stage(
            TEST_STAGE_1,
            Duration::from_secs(10),
            Duration::from_secs(1),
            1,
        )
        .with_load_step(TEST_STEP_2, Box::new(load))
        .with_stage(
            TEST_STAGE_2,
            Duration::from_secs(20),
            Duration::from_secs(1),
            10,
        )
        .build();

    assert_eq!(test_case.test_name, TEST_NAME);
    assert_eq!(test_case.test_suite, TEST_SUITE);
    assert_eq!(test_case.test_steps.len(), 2);
}

#[test]
fn given_test_info_without_steps_when_using_builder_then_build_test_case() {
    let test_case =
        TestCaseBuilder::<EmptyData>::new(TEST_NAME, TEST_SUITE, &EmptyData::default()).build();

    assert!(test_case.test_steps.is_empty());
}
