use redis::Commands;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Connection {
    inner: redis::Connection,
}

impl Connection {
    pub fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(Self { inner: conn })
    }

    pub fn processes(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers::<_, Vec<String>>("processes")?)
    }

    pub fn process_info(&mut self, process_name: &str) -> Result<HashMap<String, String>> {
        Ok(self.inner.hgetall::<_, HashMap<String, String>>(process_name)?)
    }

    pub fn workers(&mut self, process_name: &str) -> Result<HashMap<String, JsonValue>> {
        let workers = self.inner.hgetall::<_, HashMap<String, String>>(format!("{}:workers", process_name))?;
        let result: std::result::Result<_, serde_json::error::Error> = workers.into_iter().map(|(id, worker)| Ok((id, serde_json::from_str(&worker)?)) ).collect();
        Ok(result?)
    }

    pub fn queues(&mut self) -> Result<Vec<String>> {
        Ok(self.inner.smembers::<_, Vec<String>>("queues")?)
    }

    pub fn queue(&mut self, queue_name: &str) -> Result<Vec<JsonValue>> {
        let queue = self.inner.lrange::<_, Vec<String>>(format!("queue:{}", queue_name), 0, -1)?;
        let result: std::result::Result<_, serde_json::error::Error> = queue.into_iter().map(|item| serde_json::from_str(&item) ).collect();
        Ok(result?)
    }

    pub fn retry(&mut self) -> Result<Vec<JsonValue>> {
        let retry = self.inner.zrange::<_, Vec<String>>("retry", 0, -1)?;
        let result: std::result::Result<_, serde_json::error::Error> = retry.into_iter().map(|item| serde_json::from_str(&item) ).collect();
        Ok(result?)
    }

    pub fn schedule(&mut self) -> Result<Vec<JsonValue>> {
        let schedule = self.inner.zrange::<_, Vec<String>>("schedule", 0, -1)?;
        let result: std::result::Result<_, serde_json::error::Error> = schedule.into_iter().map(|item| serde_json::from_str(&item) ).collect();
        Ok(result?)
    }

    pub fn dead(&mut self) -> Result<Vec<JsonValue>> {
        let dead = self.inner.zrange::<_, Vec<String>>("dead", 0, -1)?;
        let result: std::result::Result<_, serde_json::error::Error> = dead.into_iter().map(|item| serde_json::from_str(&item) ).collect();
        Ok(result?)
    }
}
