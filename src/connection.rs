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
pub struct Process {
    pub busy: u8,
    #[serde(with = "serde_with::json::nested")]
    pub info: ProcessInfo,
    pub quiet: bool,
    pub beat: f64
}

#[derive(Debug, Deserialize)]
pub struct ProcessInfo {
    pub hostname: String,
    pub started_at: f64,
    pub pid: u16,
    pub tag: String,
    pub concurrency: u8,
    pub queues: Vec<String>,
    pub labels: Vec<String>,
    pub identity: String
}

#[derive(Debug, Deserialize)]
pub struct Retry {
    pub args: Vec<JsonValue>,
    pub class: String,
    pub created_at: f64,
    pub enqueued_at: f64,
    pub error_class: String,
    pub error_message: String,
    pub failed_at: f64,
    pub jid: String,
    pub queue: String,
    pub retried_at: f64,
    pub retry: bool,
    pub retry_count: u8
}

#[derive(Debug, Deserialize)]
pub struct Schedule {
    pub args: Vec<JsonValue>,
    pub class: String,
    pub created_at: f64,
    pub jid: String,
    pub queue: String,
    pub retry: bool
}

#[derive(Debug, Deserialize)]
pub struct Dead {
    pub args: Vec<JsonValue>,
    pub class: String,
    pub created_at: f64,
    pub enqueued_at: f64,
    pub error_class: String,
    pub error_message: String,
    pub failed_at: f64,
    pub jid: String,
    pub queue: String,
    pub retried_at: f64,
    pub retry: bool,
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
        self.inner.hgetall::<_, HashMap<String, String>>(format!("{}:workers", process_name))?.into_iter().map(|(id, worker)| Ok((id, serde_json::from_str(&worker)?)) ).try_collect()
    }

    pub fn queue_names(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers("queues")?)
    }

    pub fn queue(&mut self, queue_name: &str) -> Result<Vec<JsonValue>> {
        Ok(self.inner.lrange::<_, Vec<String>>(format!("queue:{}", queue_name), 0, -1)?.into_iter().map(|item| serde_json::from_str(&item) ).try_collect()?)
    }

    pub fn retry(&mut self) -> Result<Vec<Retry>> {
        Ok(self.inner.zrange::<_, Vec<String>>("retry", 0, -1)?.into_iter().map(|item| serde_json::from_str(&item) ).try_collect()?)
    }

    pub fn schedule(&mut self) -> Result<Vec<Schedule>> {
        Ok(self.inner.zrange::<_, Vec<String>>("schedule", 0, -1)?.into_iter().map(|item| serde_json::from_str(&item) ).try_collect()?)
    }

    pub fn dead(&mut self) -> Result<Vec<Dead>> {
        Ok(self.inner.zrange::<_, Vec<String>>("dead", 0, -1)?.into_iter().map(|item| serde_json::from_str(&item) ).try_collect()?)
    }
}
