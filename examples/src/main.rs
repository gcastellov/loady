use tokio::time::Duration;
use tokio::time::sleep;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::prelude::SliceRandom;
use loady::utils::{TestCaseBuilder};
use loady::core::runner::{TestRunner};
use loady::core::functions::*;
use std::sync::{Arc};

#[derive(Default,Clone,Debug)]
struct InnerContext {
    client_id: String,
    secret: String,
    access_token: Option<String>
}

struct Scenario;

impl Scenario {
    fn init(ctx: InnerContext) -> InitResult<'static, InnerContext> {
        Box::pin(async move {
            if !ctx.client_id.is_empty() && !ctx.secret.is_empty() {
                let token = String::from("the access token");
                let ctx = InnerContext { 
                    access_token: Some(token), 
                    ..ctx
                };
    
                return Ok(ctx);
            }
    
            Err(401)
        })
    }
    
    fn warmup(ctx: Arc<InnerContext>) -> WarmUpResult<'static> {
        Box::pin(async move {
            sleep(Duration::from_millis(500)).await;
            if ctx.access_token.is_none() {
                panic!("Access token wasn't provided");
            }
        })
    }
    
    fn load(_ctx: Arc<InnerContext>) -> LoadResult<'static> {
        Box::pin(async move {
            let mut rng: StdRng = SeedableRng::from_entropy();
            let millis = rng.gen_range(25..200);
            let mut codes = vec![200];
            codes.extend_from_slice(&[400,401,403,500]);
            let status_code = *codes.choose(&mut rng).unwrap();
    
            sleep(Duration::from_millis(millis)).await;
            
            match status_code {
                200 => Ok(()),
                _ => Err(status_code)
            }
        })
    }
    
    fn cleanup(_ctx: InnerContext) -> CleanUpResult<'static> {
        Box::pin(async move {
            sleep(Duration::from_millis(500)).await;
        })
    }
}

#[tokio::main]
async fn main() {

    let ctx = InnerContext {
        client_id: "the client id".to_string(),
        secret: "the secret".to_string(),
        access_token: None
    };

    let test_case = TestCaseBuilder::<InnerContext>
        ::new("simple sample", "samples", &ctx)
        .with_init_step(Box::new(Scenario::init))
        .with_warm_up_step(Box::new(Scenario::warmup))
            .with_stage("warm up", Duration::from_secs(10), Duration::from_secs(1), 1)
        .with_load_step("load", Box::new(Scenario::load))    
            .with_stage("first wave", Duration::from_secs(20), Duration::from_secs(1), 10)
            .with_stage("second wave", Duration::from_secs(20), Duration::from_secs(1), 10)
        .with_clean_up_step(Box::new(Scenario::cleanup))
        .build();

    let runner = TestRunner::new()
        .with_default_reporting_sink()
        .with_default_output_files()
        .with_test_summary_std_out()
        .with_reporting_frequency(5);

    _ = runner.run(test_case).await;
}