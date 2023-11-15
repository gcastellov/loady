use loady::core::functions::*;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use tokio::time::sleep;
use tokio::time::Duration;

pub const TEST_NAME: &'static str = "simple sample";
pub const TEST_SUITE: &'static str = "samples";

#[allow(dead_code)]
pub const TEST_STEP_1: &'static str = "first";

#[allow(dead_code)]
pub const TEST_STEP_2: &'static str = "second";

#[allow(dead_code)]
pub const TEST_STAGE_1: &'static str = "warm up";

#[allow(dead_code)]
pub const TEST_STAGE_2: &'static str = "load";

#[allow(dead_code)]
pub const CHANNEL_BUFFER_SIZE: usize = 10;

#[derive(Default, Clone, Debug)]
pub struct EmptyData;

#[allow(dead_code)]
pub fn init(ctx: EmptyData) -> InitResult<'static, EmptyData> {
    Box::pin(async move { Ok(ctx.to_owned()) })
}

#[allow(dead_code)]
pub fn warmup(_ctx: Arc<EmptyData>) -> WarmUpResult<'static> {
    Box::pin(async move {})
}

#[allow(dead_code)]
pub fn load(_ctx: Arc<EmptyData>) -> LoadResult<'static> {
    Box::pin(async move {
        let mut rng: StdRng = SeedableRng::from_entropy();
        let millis = rng.gen_range(25..100);
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

#[allow(dead_code)]
pub fn cleanup(_ctx: EmptyData) -> CleanUpResult<'static> {
    Box::pin(async move {})
}
