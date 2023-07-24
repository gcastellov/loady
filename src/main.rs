use loady::{Runner,TestCaseBuilder};
use loady::core::{TestCaseContext};
use std::sync::{Mutex,Arc};
use std::time::{Duration};

#[derive(Default,Clone,Copy,Debug)]
struct InnerContext {
}

fn main() {

    let positive_callback = |_: &Arc::<Mutex::<TestCaseContext::<InnerContext>>>| -> bool {
        true
    };

    let negative_callback = |_: &Arc::<Mutex::<TestCaseContext::<InnerContext>>>| -> bool {
        false
    };

    let test_case = TestCaseBuilder::<InnerContext>
        ::new(&"simple sample", &"samples")
        .with_step(&"first", positive_callback)
            .with_stage(&"warm up", Duration::from_secs(5), Duration::from_secs(1), 1)
        .with_step(&"second", negative_callback)
            .with_stage(&"load", Duration::from_secs(10), Duration::from_secs(1), 10)
        .build();

    let mut runner = Runner::default();
    runner.with_default_reporting_sink();
    runner.with_default_output_file();
    runner.run(test_case);
}