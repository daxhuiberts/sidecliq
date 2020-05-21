use hyper::server::Server;
use sidekiq_lib::client::Client;
use sidekiq_lib::types;
use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use tera::{Context, Tera};
use warp::Filter;

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
        Server::bind(&([127, 0, 0, 1], 3000).into())
    };
    server.serve(make_svc).await.unwrap();
}

fn handle_request() -> Result<String, Box<dyn Error>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let mut sidekiq = Client::new(&redis_url)?;
    let tera = get_tera_instance()?;
    let context = sidekiq_data(&mut sidekiq)?;
    let rendered = tera.render("index.html", &context)?;
    Ok(rendered)
}

fn sidekiq_data(client: &mut Client) -> Result<tera::Context, Box<dyn Error>> {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Deserialize, Serialize)]
    struct Process {
        info: types::Process,
        workers: HashMap<String, types::Job>,
    }

    let mut context = Context::new();

    let process_names = client.process_names()?;
    let processes: HashMap<String, Process> = process_names.into_iter().map(|process_name| {
        let process = client.process(&process_name).unwrap();
        let workers = client.workers(&process_name).unwrap();
        (process_name, Process { info: process, workers })
    }).collect();
    context.insert("processes", &processes);

    let queue_names = client.queue_names()?;
    let queues: HashMap<String, Vec<types::Job>> = queue_names.into_iter().map(|queue_name| {
        let queue = client.queue(&queue_name).unwrap();
        (queue_name, queue)
    }).collect();
    context.insert("queues", &queues);

    context.insert("retry", &client.retry()?);
    context.insert("schedule", &client.schedule()?);
    context.insert("dead", &client.dead()?);

    Ok(context)
}

#[cfg(feature = "dynamic_templates")]
fn get_tera_instance() -> Result<Tera, Box<dyn Error>> {
    Ok(Tera::new("templates/*")?)
}

#[cfg(not(feature = "dynamic_templates"))]
fn get_tera_instance() -> Result<Tera, Box<dyn Error>> {
    static TEMPLATE_INDEX: &str = include_str!("../templates/index.html");
    static TEMPLATE_JOBS: &str = include_str!("../templates/jobs.html");
    let mut tera = Tera::default();
    tera.add_raw_template("index.html", TEMPLATE_INDEX)?;
    tera.add_raw_template("jobs.html", TEMPLATE_JOBS)?;
    Ok(tera)
}
