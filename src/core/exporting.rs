use std::fmt::{Formatter,Result,Display};
use num_format::{Locale, ToFormattedString};
use std::fs::File;
use std::io::{Write};
use crate::core::stats::{TestStatus,StepStatus,Metrics};

pub enum FileType {
    Txt,
    Csv
}

enum FileContent {
    Txt(TestStatus, Vec<StepStatus>),
    Csv(TestStatus, Vec<StepStatus>)
}

#[derive(Default)]
pub struct Exporter {
    export_files: Vec<ExportFile>
}

#[derive(Default)]
struct Localization;

trait Content {
    fn as_csv(&self, locale: &Localization) -> String;
    fn as_txt(&self, locale: &Localization) -> String;
}

struct ExportFile {
    file_type: FileType,
    directory: String,
    file_name: String
}

impl Localization {
    fn format_number(&self, num: &u128) -> String {
        num.to_formatted_string(&Locale::en)
    }

    fn format_duration(&self, duration: &u128) -> String {
        self.format_number(&duration)
    }
}

impl Content for TestStatus {
    fn as_txt(&self, locale: &Localization) -> String {
        format!("{: <20}: {}\r\n{: <20}: {}\r\n\r\n{}", 
            "Session ID",
            self.session_id, 
            "Test Case",
            self.test_name,
            self.metrics.as_txt(locale))
    }

    fn as_csv(&self, locale: &Localization) -> String {
        format!("{};{};{}", 
            self.session_id, 
            self.test_name,
            self.metrics.as_csv(locale))
    }
}

impl Content for StepStatus {
    fn as_txt(&self, locale: &Localization) -> String {
        format!("{: <20}: {}\r\n\r\n{}", 
            "Test Step",
            self.step_name,
            self.metrics.as_txt(locale))
    }

    fn as_csv(&self, locale: &Localization) -> String {
        format!("{};{}", 
            self.step_name,
            self.metrics.as_csv(locale))
    }
}

impl Content for Metrics {
    fn as_txt(&self, locale: &Localization) -> String {        
        let mut content = format!("{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n{: <20}: {:} ms\r\n\r\n{: <20}: {:}\r\n{: <20}: {:}\r\n{: <20}: {:}", 
            "Test Duration",
            locale.format_duration(&self.test_duration),
            "Min Time",
            locale.format_duration(&self.min_time),
            "Mean Time", 
            locale.format_duration(&self.mean_time),
            "Max Time",
            locale.format_duration(&self.max_time),
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
            locale.format_number(&self.negative_hits)
        );


        if self.errors.len() > 0 {
            content += &self.errors
                .iter()
                .fold(format!("\r\n\r\n{: <20}:\r\n\r\n", "Errors count"), |curr, (key, val)| curr + &format!("{: <20}: {:}\r\n", key, val));
        }

        content
    }

    fn as_csv(&self, locale: &Localization) -> String {
        let mut content = format!("{:};{:};{:};{:};{:};{:};{:};{:};{:};{:}", 
            locale.format_duration(&self.test_duration),
            locale.format_duration(&self.min_time),
            locale.format_duration(&self.mean_time),
            locale.format_duration(&self.max_time),
            locale.format_duration(&self.p90_time),
            locale.format_duration(&self.p95_time),
            locale.format_duration(&self.p99_time),
            locale.format_number(&self.all_hits),
            locale.format_number(&self.positive_hits),
            locale.format_number(&self.negative_hits)
        );

        if self.errors.len() > 0 {
            content = self.errors
                .iter()
                .fold(content, |curr, (key, val)| curr + &format!(";{};{}", key, val));
        }

        content
    }
}

impl Display for FileContent {    
    fn fmt(&self, f: &mut Formatter<'_>) -> Result { 
        const STEP_SEPARATOR: &str = "\r\n\r\n----------------------------------------------------------------------\r\n\r\n";
        const NEW_LINE: &str = "\r\n";

        let locale = Localization::default();
        
        let content = match self {
            FileContent::Txt(test_status, steps_status) 
                => steps_status
                    .iter()
                    .fold(test_status.as_txt(&locale), |cur, nxt| cur + format!("{}{}", STEP_SEPARATOR, nxt.as_txt(&locale)).as_str()),
            
            FileContent::Csv(test_status, steps_status)
                => steps_status
                    .iter()
                    .fold(String::from(""), |cur, nxt| cur + test_status.as_csv(&locale).as_str() + nxt.as_csv(&locale).as_str() + NEW_LINE),
        };

        write!(f, "{}", content)
    }
}

impl FileType {
    
    pub fn get_content(&self, test_status: TestStatus, step_status: Vec<StepStatus>) -> String {
        let content = match self {            
            Self::Csv => FileContent::Csv(test_status, step_status),
            Self::Txt => FileContent::Txt(test_status, step_status)
        };

        format!("{}", content)
    }

    fn format_file_name(&self, file_name: &String) -> String {
        let extension = match self {
            Self::Csv => "csv",
            Self::Txt => "txt"
        };

        if file_name.ends_with(extension) { 
            file_name.to_owned() 
        } else { 
            format!("{}.{}", file_name, extension) 
        }
    }
}

impl ExportFile {
    fn new(file_type: FileType, directory: String, file_name: String) -> Self {
        ExportFile {
            file_type,
            directory, 
            file_name
        }
    }
}

impl Exporter {

    const SESSION_ID_PATTERN: &str = "{session-id}";

    pub fn with_default_output_files(&mut self) {

        let mut add_default = |file_type: FileType| {
            self.with_output_file(file_type, String::from("output"), Self::SESSION_ID_PATTERN.to_string());
        };

        add_default(FileType::Txt);
        add_default(FileType::Csv);
    }

    pub fn with_output_file(&mut self, file_type: FileType, directory: String, file_name: String) {
        self.export_files.push(ExportFile::new(file_type, directory, file_name));
    }

    pub fn write_output_files(&self, test_status: TestStatus, step_status: Vec<StepStatus>) -> std::io::Result<()> {
        for export_file in &self.export_files {
            let content = export_file.file_type.get_content(test_status.to_owned(), step_status.to_owned());
            let file_name = export_file.file_type.format_file_name(&export_file.file_name);
            Self::write_file(&export_file.directory, &file_name, &content, &test_status.session_id)?;            
        }

        Ok(())
    }

    fn write_file(output_directory: &str, output_file: &str, content: &str, session_id: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(output_directory)?;
        let mut file_name = output_file.replace(Self::SESSION_ID_PATTERN, session_id);
        file_name = format!("{}/{}", output_directory, file_name);
        let mut file = File::create(file_name)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

