use loady::utils::{TestCaseBuilder};
use loady::core::runner::{TestRunner};
use std::sync::{Arc};
use std::time::{Duration};
use std::thread;
use rand::prelude::*;

#[derive(Default,Clone,Debug)]
struct InnerContext {
    client_id: String,
    secret: String,
    access_token: Option<String>
}

fn main() {

    let init_callback = |data: InnerContext| -> Result<InnerContext, i32> {
        let mut data = data.clone();
        if !data.client_id.is_empty() && !data.secret.is_empty() {
            data.access_token = Some("the access token".to_string());
        }

        Ok(data)
    };

    let clean_up_callback = |_: InnerContext| {
        thread::sleep(Duration::from_millis(500));
    };

    let warm_up_callback = |data: &Arc::<InnerContext>| {

        if data.access_token.is_none() {
            panic!("Access token wasn't provided");
        }

        thread::sleep(Duration::from_millis(500));
    };

    let callback = |data: &Arc::<InnerContext>| -> Result<(), i32> {      
        
        if data.access_token.is_none() {
            panic!("Access token wasn't provided");
        }
        
        let mut rng = rand::thread_rng();
        let mut nums: Vec<i32> = (400..410).collect();
        let mut times: Vec<u64> = (25..200).collect();        
        nums.push(200);
        nums.shuffle(&mut rng);
        times.shuffle(&mut rng);

        thread::sleep(Duration::from_millis(*times.first().unwrap()));

        let code = nums.get(0).unwrap();
        match code {
            200 => Ok(()),
            _ => Err(*code)
        }
    };    

    let ctx = InnerContext {
        client_id: "the client id".to_string(),
        secret: "the secret".to_string(),
        access_token: None
    };

    let test_case = TestCaseBuilder::<InnerContext>
        ::new("simple sample", "samples", &ctx)
        .with_init_step(init_callback)
        .with_warm_up_step(warm_up_callback)
            .with_stage("warm up", Duration::from_secs(10), Duration::from_secs(1), 1)
        .with_load_step("load", callback)    
            .with_stage("first wave", Duration::from_secs(20), Duration::from_secs(1), 10)
            .with_stage("second wave", Duration::from_secs(20), Duration::from_secs(1), 10)
        .with_clean_up_step(clean_up_callback)
        .build();

    let _ = TestRunner::new()
        .with_default_reporting_sink()
        .with_default_output_files()
        .with_test_summary_std_out()
        .with_reporting_frequency(5)
        .run(test_case);
}