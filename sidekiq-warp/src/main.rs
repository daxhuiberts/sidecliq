use serde::{Deserialize, Serialize};
use sidekiq_lib::client::Client;
use sidekiq_lib::types::{JsonValue, Process};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};
use tokio::runtime::Runtime;
use warp::Filter;

static TEMPLATE_INDEX: &str = include_str!("../templates/index.html");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let sidekiq = Client::new(&redis_url)?;
    let mut tera = Tera::default();
    tera.add_raw_template("index.html", TEMPLATE_INDEX)?;
    let context = Arc::new(Mutex::new((sidekiq, tera)));

    let req = move || {
        let (sidekiq, tera) = &mut *context.lock().unwrap();
        let result = handle_request(sidekiq, tera).unwrap();
        warp::reply::html(result)
    };

    let index = warp::path::end().map(req);
    let server = warp::serve(index).run(([127, 0, 0, 1], 3030));
    let mut runtime = Runtime::new()?;
    runtime.block_on(server);
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct ProcessInfo {
    process: Process,
    workers: HashMap<String, JsonValue>,
}

fn handle_request(sidekiq: &mut Client, tera: &mut Tera) -> Result<String, Box<dyn Error>> {
    let process_names = sidekiq.process_names()?;
    let processes: HashMap<String, ProcessInfo> = process_names.into_iter().map(|process_name| {
        let process = sidekiq.process(&process_name).unwrap();
        let workers = sidekiq.workers(&process_name).unwrap();
        (process_name, ProcessInfo { process, workers })
    }).collect();
    let mut context = Context::new();
    context.insert("processes", &processes);
    let rendered = tera.render("index.html", &context)?;
    Ok(rendered)
}
