use loady::{TestCaseBuilder};
use loady::core::{TestCaseContext};
use loady::core::runner::{TestRunner};
use std::sync::{Mutex,Arc};
use std::time::{Duration};
use std::thread;
use rand::prelude::*;

#[derive(Default,Clone,Debug)]
struct InnerContext;

fn main() {

    let callback = |_: &Arc::<Mutex::<TestCaseContext::<InnerContext>>>| -> Result<(), i32> {        
        thread::sleep(Duration::from_millis(25));

        let mut rng = rand::thread_rng();
        let mut nums: Vec<i32> = (400..410).collect();
        nums.push(200);
        nums.shuffle(&mut rng);

        let code = nums.get(0).unwrap();

        match code {
            200 => Ok(()),
            _ => Err(*code)
        }
    };

    let test_case = TestCaseBuilder::<InnerContext>
        ::new("simple sample", "samples")
        .with_step("first", callback)
            .with_stage("warm up", Duration::from_secs(10), Duration::from_secs(1), 1)
        .with_step("second", callback)
            .with_stage("load", Duration::from_secs(20), Duration::from_secs(1), 10)
        .build();

    let _ = TestRunner::new()
        .with_default_reporting_sink()
        .with_default_output_files()
        .with_test_summary_std_out()
        .with_reporting_frequency(5)
        .run(test_case);
}