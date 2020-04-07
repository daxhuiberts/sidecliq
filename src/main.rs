use client::Client;

mod connection;
mod client;
mod types;

static REDIS_URL: &str = "redis://127.0.0.1/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new(REDIS_URL)?;

    for process_name in client.process_names()? {
        println!("\nprocess ({}): {:?}", process_name, client.process(&process_name)?);

        println!("\nworkers ({}):", process_name);
        for (id, worker) in client.workers(&process_name)? {
            println!("- {}: {:?}", id, worker);
        }
    }

    for queue_name in client.queue_names()? {
        println!("\nqueue ({}):", queue_name);
        for item in client.queue(&queue_name)? {
            println!("- {:?}", item);
        }
    }

    println!("\nretry:");
    for item in client.retry()? {
        println!("- {:?}", item);
    }

    println!("\nschedule:");
    for item in client.schedule()? {
        println!("- {:?}", item);
    }

    println!("\ndead:");
    for item in client.dead()? {
        println!("- {:?}", item);
    }

    Ok(())
}
