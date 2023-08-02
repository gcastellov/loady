# Loady [WIP]
Technology agnostic load testing tool that helps you define your load tests by using the desired communication protocols (HTTP/WebSockets/AMQP etc), libraries and so on.

## Metrics
The runner extracts metrics of the test execution during different intants of the execution. 

Once every step finishes, the runner will calculate and report these metrics. In the same way, these metrics will be handled when a single actions is completed depending on the frequency you set. The default frequency is *5 seconds*.

| Metric | Unit |
|---|---|
| Successful hits count | number |
| Unsuccessful hits count | number |
| All hits count | number |
| Errors count | number |
| Test Duration| ms |
| Min Time | ms |
| Mean Time | ms |
| Max Time | ms |
| p90% Time | ms |
| p95% Time | ms |
| p99% Time | ms |


When you define the callback action of your testing step, return the error code once it fails. This way, the app will be able to collect and present how many errors occurred by error code.

## Features

### Reporting sinks
Extract real-time metrics and save them into your desired output target, either is a database, a rolling file or just STD OUT.

By default, the app allows you the use the default reporting sink which prints the metrics to the STD OUT in a very simple way.

The *ReportingSink* trait has three hooks for reporting metrics:

    - on_tests_ended
    - on_step_ended
    - on_action_ended 

Be aware that on_action_ended is triggered depending on the reporting frequency setting.

### Exporting
Tests run metrics can be saved into TXT and CSV files to later digest the data. 

By default the library creates a directory called *output* inside the binary directory and saves the files there. However, you can define the location for any of them.

### Test summary
Show or hide the test summary depending on your needs. 