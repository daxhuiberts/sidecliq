use itertools::Itertools;
use redis::Commands;
use std::collections::HashMap;
use super::types::*;

pub struct Connection {
    inner: redis::Connection,
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

    pub fn process<'a>(&'a mut self, process_name: &'a str) -> ConnectionProcess<'a> {
        ConnectionProcess {
            inner: &mut self.inner,
            name: process_name,
        }
    }

    pub fn queue_names(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers("queues")?)
    }

    pub fn queue<'a>(&'a mut self, queue_name: &str) -> ConnectionQueue<'a> {
        ConnectionQueue {
            inner: &mut self.inner,
            name: std::borrow::Cow::Owned(format!("queue:{}", queue_name)),
            redis_type: ConnectionQueueType::List,
        }
    }

    pub fn retry<'a>(&'a mut self) -> ConnectionQueue<'a> {
        ConnectionQueue {
            inner: &mut self.inner,
            name: std::borrow::Cow::Borrowed("retry"),
            redis_type: ConnectionQueueType::SortedSet,
        }
    }

    pub fn schedule<'a>(&'a mut self) -> ConnectionQueue<'a> {
        ConnectionQueue {
            inner: &mut self.inner,
            name: std::borrow::Cow::Borrowed("schedule"),
            redis_type: ConnectionQueueType::SortedSet,
        }
    }

    pub fn dead<'a>(&'a mut self) -> ConnectionQueue<'a> {
        ConnectionQueue {
            inner: &mut self.inner,
            name: std::borrow::Cow::Borrowed("dead"),
            redis_type: ConnectionQueueType::SortedSet,
        }
    }
}

pub struct ConnectionProcess<'a> {
    inner: &'a mut redis::Connection,
    name: &'a str,
}

impl<'a> ConnectionProcess<'a> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn info(&mut self) -> Result<ProcessInfo> {
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

        let mut process: ProcessRaw = serde_redis::from_redis_value(self.inner.hgetall(self.name)?)?;
        let info = std::mem::take(&mut process.info);
        let mut map = if let JsonValue::Object(map) = info { Ok(map) } else { Err("process.info is not a json object") }?;
        map.insert("busy".into(), JsonValue::Number(process.busy.into()));
        map.insert("quiet".into(), JsonValue::Bool(process.quiet));
        map.insert("beat".into(), JsonValue::Number(serde_json::Number::from_f64(process.beat).ok_or("not a f64")?));
        Ok(serde_json::from_value(JsonValue::Object(map))?)
    }

    pub fn workers(&mut self) -> Result<Vec<Worker>> {
        let raw_result: HashMap<String, String> = self.inner.hgetall(format!("{}:workers", self.name))?;
        let result: Result<Vec<Worker>> = raw_result.into_iter().map(|(id, item_str)| {
            let mut item: HashMap<String, JsonValue> = serde_json::from_str(&item_str)?;
            let run_at = item.remove("run_at").ok_or("no run_at")?.as_i64().ok_or("no i64")?;
            let queue = item.remove("queue").ok_or("no queue")?.as_str().ok_or("no str")?.to_string();
            let payload = item.remove("payload").ok_or("no payload")?;
            let job: Job = serde_json::from_str(payload.as_str().ok_or("no str")?)?;
            let worker = Worker { id, run_at, queue, job };
            Ok(worker)
        }).try_collect();
        result
    }
}

enum ConnectionQueueType {
    List,
    SortedSet,
}

pub struct ConnectionQueue<'a> {
    inner: &'a mut redis::Connection,
    name: std::borrow::Cow<'a, str>,
    redis_type: ConnectionQueueType,
}

impl<'a> ConnectionQueue<'a> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn size(&mut self) -> Result<u32> {
        let command = match self.redis_type {
            ConnectionQueueType::List => "LLEN",
            ConnectionQueueType::SortedSet => "ZCARD",
        };
        Ok(redis::cmd(command).arg(&*self.name).query(self.inner)?)
    }

    pub fn jobs(&mut self) -> Result<Vec<Job>> {
        let command = match self.redis_type {
            ConnectionQueueType::List => "LRANGE",
            ConnectionQueueType::SortedSet => "ZRANGE",
        };
        let raw_result: Vec<String> = redis::cmd(command).arg(&*self.name).arg(0).arg(9).query(self.inner)?;
        Ok(raw_result.iter().map(AsRef::as_ref).map(serde_json::from_str).try_collect()?)
    }
}
