use redis::{Commands, Value};
use super::types::Result;

pub struct Connection {
    inner: redis::Connection,
}

impl Connection {
    pub fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(Self { inner: conn })
    }

    pub fn process_names(&mut self) -> Result<Value> {
        Ok(self.inner.smembers("processes")?)
    }

    pub fn process(&mut self, process_name: &str) -> Result<Value> {
        Ok(self.inner.hgetall(process_name)?)
    }

    pub fn workers(&mut self, process_name: &str) -> Result<Value> {
        Ok(self.inner.hgetall(format!("{}:workers", process_name))?)
    }

    pub fn queue_names(&mut self) -> Result<Value> {
        Ok(self.inner.smembers("queues")?)
    }

    pub fn queue(&mut self, queue_name: &str) -> Result<Value> {
        Ok(self.inner.lrange(format!("queue:{}", queue_name), 0, -1)?)
    }

    pub fn retry(&mut self) -> Result<Value> {
        Ok(self.inner.zrange("retry", 0, -1)?)
    }

    pub fn schedule(&mut self) -> Result<Value> {
        Ok(self.inner.zrange("schedule", 0, -1)?)
    }

    pub fn dead(&mut self) -> Result<Value> {
        Ok(self.inner.zrange("dead", 0, -1)?)
    }
}
