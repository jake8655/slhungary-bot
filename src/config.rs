use std::{env, fs, io::Read};

use serde::{Deserialize, Serialize};
use tracing::info;

pub struct Config {
    pub data_json: DataJson,
    pub token: String,
    pub suggestions_channel_id: u64,
    pub bug_report_channel_id: u64,
    pub bug_log_channel_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct DataJson {
    #[serde(rename(deserialize = "suggestionCount", serialize = "suggestionCount"))]
    pub suggestion_count: u16,
    #[serde(rename(deserialize = "bugReportCount", serialize = "bugReportCount"))]
    pub bug_report_count: u16,
}

impl Default for DataJson {
    fn default() -> Self {
        Self::new()
    }
}

impl DataJson {
    pub fn new() -> Self {
        Self {
            suggestion_count: 0,
            bug_report_count: 0,
        }
    }

    pub fn load(self) -> Self {
        match fs::File::open("./data.json") {
            Ok(mut file) => {
                let mut file_contents = String::new();
                file.read_to_string(&mut file_contents)
                    .expect("Expected to read `data.json` file");

                match serde_json::from_str(file_contents.trim()) {
                    Ok(data) => data,
                    Err(_) => {
                        info!("`data.json` is invalid. Creating new `data.json` file");
                        let new = Self::new();
                        new.save();
                        new
                    }
                }
            }
            Err(_) => {
                info!("`data.json` not found. Creating new `data.json` file");
                let new = Self::new();
                new.save();
                new
            }
        }
    }

    pub fn save(&self) {
        serde_json::to_writer(
            fs::File::create("./data.json").expect("Expected to create `data.json` file"),
            &self,
        )
        .expect("Expected to write to `data.json` file");
    }

    pub fn increment_suggestion_count(&mut self) {
        self.suggestion_count += 1;
    }

    pub fn increment_bug_report_count(&mut self) {
        self.bug_report_count += 1;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let token = env::var("DISCORD_TOKEN").expect("Expected `DISCORD_TOKEN` in the environment");
        let suggestions_channel_id = env::var("SUGGESTIONS_CHANNEL_ID")
            .expect("Expected `SUGGESTIONS_CHANNEL_ID` in the environment")
            .parse::<u64>()
            .expect("Expected `SUGGESTIONS_CHANNEL_ID` to be a number");
        let bug_report_channel_id = env::var("BUG_REPORT_CHANNEL_ID")
            .expect("Expected `BUG_REPORT_CHANNEL_ID` in the environment")
            .parse::<u64>()
            .expect("Expected `BUG_REPORT_CHANNEL_ID` to be a number");
        let bug_log_channel_id = env::var("BUG_LOG_CHANNEL_ID")
            .expect("Expected `BUG_LOG_CHANNEL_ID` in the environment")
            .parse::<u64>()
            .expect("Expected `BUG_LOG_CHANNEL_ID` to be a number");

        Self {
            data_json: DataJson::new().load(),
            token,
            suggestions_channel_id,
            bug_report_channel_id,
            bug_log_channel_id,
        }
    }
}
