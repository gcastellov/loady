use std::sync::Arc;

use loady::{
    core::{
        composition::TestCase,
        context::TestCaseContext,
        functions::{InitResult, LoadResult},
    },
    utils::TestCaseBuilder,
};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use tokio::time::{sleep, Duration};

pub(crate) struct Sample;

#[derive(Default, Clone, Debug)]
pub struct InnerContext;

fn init(_: InnerContext) -> InitResult<'static, InnerContext> {
    Box::pin(async move {
        let ctx = InnerContext {};
        Ok(ctx)
    })
}

fn load(_ctx: Arc<InnerContext>) -> LoadResult<'static> {
    Box::pin(async move {
        let mut rng: StdRng = SeedableRng::from_entropy();
        let millis = rng.gen_range(25..200);
        let mut codes = vec![200];
        codes.extend_from_slice(&[400, 401, 403, 500]);
        let status_code = *codes.choose(&mut rng).unwrap();

        sleep(Duration::from_millis(millis)).await;

        match status_code {
            200 => Ok(()),
            _ => Err(status_code),
        }
    })
}

impl Sample {
    pub fn build_test_case() -> TestCase<'static, TestCaseContext<'static>, InnerContext> {
        let ctx = InnerContext {};

        TestCaseBuilder::<InnerContext>::new("simple sample", "samples", &ctx)
            .with_init_step(Box::new(init))
            .with_load_step("load", Box::new(load))
            .with_stage(
                "first wave",
                Duration::from_secs(15),
                Duration::from_secs(2),
                5,
            )
            .with_stage(
                "second wave",
                Duration::from_secs(30),
                Duration::from_secs(1),
                10,
            )
            .with_stage(
                "third wave",
                Duration::from_secs(30),
                Duration::from_secs(2),
                5,
            )
            .with_stage(
                "forth wave",
                Duration::from_secs(15),
                Duration::from_secs(2),
                2,
            )
            .build()
    }
}
