# Loady [WIP]
Agnostic load testing tool that helps you to define your load tests by using the desired communication protocols, libraries and so on.

## Features

### Reporting sinks
Extract real-time metrics and save them into your desired output target, either is a database, a rolling file or just STD OUT.

By default, the app allows you the use the default reporting sink which prints the metrics to the STD OUT in a very simple way.

### Exporting
Tests run metrics can be saved into TXT and CSV files to later digest the data. 

By default the library creates a directory called *output* inside the binary directory and saves the files there. However, you can define the location for any of them.

### Test summary
Show or hide the test summary depending on your needs. 