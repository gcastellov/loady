use crate::core::context::TestContext;
use crate::core::functions::*;
use std::fmt::Debug;
use std::marker::Sync;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::time::{Duration, Instant};

pub struct TestCase<'a, T: TestContext, U> {
    pub test_name: &'static str,
    pub test_suite: &'static str,
    pub test_context: Option<T>,
    pub test_steps: Vec<TestStep<'a, U>>,
    pub data: U,
}

pub enum TestStep<'a, T> {
    Init {
        action: Option<InitFunction<'a, T>>,
    },
    WarmUp {
        action: Option<WarmUpFunction<'a, T>>,
        stages: Vec<TestStepStage>,
    },
    Load {
        name: &'static str,
        stages: Vec<TestStepStage>,
        action: Option<LoadFunction<'a, T>>,
    },
    CleanUp {
        action: Option<CleanUpFunction<'a, T>>,
    },
}

pub struct TestStepStage {
    stage_name: &'static str,
    during: Duration,
    interval: Duration,
    rate: u32,
}

impl<'a, T, U> TestCase<'static, T, U>
where
    T: TestContext + 'static + Sync + Debug,
    U: 'static + Clone + Sync + Send,
{
    pub fn new(test_name: &'static str, test_suite: &'static str, data: U) -> Self {
        TestCase::<T, U> {
            test_name,
            test_suite,
            test_context: None,
            test_steps: Vec::default(),
            data,
        }
    }

    pub fn with_step(&mut self, test_step: TestStep<'static, U>) {
        match test_step {
            TestStep::Init { .. } => {
                if self.has_init_step() {
                    panic!("Only one Init Step can be used");
                }
            }
            TestStep::WarmUp { .. } => {
                if self.has_warm_up_step() {
                    panic!("Only one Warm Up step can be used");
                }
            }
            TestStep::CleanUp { .. } => {
                if self.has_clean_up_step() {
                    panic!("Only one Clean Up step can be used")
                }
            }
            _ => (),
        };

        self.test_steps.push(test_step);
        self.test_steps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    pub async fn run(
        &mut self,
        tx_action: &Sender<T>,
        tx_step: &Sender<T>,
        tx_internal_step: &Sender<T>,
    ) -> Result<(), &'static str> {
        if !self.has_load_steps() {
            return Err("No load steps have found!");
        }

        let mut data = self.data.clone();
        let ctx = Arc::new(Mutex::new(T::new(self.test_name, self.test_suite)));
        let mut load_start_time: Option<Instant> = None;

        for test_step in &mut self.test_steps {
            {
                let mut ctx = ctx.lock().await;
                ctx.set_current_step(test_step.get_name());
            }

            match test_step {
                TestStep::Init { action } => {
                    let action = action.take().unwrap();
                    data = Self::execute_init(action, data.to_owned()).await;
                    let ctx = ctx.lock().await;
                    _ = tx_internal_step.send(ctx.to_owned()).await;
                }
                TestStep::WarmUp { action, stages } => {
                    let action = action.take().unwrap();
                    Self::execute_warmup(action, data.to_owned(), stages).await;
                    let ctx = ctx.lock().await;
                    _ = tx_internal_step.send(ctx.to_owned()).await;
                }
                TestStep::Load { stages, action, .. } => {
                    let load_start_time = load_start_time.get_or_insert(Instant::now());
                    let action = action.take().unwrap();
                    Self::execute_load(
                        action,
                        data.to_owned(),
                        stages,
                        &ctx,
                        tx_action,
                        load_start_time.to_owned(),
                    )
                    .await;
                    let ctx = ctx.lock().await;
                    _ = tx_step.send(ctx.to_owned()).await;
                }
                TestStep::CleanUp { action } => {
                    let action = action.take().unwrap();
                    Self::execute_cleanup(action, data.to_owned()).await;
                    let step_ctx = ctx.lock().await;
                    _ = tx_internal_step.send(step_ctx.to_owned()).await;
                }
            };
        }

        let ctx = ctx.lock().await;
        self.test_context = Some(ctx.to_owned());
        Ok(())
    }

    async fn execute_init(callback: InitFunction<'static, U>, data: U) -> U {
        callback(data).await.expect("Init operation has failed")
    }

    async fn execute_cleanup(callback: CleanUpFunction<'static, U>, data: U) {
        callback(data).await
    }

    async fn execute_warmup(
        callback: WarmUpFunction<'static, U>,
        data: U,
        stages: &Vec<TestStepStage>,
    ) {
        let data = Arc::new(data);
        let callback = Arc::new(callback);
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for test_stage in stages {
            let stage_start_time = Instant::now();
            let mut next_period = stage_start_time;

            while stage_start_time.elapsed() <= test_stage.during {
                for _ in 0..test_stage.rate {
                    let data = Arc::clone(&data);
                    let callback = Arc::clone(&callback);

                    let handle = tokio::spawn(async move {
                        (callback)(data).await;
                    });

                    handles.push(handle);
                }

                next_period =
                    Self::sleep_for(&stage_start_time, &next_period, &test_stage.interval).await;
            }
        }

        for handle in handles {
            _ = handle.await;
        }
    }

    async fn execute_load(
        callback: LoadFunction<'static, U>,
        data: U,
        stages: &Vec<TestStepStage>,
        ctx: &Arc<Mutex<T>>,
        tx_action: &Sender<T>,
        load_start_time: Instant,
    ) {
        let data = Arc::new(data);
        let callback = Arc::new(callback);
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for test_stage in stages {
            ctx.lock().await.set_current_stage(test_stage.stage_name);

            let stage_start_time = Instant::now();
            let mut next_period = stage_start_time;

            while stage_start_time.elapsed() < test_stage.during {
                for _ in 0..test_stage.rate {
                    let action_transmitter = Sender::clone(tx_action);
                    let ctx = Arc::clone(ctx);
                    let data = Arc::clone(&data);
                    let callback = Arc::clone(&callback);

                    let handle = tokio::spawn(async move {
                        let action_start_time = Instant::now();
                        let action_result = callback(data).await;
                        let mut ctx = ctx.lock().await;
                        ctx.add_hit(action_result, action_start_time.elapsed());
                        ctx.set_current_load_duration(load_start_time.elapsed());
                        _ = action_transmitter.send(ctx.to_owned()).await;
                    });

                    handles.push(handle);
                }

                next_period =
                    Self::sleep_for(&stage_start_time, &next_period, &test_stage.interval).await;
            }
        }

        for handle in handles {
            _ = handle.await;
        }
    }

    fn has_load_steps(&self) -> bool {
        self.test_steps.iter().any(|step| match step {
            TestStep::Load { stages, .. } => !stages.is_empty(),
            _ => false,
        })
    }

    fn has_init_step(&self) -> bool {
        self.test_steps
            .iter()
            .any(|step| matches!(step, TestStep::Init { .. }))
    }

    fn has_warm_up_step(&self) -> bool {
        self.test_steps
            .iter()
            .any(|step| matches!(step, TestStep::WarmUp { .. }))
    }

    fn has_clean_up_step(&self) -> bool {
        self.test_steps
            .iter()
            .any(|step| matches!(step, TestStep::CleanUp { .. }))
    }

    async fn sleep_for(
        stage_start_time: &Instant,
        next_period: &Instant,
        interval: &Duration,
    ) -> Instant {
        let next_period = next_period
            .checked_add(*interval)
            .unwrap_or(*stage_start_time);
        if let Some(time) = next_period.checked_duration_since(Instant::now()) {
            sleep(time).await;
        }

        next_period
    }
}

impl<'a, T> PartialEq for TestStep<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_order() == other.get_order()
    }
}

impl<'a, T> PartialOrd for TestStep<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let current_order = self.get_order();
        let other_order = other.get_order();
        Some(current_order.cmp(&other_order))
    }
}

impl<'a, T> TestStep<'a, T> {
    pub fn as_init(action: InitFunction<'a, T>) -> Self {
        Self::Init {
            action: Some(action),
        }
    }

    pub fn as_warm_up(action: WarmUpFunction<'a, T>, stages: Vec<TestStepStage>) -> Self {
        Self::WarmUp {
            action: Some(action),
            stages,
        }
    }

    pub fn as_load(
        name: &'static str,
        action: LoadFunction<'a, T>,
        stages: Vec<TestStepStage>,
    ) -> Self {
        Self::Load {
            name,
            action: Some(action),
            stages,
        }
    }

    pub fn as_clean_up(action: CleanUpFunction<'a, T>) -> Self {
        Self::CleanUp {
            action: Some(action),
        }
    }

    fn get_order(&self) -> usize {
        match self {
            TestStep::Init { .. } => 0,
            TestStep::WarmUp { .. } => 1,
            TestStep::Load { .. } => 2,
            TestStep::CleanUp { .. } => 3,
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TestStep::Init { .. } => "Init",
            TestStep::WarmUp { .. } => "Warm Up",
            TestStep::Load { name, .. } => name,
            TestStep::CleanUp { .. } => "Clean Up",
        }
    }
}

impl TestStepStage {
    pub fn new(stage_name: &'static str, during: Duration, interval: Duration, rate: u32) -> Self {
        Self {
            stage_name,
            during,
            interval,
            rate,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::context::TestCaseContext;
    use std::matches;

    const TEST_NAME: &str = "test name";
    const TEST_SUITE: &str = "test_suite";

    #[derive(Default, Clone)]
    struct EmptyData;

    fn init(ctx: EmptyData) -> InitResult<'static, EmptyData> {
        Box::pin(async move { Ok(ctx.to_owned()) })
    }

    fn warmup(_ctx: Arc<EmptyData>) -> WarmUpResult<'static> {
        Box::pin(async move {})
    }

    fn load(_ctx: Arc<EmptyData>) -> LoadResult<'static> {
        Box::pin(async move { Ok(()) })
    }

    fn cleanup(_ctx: EmptyData) -> CleanUpResult<'static> {
        Box::pin(async move {})
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_init_step_when_adding_additional_init_step_then_panics() {
        let first_init_step = TestStep::<'static, EmptyData>::as_init(Box::new(init));
        let second_init_step = TestStep::<'static, EmptyData>::as_init(Box::new(init));
        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(
            TEST_NAME,
            TEST_SUITE,
            EmptyData::default(),
        );
        test_case.with_step(first_init_step);
        test_case.with_step(second_init_step);
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_clean_up_step_when_adding_additional_clean_up_step_then_panics() {
        let first_clean_up_step = TestStep::<'static, EmptyData>::as_clean_up(Box::new(cleanup));
        let second_clean_up_step = TestStep::<'static, EmptyData>::as_clean_up(Box::new(cleanup));
        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(
            TEST_NAME,
            TEST_SUITE,
            EmptyData::default(),
        );
        test_case.with_step(first_clean_up_step);
        test_case.with_step(second_clean_up_step);
    }

    #[test]
    #[should_panic]
    fn given_test_case_with_warm_up_step_when_adding_additional_warm_up_step_then_panics() {
        let first_warm_up_step =
            TestStep::<'static, EmptyData>::as_warm_up(Box::new(warmup), Vec::default());
        let second_warm_up_step =
            TestStep::<'static, EmptyData>::as_warm_up(Box::new(warmup), Vec::default());
        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(
            TEST_NAME,
            TEST_SUITE,
            EmptyData::default(),
        );
        test_case.with_step(first_warm_up_step);
        test_case.with_step(second_warm_up_step);
    }

    #[test]
    fn given_test_case_with_steps_when_getting_ordered_steps_then_ensure_proper_ordering() {
        const FIRST_LOAD_STEP: &str = "first";
        const SECOND_LOAD_STEP: &str = "second";
        const THIRD_LOAD_STEP: &str = "third";

        let mut test_case = TestCase::<'static, TestCaseContext, EmptyData>::new(
            TEST_NAME,
            TEST_SUITE,
            EmptyData::default(),
        );
        let init_step = TestStep::<'static, EmptyData>::as_init(Box::new(init));
        let clean_up_step = TestStep::<'static, EmptyData>::as_clean_up(Box::new(cleanup));
        let warm_up_step =
            TestStep::<'static, EmptyData>::as_warm_up(Box::new(warmup), Vec::default());
        let first_load_step = TestStep::<'static, EmptyData>::as_load(
            FIRST_LOAD_STEP,
            Box::new(load),
            Vec::default(),
        );
        let second_load_step = TestStep::<'static, EmptyData>::as_load(
            SECOND_LOAD_STEP,
            Box::new(load),
            Vec::default(),
        );
        let third_load_step = TestStep::<'static, EmptyData>::as_load(
            THIRD_LOAD_STEP,
            Box::new(load),
            Vec::default(),
        );

        test_case.with_step(clean_up_step);
        test_case.with_step(warm_up_step);
        test_case.with_step(init_step);
        test_case.with_step(first_load_step);
        test_case.with_step(second_load_step);
        test_case.with_step(third_load_step);

        let actual = &test_case.test_steps;

        assert_eq!(test_case.test_steps.len(), 6);
        assert_eq!(actual.len(), test_case.test_steps.len());

        for i in 0..actual.len() {
            let step = actual.get(i).unwrap();

            let expected_index = match step {
                TestStep::Init { .. } => 0,
                TestStep::WarmUp { .. } => 1,
                TestStep::Load { name, .. } => match *name {
                    FIRST_LOAD_STEP => 2,
                    SECOND_LOAD_STEP => 3,
                    THIRD_LOAD_STEP => 4,
                    _ => todo!(),
                },
                TestStep::CleanUp { .. } => 5,
            };

            assert_eq!(i, expected_index);
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
