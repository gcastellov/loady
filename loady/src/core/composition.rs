use tokio::time::{Instant,Duration};
use tokio::time::sleep;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::fmt::Debug;
use std::marker::Sync;
use crate::core::context::TestContext;
use crate::core::functions::*;

pub struct TestCase<'a, T: TestContext, U> {
    pub test_name: &'static str,
    pub test_suite: &'static str,
    pub test_context: Option<T>,
    pub test_steps: Vec<TestStep<'a, U>>,
    pub data: U
}

pub enum TestStep<'a, T> {
    Init { 
        action: InitFunction<'a, T>
    },
    WarmUp { 
        action: WarmUpFunction<'a, T>, 
        stages: Vec<TestStepStage> 
    },
    Load { 
        name:  &'static str, 
        stages: Vec<TestStepStage>, 
        action: LoadFunction<'a, T>
    },
    CleanUp { 
        action: CleanUpFunction<'a, T>
    }
}

pub struct TestStepStage {
    stage_name: &'static str,
    during: Duration,
    interval: Duration,
    rate: u32
}

impl<'a, T, U> TestCase<'a, T, U> 
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

    pub fn with_step(&mut self, test_step: TestStep<'a, U>) {
        
        match test_step {
            TestStep::Init { .. } => if self.has_init_step() {
                panic!("Only one Init Step can be used");
            },
            TestStep::WarmUp { .. } => if self.has_warm_up_step() {
                panic!("Only one Warm Up step can be used");
            },
            TestStep::CleanUp { .. } =>  if self.has_clean_up_step() {
                panic!("Only one Clean Up step can be used")
            },
            _ => ()
        };

        self.test_steps.push(test_step);
    }    

    pub async fn run(&mut self, tx_action: &Sender::<T>, tx_step: &Sender::<T>, tx_internal_step: &Sender<&str>) -> Result<(), &'static str> {
        
        if !self.has_load_steps() {
            return Err("No load steps have found!");
        }

        let mut owned_data = self.data.clone();
        let ctx = Arc::new(Mutex::new(T::new(self.test_name, self.test_suite)));
        let steps = self.get_ordered_test_steps();
        let start_time = Instant::now();
        
        for (_, test_step) in steps {
            match test_step {
                TestStep::Init { action } => {                    
                    if let Ok(result) = action(self.data.to_owned()).await {
                        owned_data = result;
                        _ = tx_internal_step.send(test_step.get_name()).await;
                    } else {
                        panic!("Init operation has failed");
                    }
                },
                TestStep::WarmUp { action, stages } => {
                    let data = Arc::new(owned_data.clone());

                    for test_stage in stages { 
                        let stage_start_time = Instant::now();
                        while stage_start_time.elapsed() < test_stage.during {
                            for _ in 0..test_stage.rate {
                                let t_data = Arc::clone(&data);
                                let action = action.clone();
                                action(t_data).await;
                            }

                            sleep(test_stage.interval).await;
                        }
                    }

                    _ = tx_internal_step.send(test_step.get_name()).await;
                },
                TestStep::Load { name, stages, action } => {
                    // let mut handles = Vec::default();
                    let data = Arc::new(owned_data.clone());
                    for test_stage in stages {
                        ctx
                            .lock()
                            .await
                            .set_current_step(name, test_stage.stage_name);
        
                        let stage_start_time = Instant::now();
        
                        while stage_start_time.elapsed() < test_stage.during {
        
                            for _ in 0..test_stage.rate {
                                let action_transmitter = Sender::clone(tx_action);
                                let t_ctx = Arc::clone(&ctx);
                                let t_data = Arc::clone(&data);
                                let action_start_time = Instant::now();                     
                                let action_result = action(t_data).await;
                                let mut inner_ctx = t_ctx.lock().await;
                                inner_ctx.add_hit(action_result, action_start_time.elapsed());
                                inner_ctx.set_current_duration(start_time.elapsed());
                                _ = action_transmitter.send(inner_ctx.to_owned()).await;
                            }
        
                            sleep(test_stage.interval).await;
                        }
                    }

                    let mut step_ctx = ctx.lock().await;
                    step_ctx.set_current_duration(start_time.elapsed());                   
                    _ = tx_step.send(step_ctx.to_owned()).await;
                },
                TestStep::CleanUp { action } => {
                    action(self.data.to_owned()).await;
                    _ = tx_internal_step.send(test_step.get_name()).await;
                }
            };
        }

        let mut context = ctx.lock().await;
        context.set_current_duration(start_time.elapsed());
        self.test_context = Some(context.clone());
        Ok(())
    }

    fn get_ordered_test_steps(&self) -> Vec<(usize, &TestStep<'a, U>)> {
        let mut steps = self.test_steps
            .iter()
            .map(|step|(step.get_order(), step))
            .collect::<Vec<(i32, &TestStep<U>)>>();
    
        steps.sort_by(|(a, _), (b, _)|a.cmp(&b));
        steps.iter().enumerate().map(|(index, (_, step))|(index, *step)).collect()
    }

    fn has_load_steps(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::Load { stages, .. } => stages.len() > 0,
            _ => false
        })
        .is_some()
    }

    fn has_init_step(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::Init { .. } => true,
            _ => false
        })
        .is_some()
    }

    fn has_warm_up_step(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::WarmUp { .. } => true,
            _ => false
        })
        .is_some()
    }

    fn has_clean_up_step(&self) -> bool {
        self.test_steps.iter().find(|step| match step {
            TestStep::CleanUp { .. } => true,
            _ => false
        })
        .is_some()
    }
}

impl<'a, T> TestStep<'a, T> {
    pub fn as_init(action: InitFunction<'a, T>) -> Self {
        Self::Init { action }
    }

    pub fn as_warm_up(action: WarmUpFunction<'a, T>, stages: Vec<TestStepStage>) -> Self {
        Self::WarmUp { action, stages }
    }

    pub fn as_load(name: &'static str, action: LoadFunction<'a, T>, stages: Vec<TestStepStage>) -> Self {
        Self::Load { name, action, stages }
    }

    pub fn as_clean_up(action: CleanUpFunction<'a, T>) -> Self {
        Self::CleanUp { action }
    }

    fn get_order(&self) -> i32 {
        match self {
            TestStep::Init { .. } => 1,
            TestStep::WarmUp { .. } => 2,
            TestStep::Load { .. } => 3,
            TestStep::CleanUp { .. } => 4
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TestStep::Init { .. } => "Init",
            TestStep::WarmUp { .. } => "Warm Up",
            TestStep::Load { name, .. } => name,
            TestStep::CleanUp { .. } => "Clean Up"
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
    use std::matches;

    const TEST_NAME: &str = "test name";
    const TEST_SUITE: &str = "test_suite";

    #[derive(Default,Clone)]
    struct EmptyData;

    fn init(ctx: EmptyData) -> InitResult<'static, EmptyData> {
        Box::pin(async move {
            Ok(ctx.to_owned())
        })
    }
    
    fn warmup(_ctx: Arc<EmptyData>) -> WarmUpResult<'static> {
        Box::pin(async move {            
        })
    }
    
    fn load(_ctx: Arc<EmptyData>) -> LoadResult<'static> {
        Box::pin(async move {
            Ok(())
        })
    }
    
    fn cleanup(_ctx: EmptyData) -> CleanUpResult<'static> {
        Box::pin(async move {
        })
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_init_step_when_adding_additional_init_step_then_panics() {
        let first_init_step = TestStep::<'static, EmptyData>::as_init(Box::new(init));
        let second_init_step = TestStep::<'static, EmptyData>::as_init(Box::new(init));
        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
        test_case.with_step(first_init_step);
        test_case.with_step(second_init_step);
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_clean_up_step_when_adding_additional_clean_up_step_then_panics() {
        let first_clean_up_step = TestStep::<'static, EmptyData>::as_clean_up(Box::new(cleanup));
        let second_clean_up_step = TestStep::<'static, EmptyData>::as_clean_up(Box::new(cleanup));
        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
        test_case.with_step(first_clean_up_step);
        test_case.with_step(second_clean_up_step);
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_warm_up_step_when_adding_additional_warm_up_step_then_panics() {
        let first_warm_up_step = TestStep::<'static, EmptyData>::as_warm_up(Box::new(warmup), Vec::default());
        let second_warm_up_step = TestStep::<'static, EmptyData>::as_warm_up(Box::new(warmup), Vec::default());
        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default());
        test_case.with_step(first_warm_up_step);
        test_case.with_step(second_warm_up_step);
    }

    #[test]
    fn given_test_case_with_steps_when_getting_ordered_steps_then_ensure_proper_ordering() {

        const FIRST_LOAD_STEP: &str = "first";
        const SECOND_LOAD_STEP: &str = "second";
        const THIRD_LOAD_STEP: &str = "third";

        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(TEST_NAME, TEST_SUITE, EmptyData::default()); 
        let init_step = TestStep::<'static, EmptyData>::as_init(Box::new(init));
        let clean_up_step = TestStep::<'static, EmptyData>::as_clean_up(Box::new(cleanup));
        let warm_up_step = TestStep::<'static, EmptyData>::as_warm_up(Box::new(warmup), Vec::default());
        let first_load_step = TestStep::<'static, EmptyData>::as_load(FIRST_LOAD_STEP, Box::new(load), Vec::default());
        let second_load_step = TestStep::<'static, EmptyData>::as_load(SECOND_LOAD_STEP, Box::new(load), Vec::default());
        let third_load_step = TestStep::<'static, EmptyData>::as_load(THIRD_LOAD_STEP, Box::new(load), Vec::default());

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
                TestStep::Init { .. } => 0,
                TestStep::WarmUp { .. } => 1,
                TestStep::Load { name, .. } => match *name {
                    FIRST_LOAD_STEP => 2,
                    SECOND_LOAD_STEP => 3,
                    THIRD_LOAD_STEP => 4,
                    _ => todo!()
                },
                TestStep::CleanUp { .. } => 5
            };

            assert_eq!(*index, expected_index);
        }
    }

    #[test]
    fn when_creating_new_step_as_init_then_returns_expected_type() {
        let step = TestStep::<'static, EmptyData>::as_init(Box::new(init));
        assert!(matches!(step, TestStep::Init { .. }));    
    }

    #[test]
    fn when_creating_new_step_as_warm_up_then_returns_expected_type() {
        let step = TestStep::<'static, EmptyData>::as_warm_up(Box::new(warmup), Vec::default());
        assert!(matches!(step, TestStep::WarmUp { .. }));    
    }

    #[test]
    fn when_creating_new_step_as_load_then_returns_expected_type() {
        let step = TestStep::<'static, EmptyData>::as_load("step", Box::new(load), Vec::default());
        assert!(matches!(step, TestStep::Load { .. }));    
    }

    #[test]
    fn when_creating_new_step_as_clean_up_then_returns_expected_type() {
        let step = TestStep::<'static, EmptyData>::as_clean_up(Box::new(cleanup));
        assert!(matches!(step, TestStep::CleanUp { .. }));    
    }
}