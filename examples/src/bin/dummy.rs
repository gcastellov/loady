use loady::core::functions::*;
use loady::core::runner::TestRunner;
use loady::utils::TestCaseBuilder;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Default, Clone, Debug)]
struct InnerContext;

struct Scenario;

impl Scenario {
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
}

#[tokio::main]
async fn main() {
    let ctx = InnerContext {};

    let test_case = TestCaseBuilder::<InnerContext>::new("simple sample", "samples", &ctx)
        .with_load_step("load", Box::new(Scenario::load))
        .with_stage(
            "first wave",
            Duration::from_secs(20),
            Duration::from_secs(1),
            10,
        )
        .with_stage(
            "second wave",
            Duration::from_secs(20),
            Duration::from_secs(1),
            10,
        )
        .build();

    let runner = TestRunner::default()
        .with_default_output_files()
        .with_test_summary_std_out();

    _ = runner.run(test_case).await;
}
