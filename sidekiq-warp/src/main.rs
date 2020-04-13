use sidekiq_lib::client::Client;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use warp::Filter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let client = Client::new(&redis_url)?;
    let client = Arc::new(Mutex::new(client));
    let req = move || {
        client.lock().unwrap().process_names().map(|names| format!("{:?}", names)).unwrap_or("error".to_owned())
    };
    let index = warp::path::end().map(req);
    let server = warp::serve(index).run(([127, 0, 0, 1], 3030));
    let mut runtime = Runtime::new()?;
    runtime.block_on(server);
    Ok(())
}
