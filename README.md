# Loady

[![Rust](https://github.com/gcastellov/loady/actions/workflows/rust.yml/badge.svg)](https://github.com/gcastellov/loady/actions/workflows/rust.yml)

Technology agnostic load testing tool that helps you define your load tests by using the desired communication protocols (HTTP/WebSockets/AMQP etc), libraries and so on.

```rust
#[tokio::main]
async fn main() {

let ctx = InnerContext {
        warmup_url: "http://localhost:8080",
        load_url: "http://localhost:8080/hey",
        client_id: "the client id",
        secret: "the secret",
        access_token: None
    };

    let test_case = TestCaseBuilder::<InnerContext>
        ::new("simple sample", "samples", &ctx)
        .with_init_step(Box::new(Scenario::init))
        .with_warm_up_step(Box::new(Scenario::warmup))
            .with_stage("warm up", Duration::from_secs(10), Duration::from_secs(1), 2)
        .with_load_step("load", Box::new(Scenario::load))    
            .with_stage("first wave", Duration::from_secs(10), Duration::from_secs(1), 15)
            .with_stage("second wave", Duration::from_secs(30), Duration::from_secs(1), 50)
            .with_stage("third wave", Duration::from_secs(10), Duration::from_secs(1), 15)
        .with_clean_up_step(Box::new(Scenario::cleanup))
        .build();

    let runner = TestRunner::new()
        .with_default_reporting_sink()
        .with_default_output_files()
        .with_test_summary_std_out()
        .with_reporting_frequency(5);

    _ = runner.run(test_case).await;
}
```

## Features

### Test steps aka test scenarios

As your test can be composed by multiple scenarios, the application allows you to define different steps which will be executed sequentially. Before executing the loading steps, where all metrics are extracted, the app will execute other steps, if defined, such as *Init* or *Warm Up*. After the loading steps you can define an extra step to perform certain operation like releasing resources or cleaning up data. This is accomplished with the *Clean Up* step.

|Step||
|--|--|
|**Init**|It's executed only once. Useful for seeding data, getting access rights ...|
|**Warm Up**|It's executed only once. Its action will execute as many times as it's defined in its stage's configuration.|
|**Load**|You can add as many load steps you want. Each step will execute only once and its action will execute as many times as it's defined in its stage's configuration.|
|**Clean Up**|It's executed only once. Useful for releasing resources and so on.|

### Reporting sinks
Extract real-time metrics and save them into your desired output target, either is a database, a rolling file or just STD OUT.

By default, the app allows you the use the default reporting sink which prints the metrics to the STD OUT in a very simple way.

The *ReportingSink* trait has three hooks for reporting metrics:

|Hook||
|--|--|
|**on_test_ended**|It triggers once the whole run ends.|
|**on_load_step_ended**|It triggers once a load step ends.|
|**on_load_action_ended**|It triggers on a time basis once a load action ends.|
|**on_internal_step_ended**|It triggers once a *Init*, *Warm Up* or *Clean Up* step ends.|

### Metrics
The runner extracts metrics of the test execution during different intants of the execution. 

Once every step finishes, the runner will calculate and report these metrics. In the same way, these metrics will be handled when a single actions is completed depending on the frequency you set. The default frequency is *5 seconds*.

| Metric | Unit |
|---|---|
| Successful hits count | number |
| Unsuccessful hits count | number |
| All hits count | number |
| Errors count | number |
| Test duration| ms |
| Min time | ms |
| Mean time | ms |
| Max time | ms |
| Standard Deviation | ms |
| p90% time | ms |
| p95% time | ms |
| p99% time | ms |


When you define the callback action of your testing step, return the error code once it fails. This way, the app will be able to collect and present how many errors occurred by error code.

Be aware that on_action_ended is triggered depending on the reporting frequency setting.

### Exporting
Tests metrics can be saved into TXT, CSV or JSON files to later digest the data. 

By default the library creates a directory called *output* inside the binary directory and saves the files there. However, you can define the location for any of them.

### Test summary
Show or hide the test summary depending on your needs. 