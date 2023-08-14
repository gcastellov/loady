use std::time::{Duration, SystemTime};
use std::sync::{Mutex,Arc};
use std::sync::mpsc::{Sender};
use std::fmt::Debug;
use std::marker::Sync;
use std::thread;
use crate::core::context::TestContext;

pub mod context;
pub mod stats;
pub mod reporting;
pub mod exporting;
pub mod runner;

pub struct TestCase<T: TestContext, U> {
    pub test_name: &'static str,
    pub test_suite: &'static str,
    pub test_context: Option<T>,
    pub test_steps: Vec<TestStep<U>>,
    pub data: U
}

pub enum TestStep<T> {
    Init { 
        action: fn(T) -> Result<T, i32>
    },
    WarmUp { 
        action: fn(&Arc::<T>), 
        stages: Vec<TestStepStage> 
    },
    Load { 
        name:  &'static str, 
        stages: Vec<TestStepStage>, 
        action: fn(&Arc::<T>) -> Result<(), i32> 
    },
    CleanUp { 
        action: fn(T) 
    }
}

pub struct TestStepStage {
    stage_name: &'static str,
    during: Duration,
    interval: Duration,
    rate: u32
}

impl<'a, T, U> TestCase<T, U> 
    where T: TestContext + 'static + Sync + Debug, U: 'static + Clone + Sync + Send {

    pub fn new(test_name: &'static str, test_suite: &'static str, data: U) -> Self {        
        TestCase::<T, U> {
            test_name,
            test_suite,
            test_context : None,
            test_steps: Vec::default(),
            data
        }
    }

    pub fn with_step(&mut self, test_step: TestStep<U>) {
        
        match test_step {
            TestStep::<U>::Init { .. } => if self.has_init_step() {
                panic!("Only one Init Step can be used");
            },
            TestStep::<U>::WarmUp { .. } => if self.has_warm_up_step() {
                panic!("Only one Warm Up step can be used");
            },
            TestStep::<U>::CleanUp { .. } =>  if self.has_clean_up_step() {
                panic!("Only one Clean Up step can be used")
            },
            _ => ()
        };

        self.test_steps.push(test_step);
    }    

    pub fn run(&mut self, tx_action: &Sender::<T>, tx_step: &Sender::<T>, tx_internal_step: &Sender<&str>) {
        
        if !self.has_load_steps() {
            return;
        }

        let mut owned_data = self.data.clone();
        let ctx = Arc::new(Mutex::new(T::new(self.test_name, self.test_suite)));
        let steps = self.get_ordered_test_steps();
        let start_time = SystemTime::now();
        
        for (_, test_step) in steps {
            match test_step {
                TestStep::<U>::Init { action } => {                                        
                    let result = action(self.data.to_owned()).unwrap();
                    owned_data = result;
                    tx_internal_step.send(test_step.get_name()).unwrap();
                },
                TestStep::<U>::WarmUp { action, stages } => {
                    let mut handles = Vec::default();
                    let data = Arc::new(owned_data.clone());
                    for test_stage in stages { 
                        let stage_start_time = SystemTime::now();
                        while stage_start_time.elapsed().unwrap() < test_stage.during {
                            for _ in 0..test_stage.rate {
                                let t_data = Arc::clone(&data);
                                let action = action.clone();
                                let handle = thread::spawn(move || {  
                                    action(&t_data);
                                });

                                handles.push(handle);
                            }

                            thread::sleep(test_stage.interval);
                        }
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    tx_internal_step.send(test_step.get_name()).unwrap();
                },
                TestStep::<U>::Load { name, stages, action } => {
                    let mut handles = Vec::default();
                    let data = Arc::new(owned_data.clone());
                    for test_stage in stages {
                        ctx
                            .lock()
                            .unwrap()
                            .set_current_step(name, test_stage.stage_name);
        
                        let stage_start_time = SystemTime::now();
        
                        while stage_start_time.elapsed().unwrap() < test_stage.during {
        
                            for _ in 0..test_stage.rate {
                                let action_transmitter = Sender::clone(tx_action);
                                let t_ctx = Arc::clone(&ctx);
                                let t_data = Arc::clone(&data);
                                let action = action.clone();
                    
                                let handle = thread::spawn(move || {            
                                    let action_start_time = SystemTime::now();                     
                                    let action_result = action(&t_data);
                                    let mut inner_ctx = t_ctx.lock().unwrap();
                                    inner_ctx.add_hit(action_result, action_start_time.elapsed().unwrap());
                                    inner_ctx.set_current_duration(start_time.elapsed().unwrap());
                                    action_transmitter.send(inner_ctx.to_owned()).unwrap();
                                });
                    
                                handles.push(handle);
                            }
        
                            thread::sleep(test_stage.interval);
                        }
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let mut step_ctx = ctx.lock().unwrap();
                    step_ctx.set_current_duration(start_time.elapsed().unwrap());                   
                    tx_step.send(step_ctx.to_owned()).unwrap();
                },
                TestStep::<U>::CleanUp { action } => {
                    action(self.data.to_owned());
                    tx_internal_step.send(test_step.get_name()).unwrap();
                }
            };
        }

        let mut context = ctx.lock().unwrap();
        context.set_current_duration(start_time.elapsed().unwrap());
        self.test_context = Some(context.clone());        
    }

    fn get_ordered_test_steps(&self) -> Vec<(usize, &TestStep<U>)> {
        let mut steps = self.test_steps
            .iter()
            .map(|step|(step.get_order(), step))
            .collect::<Vec<(i32, &TestStep<U>)>>();
    
        steps.sort_by(|(a, _), (b, _)|a.cmp(&b));
        steps.iter().enumerate().map(|(index, (_, step))|(index, *step)).collect()
    }

    fn has_load_steps(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::<U>::Load { stages, .. } => stages.len() > 0,
            _ => false
        })
        .is_some()
    }

    fn has_init_step(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::<U>::Init { .. } => true,
            _ => false
        })
        .is_some()
    }

    fn has_warm_up_step(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::<U>::WarmUp { .. } => true,
            _ => false
        })
        .is_some()
    }

    fn has_clean_up_step(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::<U>::CleanUp { .. } => true,
            _ => false
        })
        .is_some()
    }
}

impl<T> TestStep<T> {
    pub fn as_init(action: fn(T) -> Result<T, i32>) -> Self {
        Self::Init { action }
    }

    pub fn as_warm_up(action: fn(&Arc::<T>), stages: Vec<TestStepStage>) -> Self {
        Self::WarmUp { action, stages }
    }

    pub fn as_load(name: &'static str, action: fn(&Arc::<T>) -> Result<(), i32>, stages: Vec<TestStepStage>) -> Self {
        Self::Load { name, action, stages }
    }

    pub fn as_clean_up(action: fn(T)) -> Self {
        Self::CleanUp { action }
    }

    fn get_order(&self) -> i32 {
        match self {
            TestStep::<T>::Init { .. } => 1,
            TestStep::<T>::WarmUp { .. } => 2,
            TestStep::<T>::Load { .. } => 3,
            TestStep::<T>::CleanUp { .. } => 4
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TestStep::<T>::Init { .. } => "Init",
            TestStep::<T>::WarmUp { .. } => "Warm Up",
            TestStep::<T>::Load { name, .. } => name,
            TestStep::<T>::CleanUp { .. } => "Clean Up"
        }
    }
}

impl TestStepStage {
    pub fn new(stage_name: &'static str, during: Duration, interval: Duration, rate: u32) -> Self {
        Self { stage_name, during, interval, rate }
    }
}

#[cfg(test)]
mod tests {

    use crate::core::context::TestCaseContext;
    use super::*;

    const TEST_NAME: &str = "test name";
    const TEST_SUITE: &str = "test_suite";

    #[derive(Default,Clone)]
    struct EmptyData;

    #[test]
    #[should_panic]
    fn given_test_case_with_init_step_when_adding_additional_init_step_then_panics() {
        let first_init_step = TestStep::<EmptyData>::as_init(|data|{ Ok(data.to_owned()) });
        let second_init_step = TestStep::<EmptyData>::as_init(|data|{ Ok(data.to_owned()) });
        let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
        test_case.with_step(first_init_step);
        test_case.with_step(second_init_step);
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_clean_up_step_when_adding_additional_clean_up_step_then_panics() {
        let first_clean_up_step = TestStep::<EmptyData>::as_clean_up(|_|{ });
        let second_clean_up_step = TestStep::<EmptyData>::as_clean_up(|_|{ });
        let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
        test_case.with_step(first_clean_up_step);
        test_case.with_step(second_clean_up_step);
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_warm_up_step_when_adding_additional_warm_up_step_then_panics() {
        let first_warm_up_step = TestStep::<EmptyData>::as_warm_up(|_|{ }, Vec::default());
        let second_warm_up_step = TestStep::<EmptyData>::as_warm_up(|_|{ }, Vec::default());
        let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
        test_case.with_step(first_warm_up_step);
        test_case.with_step(second_warm_up_step);
    }

    #[test]
    fn given_test_case_with_steps_when_getting_ordered_steps_then_ensure_proper_ordering() {

        const FIRST_LOAD_STEP: &str = "first";
        const SECOND_LOAD_STEP: &str = "second";
        const THIRD_LOAD_STEP: &str = "third";

        let mut test_case = TestCase::<TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default()); 
        let init_step = TestStep::<EmptyData>::as_init(|data|{ Ok(data.to_owned()) });
        let clean_up_step = TestStep::<EmptyData>::as_clean_up(|_|{ });
        let warm_up_step = TestStep::<EmptyData>::as_warm_up(|_|{ }, Vec::default());
        let first_load_step = TestStep::<EmptyData>::as_load(FIRST_LOAD_STEP, |_| { Ok(()) }, Vec::default());
        let second_load_step = TestStep::<EmptyData>::as_load(SECOND_LOAD_STEP, |_| { Ok(()) }, Vec::default());
        let third_load_step = TestStep::<EmptyData>::as_load(THIRD_LOAD_STEP, |_| { Ok(()) }, Vec::default());

        test_case.with_step(clean_up_step);
        test_case.with_step(warm_up_step);
        test_case.with_step(init_step);
        test_case.with_step(first_load_step);
        test_case.with_step(second_load_step);
        test_case.with_step(third_load_step);

        let actual = test_case.get_ordered_test_steps();

        assert_eq!(test_case.test_steps.len(), 6);
        assert_eq!(actual.len(), test_case.test_steps.len());

        for i in 0..actual.len() {
            let (index, step) = actual.get(i).unwrap();

            let expected_index = match step {
                TestStep::<EmptyData>::Init { .. } => 0,
                TestStep::<EmptyData>::WarmUp { .. } => 1,
                TestStep::<EmptyData>::Load { name, .. } => match *name {
                    FIRST_LOAD_STEP => 2,
                    SECOND_LOAD_STEP => 3,
                    THIRD_LOAD_STEP => 4,
                    _ => todo!()
                },
                TestStep::<EmptyData>::CleanUp { .. } => 5
            };

            assert_eq!(*index, expected_index);
        }
    }
}