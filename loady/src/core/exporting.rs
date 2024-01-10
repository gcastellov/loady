use crate::core::stats::{Metrics, StepStatus, TestStatus};
use num_format::{Locale, ToFormattedString};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

#[derive(Eq, PartialEq, Debug)]
pub enum FileType {
    Txt,
    Csv,
    Json,
}

enum FileContent<'a> {
    Txt(TestReport<'a>),
    Csv(TestReport<'a>),
    Json(TestReport<'a>),
}

#[derive(Default)]
pub struct Exporter {
    export_files: Vec<ExportFile>,
}

#[derive(Default)]
pub struct Localization;

#[derive(Serialize)]
struct TestReport<'a> {
    test_status: &'a TestStatus,
    step_status: &'a [StepStatus],
}

struct ExportFile {
    file_type: FileType,
    directory: String,
    file_name: String,
}

impl Localization {
    fn format_number(&self, num: &u128) -> String {
        num.to_formatted_string(&Locale::en)
    }

    fn format_float(&self, num: &f64) -> String {
        format!("{:.2}", num)
    }

    fn format_duration(&self, duration: &u128) -> String {
        self.format_number(duration)
    }
}

impl TestStatus {
    pub fn as_txt(&self, locale: &Localization) -> String {
        format!(
            "{: <20}: {}\r\n{: <20}: {}\r\n\r\n{}",
            "Session ID",
            self.session_id,
            "Test Case",
            self.test_name,
            self.metrics.as_txt(locale)
        )
    }

    fn as_csv(&self, locale: &Localization) -> String {
        format!(
            "{};{};{}",
            self.session_id,
            self.test_name,
            self.metrics.as_csv(locale)
        )
    }
}

impl StepStatus {
    pub fn as_txt(&self, locale: &Localization) -> String {
        format!(
            "{: <20}: {}\r\n\r\n{}",
            "Test Step",
            self.step_name,
            self.metrics.as_txt(locale)
        )
    }

    fn as_csv(&self, locale: &Localization) -> String {
        format!("{};{}", self.step_name, self.metrics.as_csv(locale))
    }
}

impl Metrics {
    fn as_txt(&self, locale: &Localization) -> String {
        let mut content = format!("{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n\r\n{: <20}: {:}\r\n{: <20}: {:}\r\n{: <20}: {:}\r\n{: <20}: {:}", 
            "Test Duration",
            locale.format_duration(&self.test_duration),
            "Load Duration",
            locale.format_duration(&self.load_duration),
            "Min Time",
            locale.format_duration(&self.min_time),
            "Mean Time", 
            locale.format_duration(&self.mean_time),
            "Max Time",
            locale.format_duration(&self.max_time),
            "Std Dev",
            locale.format_duration(&self.std_dev),
            "p90",
            locale.format_duration(&self.p90_time),
            "p95",
            locale.format_duration(&self.p95_time),
            "p99",
            locale.format_duration(&self.p99_time),
            "All Hits",
            locale.format_number(&self.all_hits),
            "Successful hits",
            locale.format_number(&self.positive_hits),
            "Unsuccessul hits",
            locale.format_number(&self.negative_hits),
            "Requests/sec",
            locale.format_float(&self.request_per_sec),
        );

        if !self.errors.is_empty() {
            content += &self.errors.iter().fold(
                format!("\r\n\r\n{: <20}:\r\n\r\n", "Errors count"),
                |curr, (key, val)| curr + &format!("{: <20}: {:}\r\n", key, val),
            );
        }

        content
    }

    fn as_csv(&self, locale: &Localization) -> String {
        let mut content = format!(
            "{:};{:};{:};{:};{:};{:};{:};{:};{:};{:};{:};{:};{:}",
            locale.format_duration(&self.test_duration),
            locale.format_duration(&self.load_duration),
            locale.format_duration(&self.min_time),
            locale.format_duration(&self.mean_time),
            locale.format_duration(&self.max_time),
            locale.format_duration(&self.std_dev),
            locale.format_duration(&self.p90_time),
            locale.format_duration(&self.p95_time),
            locale.format_duration(&self.p99_time),
            locale.format_number(&self.all_hits),
            locale.format_number(&self.positive_hits),
            locale.format_number(&self.negative_hits),
            locale.format_float(&self.request_per_sec),
        );

        if !self.errors.is_empty() {
            content = self.errors.iter().fold(content, |curr, (key, val)| {
                curr + &format!(";{};{}", key, val)
            });
        }

        content
    }
}

impl Display for FileContent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const STEP_SEPARATOR: &str = "\r\n\r\n----------------------------------------------------------------------\r\n\r\n";
        const NEW_LINE: &str = "\r\n";

        let locale = Localization::default();

        let content = match self {
            FileContent::Txt(report) => report
                .step_status
                .iter()
                .fold(report.test_status.as_txt(&locale), |cur, nxt| {
                    cur + format!("{}{}", STEP_SEPARATOR, nxt.as_txt(&locale)).as_str()
                }),

            FileContent::Csv(report) => {
                report
                    .step_status
                    .iter()
                    .fold(String::from(""), |cur, nxt| {
                        cur + report.test_status.as_csv(&locale).as_str()
                            + nxt.as_csv(&locale).as_str()
                            + NEW_LINE
                    })
            }

            FileContent::Json(report) => serde_json::to_string(report).unwrap(),
        };

        write!(f, "{}", content)
    }
}

impl FileType {
    pub fn get_content(&self, test_status: TestStatus, step_status: Vec<StepStatus>) -> String {
        let report = TestReport {
            test_status: &test_status,
            step_status: step_status.as_slice(),
        };

        let content = match self {
            Self::Csv => FileContent::Csv(report),
            Self::Txt => FileContent::Txt(report),
            Self::Json => FileContent::Json(report),
        };

        format!("{}", content)
    }

    fn get_extension(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Txt => "txt",
            Self::Json => "json",
        }
    }
}

impl ExportFile {
    fn new(file_type: FileType, directory: String, file_name: String) -> Self {
        ExportFile {
            file_type,
            directory,
            file_name,
        }
    }

    fn format_file_name(&self) -> String {
        let extension = self.file_type.get_extension();

        if self.file_name.ends_with(extension) {
            self.file_name.to_owned()
        } else {
            format!("{}.{}", self.file_name, extension)
        }
    }
}

impl Exporter {
    const SESSION_ID_PATTERN: &str = "{session-id}";

    pub fn with_default_output_files(&mut self) {
        let mut add_default = |file_type: FileType| {
            self.with_output_file(
                file_type,
                String::from("output"),
                Self::SESSION_ID_PATTERN.to_string(),
            );
        };

        add_default(FileType::Txt);
        add_default(FileType::Csv);
        add_default(FileType::Json);
    }

    pub fn with_output_file(&mut self, file_type: FileType, directory: String, file_name: String) {
        self.export_files
            .push(ExportFile::new(file_type, directory, file_name));
    }

    pub fn write_output_files(
        &self,
        test_status: TestStatus,
        step_status: Vec<StepStatus>,
    ) -> std::io::Result<()> {
        for export_file in &self.export_files {
            let content = export_file
                .file_type
                .get_content(test_status.to_owned(), step_status.to_owned());
            let file_name = export_file.format_file_name();
            Self::write_file(
                &export_file.directory,
                &file_name,
                &content,
                &test_status.session_id,
            )?;
        }

        Ok(())
    }

    fn write_file(
        output_directory: &str,
        output_file: &str,
        content: &str,
        session_id: &str,
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(output_directory)?;
        let mut file_name = output_file.replace(Self::SESSION_ID_PATTERN, session_id);
        file_name = format!("{}/{}", output_directory, file_name);
        let mut file = File::create(file_name)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn given_filetype_when_getting_extension_then_returns_expected_value() {
        assert_eq!(FileType::Txt.get_extension(), "txt");
        assert_eq!(FileType::Csv.get_extension(), "csv");
    }

    #[test]
    fn given_exporter_when_adding_default_export_types_then_loads_defaults() {
        let assert_file = |file: &ExportFile, expected_file_type: FileType| {
            assert_eq!(file.file_type, expected_file_type);
            assert!(!file.file_name.is_empty());
            assert!(!file.directory.is_empty());
        };

        let mut exporter = Exporter::default();
        exporter.with_default_output_files();

        assert_eq!(exporter.export_files.len(), 3);
        assert_file(exporter.export_files.get(0).unwrap(), FileType::Txt);
        assert_file(exporter.export_files.get(1).unwrap(), FileType::Csv);
        assert_file(exporter.export_files.get(2).unwrap(), FileType::Json);
    }
}
