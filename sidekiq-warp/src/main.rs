use hyper::server::Server;
use serde::{Deserialize, Serialize};
use sidekiq_lib::client::Client;
use sidekiq_lib::types;
use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use tera::{Context, Tera};
use warp::Filter;

static TEMPLATE_INDEX: &str = include_str!("../templates/index.html");

#[tokio::main]
async fn main() {
    let req = || {
        warp::reply::html(handle_request().unwrap())
    };

    let index = warp::path::end().map(req);
    let service = warp::service(index);
    let make_svc = hyper::service::make_service_fn(|_| async move { Ok::<_, Infallible>(service) });

    let mut listenfd = listenfd::ListenFd::from_env();
    let server = if let Some(listener) = listenfd.take_tcp_listener(0).unwrap() {
        Server::from_tcp(listener).unwrap()
    } else {
        Server::bind(&([127, 0, 0, 1], 3030).into())
    };
    server.serve(make_svc).await.unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct Process {
    info: types::Process,
    workers: HashMap<String, types::JsonValue>,
}

fn handle_request() -> Result<String, Box<dyn Error>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let mut sidekiq = Client::new(&redis_url)?;
    let mut tera = Tera::default();
    tera.add_raw_template("index.html", TEMPLATE_INDEX)?;
    let process_names = sidekiq.process_names()?;
    let processes: HashMap<String, Process> = process_names.into_iter().map(|process_name| {
        let process = sidekiq.process(&process_name).unwrap();
        let workers = sidekiq.workers(&process_name).unwrap();
        (process_name, Process { info: process, workers })
    }).collect();
    let mut context = Context::new();
    context.insert("processes", &processes);
    let rendered = tera.render("index.html", &context)?;
    Ok(rendered)
}
