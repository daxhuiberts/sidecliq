use itertools::Itertools;
use redis::Commands;
use std::collections::HashMap;
use super::types::*;

pub struct Client {
    inner: redis::Connection,
}

impl Client {
    pub fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(Self { inner: conn })
    }

    pub fn process_names(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers("processes")?)
    }

    pub fn process(&mut self, process_name: &str) -> Result<Process> {
        use serde::{Deserialize, Serialize};
        #[derive(Debug, Deserialize, Serialize)]
        #[serde(deny_unknown_fields)]
        pub struct ProcessRaw {
            pub busy: u8,
            #[serde(deserialize_with = "serde_with::json::nested::deserialize")]
            pub info: JsonValue,
            pub quiet: bool,
            pub beat: f64
        }

        let mut process: ProcessRaw = serde_redis::from_redis_value(self.inner.hgetall(process_name)?)?;
        let info = std::mem::take(&mut process.info);
        let mut map = if let JsonValue::Object(map) = info { Ok(map) } else { Err("process.info is not a json object") }?;
        map.insert("busy".into(), JsonValue::Number(process.busy.into()));
        map.insert("quiet".into(), JsonValue::Bool(process.quiet));
        map.insert("beat".into(), JsonValue::Number(serde_json::Number::from_f64(process.beat).unwrap()));
        Ok(serde_json::from_value(JsonValue::Object(map))?)
    }

    pub fn workers(&mut self, process_name: &str) -> Result<HashMap<String, Job>> {
        let raw_result: HashMap<String, String> = self.inner.hgetall(format!("{}:workers", process_name))?;
        let result: Result<HashMap<String, Job>> = raw_result.into_iter().map(|(id, worker)| Ok((id, serde_json::from_str(&worker)?)) ).try_collect();
        result
    }

    pub fn queue_names(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers("queues")?)
    }

    pub fn queue_jobs(&mut self, queue_name: &str) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = self.inner.lrange(format!("queue:{}", queue_name), 0, 10)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }

    pub fn retry_jobs(&mut self) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = self.inner.zrange("retry", 0, 10)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }

    pub fn schedule_jobs(&mut self) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = self.inner.zrange("schedule", 0, 10)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }

    pub fn dead_jobs(&mut self) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = self.inner.zrange("dead", 0, 10)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }
}
