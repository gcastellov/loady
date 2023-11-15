use crate::support::*;
use loady::core::composition::{TestCase, TestStep, TestStepStage};
use loady::core::context::TestCaseContext;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::time::Duration;

mod support;

#[test]
fn given_test_info_when_creating_test_case_then_gets_new_instance() {
    let data = EmptyData::default();
    let test_case =
        TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);

    assert_eq!(test_case.test_name, TEST_NAME);
    assert_eq!(test_case.test_suite, TEST_SUITE);
}

#[tokio::test]
async fn given_test_case_without_steps_when_running_then_do_nothing() {
    let data = EmptyData::default();
    let (tx_load_action, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_load_step, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_internal_step, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let mut test_case =
        TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);

    _ = test_case
        .run(&tx_load_action, &tx_load_step, &tx_internal_step)
        .await;

    assert!(test_case.test_context.is_none());
}

#[tokio::test]
async fn given_test_case_without_load_steps_when_running_then_do_nothing() {
    let data = EmptyData::default();
    let (tx_load_action, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_load_step, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_internal_step, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let mut test_case =
        TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);
    test_case.with_step(TestStep::<'static, EmptyData>::as_clean_up(Box::new(
        cleanup,
    )));
    test_case.with_step(TestStep::<'static, EmptyData>::as_warm_up(
        Box::new(warmup),
        Vec::default(),
    ));
    test_case.with_step(TestStep::<'static, EmptyData>::as_init(Box::new(init)));

    _ = test_case
        .run(&tx_load_action, &tx_load_step, &tx_internal_step)
        .await;

    assert!(test_case.test_context.is_none());
}

#[tokio::test]
async fn given_test_case_with_load_step_and_empty_stages_when_running_then_do_nothing() {
    let data = EmptyData::default();
    let (tx_load_action, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_load_step, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_internal_step, _) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let mut test_case =
        TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);
    test_case.with_step(TestStep::<'static, EmptyData>::as_load(
        "step",
        Box::new(load),
        Vec::default(),
    ));

    _ = test_case
        .run(&tx_load_action, &tx_load_step, &tx_internal_step)
        .await;

    assert!(test_case.test_context.is_none());
}

#[tokio::test]
async fn given_test_case_with_load_step_and_stages_when_running_then_do_something() {
    let data = EmptyData::default();
    let (tx_load_action, mut rx_load_action) =
        mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_load_step, mut rx_load_step) = mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);
    let (tx_internal_step, mut rx_internal_step) =
        mpsc::channel::<TestCaseContext>(CHANNEL_BUFFER_SIZE);

    _ = tokio::spawn(async move {
        while let Some(_) = rx_load_action.recv().await {
            sleep(Duration::from_millis(200)).await;
        }
    });

    _ = tokio::spawn(async move {
        while let Some(_) = rx_load_step.recv().await {
            sleep(Duration::from_millis(200)).await;
        }
    });

    _ = tokio::spawn(async move {
        while let Some(_) = rx_internal_step.recv().await {
            sleep(Duration::from_millis(200)).await;
        }
    });

    let stages = vec![
        TestStepStage::new("first", Duration::from_secs(2), Duration::from_secs(1), 1),
        TestStepStage::new("second", Duration::from_secs(3), Duration::from_secs(1), 1),
    ];

    let load_step = TestStep::<'static, EmptyData>::as_load("step", Box::new(load), stages);
    let mut test_case =
        TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);
    test_case.with_step(load_step);

    _ = test_case
        .run(&tx_load_action, &tx_load_step, &tx_internal_step)
        .await;

    assert!(test_case.test_context.is_some());
}
