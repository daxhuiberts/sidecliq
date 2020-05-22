use sidemon_lib::client::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let mut client = Client::new(&redis_url)?;

    for process_name in client.process_names()? {
        println!("\nprocess ({}): {:?}", process_name, client.process(&process_name)?);

        println!("\nworkers ({}):", process_name);
        for (id, worker) in client.workers(&process_name)? {
            println!("- {}: {:?}", id, worker);
        }
    }

    for queue_name in client.queue_names()? {
        println!("\nqueue ({}):", queue_name);
        for item in client.queue(&queue_name).jobs()? {
            println!("- {:?}", item);
        }
    }

    println!("\nretry:");
    for item in client.retry().jobs()? {
        println!("- {:?}", item);
    }

    println!("\nschedule:");
    for item in client.schedule().jobs()? {
        println!("- {:?}", item);
    }

    println!("\ndead:");
    for item in client.dead().jobs()? {
        println!("- {:?}", item);
    }

    Ok(())
}
