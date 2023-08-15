use loady::core::{TestCase,TestStep,TestStepStage};
use loady::core::context::{TestCaseContext};
use crate::support::*;
use std::sync::{mpsc};
use std::time::Duration;
use std::thread;

mod support;

#[test]
fn given_test_info_when_creating_test_case_then_gets_new_instance() {
    let data = EmptyData::default();
    let test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);

    assert_eq!(test_case.test_name, TEST_NAME);
    assert_eq!(test_case.test_suite, TEST_SUITE);
}

#[test]
fn given_test_case_without_steps_when_running_then_do_nothing() {
    let data = EmptyData::default();
    let (tx_load_action, _) = mpsc::channel::<TestCaseContext>();
    let (tx_load_step, _) = mpsc::channel::<TestCaseContext>();
    let (tx_internal_step, _) = mpsc::channel::<&str>();
    let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);

    test_case.run(&tx_load_action, &tx_load_step, &tx_internal_step);

    assert!(test_case.test_context.is_none());
}

#[test]
fn given_test_case_without_load_steps_when_running_then_do_nothing() {
    let data = EmptyData::default();
    let (tx_load_action, _) = mpsc::channel::<TestCaseContext>();
    let (tx_load_step, _) = mpsc::channel::<TestCaseContext>();
    let (tx_internal_step, _) = mpsc::channel::<&str>();
    let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);
    test_case.with_step(TestStep::<EmptyData>::as_clean_up(|_|{}));
    test_case.with_step(TestStep::<EmptyData>::as_warm_up(|_|{}, Vec::default()));
    test_case.with_step(TestStep::<EmptyData>::as_init(|data|{ Ok(data.clone())}));

    test_case.run(&tx_load_action, &tx_load_step, &tx_internal_step);

    assert!(test_case.test_context.is_none());
}

#[test]
fn given_test_case_with_load_step_and_empty_stages_when_running_then_do_nothing() {
    let data = EmptyData::default();
    let (tx_load_action, _) = mpsc::channel::<TestCaseContext>();
    let (tx_load_step, _) = mpsc::channel::<TestCaseContext>();
    let (tx_internal_step, _) = mpsc::channel::<&str>();
    let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);
    test_case.with_step(TestStep::<EmptyData>::as_load("step", |_|{Ok(())}, Vec::default()));

    test_case.run(&tx_load_action, &tx_load_step, &tx_internal_step);

    assert!(test_case.test_context.is_none());
}

#[test]
fn given_test_case_with_load_step_and_stages_when_running_then_do_something() {
    let data = EmptyData::default();
    let (tx_load_action, rx_load_action) = mpsc::channel::<TestCaseContext>();
    let (tx_load_step, rx_load_step) = mpsc::channel::<TestCaseContext>();
    let (tx_internal_step, rx_internal_step) = mpsc::channel::<&str>();
    
    _ = thread::spawn(move || { 
        while let Ok(_) = rx_load_action.recv() {
            thread::sleep(Duration::from_millis(200));
        }
    });

    _ = thread::spawn(move || { 
        while let Ok(_) = rx_load_step.recv() {
            thread::sleep(Duration::from_millis(200));
        }
    });

    _ = thread::spawn(move || { 
        while let Ok(_) = rx_internal_step.recv() {
            thread::sleep(Duration::from_millis(200));
        }
    });
    
    let stages = vec![
        TestStepStage::new("first", Duration::from_secs(2), Duration::from_secs(1), 1),
        TestStepStage::new("second", Duration::from_secs(3), Duration::from_secs(1), 1)
    ];
    let load_step = TestStep::<EmptyData>::as_load("step", |_|{Ok(())}, stages);
    let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, data);
    test_case.with_step(load_step);

    test_case.run(&tx_load_action, &tx_load_step, &tx_internal_step);

    assert!(test_case.test_context.is_some());
}