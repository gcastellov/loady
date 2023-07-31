use loady::{TestCaseBuilder};
use loady::core::{TestCaseContext};
use loady::core::runner::{TestRunner};
use std::sync::{Mutex,Arc};
use std::time::{Duration};
use std::thread;
use rand::prelude::*;

#[derive(Default,Clone,Copy,Debug)]
struct InnerContext {
}

fn main() {

    let positive_callback = |_: &Arc::<Mutex::<TestCaseContext::<InnerContext>>>| -> Result<(), u32> {
        thread::sleep(Duration::from_millis(50));
        Ok(())
    };

    let negative_callback = |_: &Arc::<Mutex::<TestCaseContext::<InnerContext>>>| -> Result<(), u32> {        
        thread::sleep(Duration::from_millis(25));

        let mut rng = rand::thread_rng();
        let mut nums: Vec<u32> = (400..410).collect();
        nums.shuffle(&mut rng);

        Err(*nums.get(0).unwrap())
    };

    let test_case = TestCaseBuilder::<InnerContext, u32>
        ::new("simple sample", "samples")
        .with_step("first", positive_callback)
            .with_stage("warm up", Duration::from_secs(10), Duration::from_secs(1), 1)
        .with_step("second", negative_callback)
            .with_stage("load", Duration::from_secs(20), Duration::from_secs(1), 10)
        .build();

    let mut runner = TestRunner::new();
    runner.with_default_reporting_sink();
    runner.with_default_output_files();
    runner.with_test_summary_std_out();
    runner.with_reporting_frequency(5);
    
    let _ = runner.run(test_case);
}