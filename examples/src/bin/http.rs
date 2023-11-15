use loady::core::functions::*;
use loady::core::runner::TestRunner;
use loady::utils::TestCaseBuilder;
use reqwest::{Error, Response, StatusCode};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Default, Clone, Debug)]
struct InnerContext {
    warmup_url: &'static str,
    load_url: &'static str,
    client_id: &'static str,
    secret: &'static str,
    access_token: Option<String>,
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
            if ctx.access_token.is_none() {
                panic!("Access token wasn't provided");
            }

            let result = execute_request(ctx.warmup_url).await;

            if let Ok(response) = result {
                let content = response.text().await;
                println!("Response is: {}", content.unwrap());
            } else {
                println!("Failing at warm up");
            }
        })
    }

    fn load(ctx: Arc<InnerContext>) -> LoadResult<'static> {
        Box::pin(async move {
            let result = execute_request(ctx.load_url).await;

            match result {
                Ok(_) => Ok(()),
                Err(e) => match e.is_status() {
                    true => Err(e.status().unwrap().as_u16() as i32),
                    false => Err(StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32),
                },
            }
        })
    }

    fn cleanup(_ctx: InnerContext) -> CleanUpResult<'static> {
        Box::pin(async move {
            sleep(Duration::from_millis(500)).await;
        })
    }
}

async fn execute_request(url: &str) -> Result<Response, Error> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    Ok(response)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    let ctx = InnerContext {
        warmup_url: "http://localhost:8080",
        load_url: "http://localhost:8080/hey",
        client_id: "the client id",
        secret: "the secret",
        access_token: None,
    };

    let test_case = TestCaseBuilder::<InnerContext>::new("simple sample", "samples", &ctx)
        .with_init_step(Box::new(Scenario::init))
        .with_warm_up_step(Box::new(Scenario::warmup))
        .with_stage(
            "warm up",
            Duration::from_secs(10),
            Duration::from_secs(1),
            2,
        )
        .with_load_step("load", Box::new(Scenario::load))
        .with_stage(
            "first wave",
            Duration::from_secs(5),
            Duration::from_secs(1),
            15,
        )
        .with_stage(
            "second wave",
            Duration::from_secs(10),
            Duration::from_secs(1),
            50,
        )
        .with_stage(
            "third wave",
            Duration::from_secs(5),
            Duration::from_secs(1),
            15,
        )
        .with_clean_up_step(Box::new(Scenario::cleanup))
        .build();

    let runner = TestRunner::default()
        .with_default_reporting_sink()
        .with_default_output_files()
        .with_test_summary_std_out()
        .with_reporting_frequency(5);

    _ = runner.run(test_case).await;
}
