pub use serde_json::Value as JsonValue;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessInfo {
    pub busy: u8,
    pub hostname: String,
    pub started_at: f64,
    pub pid: u32,
    pub tag: String,
    pub concurrency: u8,
    pub queues: Vec<String>,
    pub labels: Vec<String>,
    pub identity: String,
    pub quiet: bool,
    pub beat: f64
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Worker {
    pub id: String,
    pub run_at: i64,
    pub queue: String,
    pub job: Job,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub args: Vec<JsonValue>,
    pub class: String,
    pub created_at: f64,
    pub jid: String,
    pub queue: String,
    pub retry: bool,
    #[serde(flatten)]
    pub retry_info: Option<RetryInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetryInfo {
    pub enqueued_at: f64,
    pub error_class: String,
    pub error_message: String,
    pub failed_at: f64,
    pub retried_at: f64,
    pub retry_count: u8
}
