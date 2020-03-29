use itertools::Itertools;
use redis::Commands;
use serde_json::Value as JsonValue;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Connection {
    inner: redis::Connection,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Process {
    pub busy: u8,
    #[serde(with = "serde_with::json::nested")]
    pub info: ProcessInfo,
    pub quiet: bool,
    pub beat: f64
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessInfo {
    pub hostname: String,
    pub started_at: f64,
    pub pid: u32,
    pub tag: String,
    pub concurrency: u8,
    pub queues: Vec<String>,
    pub labels: Vec<String>,
    pub identity: String
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryInfo {
    pub enqueued_at: f64,
    pub error_class: String,
    pub error_message: String,
    pub failed_at: f64,
    pub retried_at: f64,
    pub retry_count: u8
}

impl Connection {
    pub fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(Self { inner: conn })
    }

    pub fn process_names(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers("processes")?)
    }

    pub fn process(&mut self, process_name: &str) -> Result<Process> {
        Ok(serde_redis::from_redis_value(self.inner.hgetall(process_name)?)?)
    }

    pub fn workers(&mut self, process_name: &str) -> Result<HashMap<String, JsonValue>> {
        let raw_result: HashMap<String, String> = self.inner.hgetall(format!("{}:workers", process_name))?;
        let result: Result<HashMap<String, JsonValue>> = raw_result.into_iter().map(|(id, worker)| Ok((id, serde_json::from_str(&worker)?)) ).try_collect();
        result
    }

    pub fn queue_names(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers("queues")?)
    }

    pub fn queue(&mut self, queue_name: &str) -> Result<Vec<Job>> {
        self.jobs("lrange", &format!("queue:{}", queue_name))
    }

    pub fn retry(&mut self) -> Result<Vec<Job>> {
        self.jobs("zrange", "retry")
    }

    pub fn schedule(&mut self) -> Result<Vec<Job>> {
        self.jobs("zrange", "schedule")
    }

    pub fn dead(&mut self) -> Result<Vec<Job>> {
        self.jobs("zrange", "dead")
    }

    fn jobs(&mut self, method: &str, name: &str) -> Result<Vec<Job>> {
        let method = match method {
            "zrange" => redis::Connection::zrange,
            "lrange" => redis::Connection::lrange,
            _ => panic!("unsupported method {}", method)
        };
        let raw_result: Vec<String> = method(&self.inner, name, 0, -1)?;
        let result = raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?;
        Ok(result)
    }
}
