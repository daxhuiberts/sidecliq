use itertools::Itertools;
use std::collections::HashMap;
use super::connection::Connection;
use super::types::*;

pub struct Client {
    connection: Connection,
}

impl Client {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self { connection: Connection::new(url)? })
    }

    pub fn process_names(&mut self) -> Result<Vec<String>> {
        Ok(serde_redis::from_redis_value(self.connection.process_names()?)?)
    }

    pub fn process(&mut self, process_name: &str) -> Result<Process> {
        use serde::{Deserialize, Serialize};
        #[derive(Debug, Deserialize, Serialize)]
        #[serde(deny_unknown_fields)]
        pub struct ProcessRaw {
            pub busy: bool,
            #[serde(deserialize_with = "serde_with::json::nested::deserialize")]
            pub info: JsonValue,
            pub quiet: bool,
            pub beat: f64
        }

        let mut process: ProcessRaw = serde_redis::from_redis_value(self.connection.process(process_name)?)?;
        let info = std::mem::take(&mut process.info);
        let mut map = if let JsonValue::Object(map) = info { Ok(map) } else { Err("process.info is not a json object") }?;
        map.insert("busy".into(), JsonValue::Bool(process.busy));
        map.insert("quiet".into(), JsonValue::Bool(process.quiet));
        map.insert("beat".into(), JsonValue::Number(serde_json::Number::from_f64(process.beat).unwrap()));
        Ok(serde_json::from_value(JsonValue::Object(map))?)
    }

    pub fn workers(&mut self, process_name: &str) -> Result<HashMap<String, Job>> {
        let raw_result: HashMap<String, String> = serde_redis::from_redis_value(self.connection.workers(&format!("{}:workers", process_name))?)?;
        let result: Result<HashMap<String, Job>> = raw_result.into_iter().map(|(id, worker)| Ok((id, serde_json::from_str(&worker)?)) ).try_collect();
        result
    }

    pub fn queue_names(&mut self) -> Result<Vec<String>> {
        Ok(serde_redis::from_redis_value(self.connection.queue_names()?)?)
    }

    pub fn queue(&mut self, queue_name: &str) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = serde_redis::from_redis_value(self.connection.queue(queue_name)?)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }

    pub fn retry(&mut self) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = serde_redis::from_redis_value(self.connection.retry()?)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }

    pub fn schedule(&mut self) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = serde_redis::from_redis_value(self.connection.schedule()?)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }

    pub fn dead(&mut self) -> Result<Vec<Job>> {
        let raw_result: Vec<String> = serde_redis::from_redis_value(self.connection.dead()?)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }
}
