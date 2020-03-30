use connection::Connection;

mod connection;
mod types;

static REDIS_URL: &str = "redis://127.0.0.1/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::new(REDIS_URL)?;

    for process_name in conn.process_names()? {
        println!("\nprocess ({}): {:?}", process_name, conn.process(&process_name)?);

        println!("\nworkers ({}):", process_name);
        for (id, worker) in conn.workers(&process_name)? {
            println!("- {}: {:?}", id, worker);
        }
    }

    for queue_name in conn.queue_names()? {
        println!("\nqueue ({}):", queue_name);
        for item in conn.queue(&queue_name)? {
            println!("- {:?}", item);
        }
    }

    println!("\nretry:");
    for item in conn.retry()? {
        println!("- {:?}", item);
    }

    println!("\nschedule:");
    for item in conn.schedule()? {
        println!("- {:?}", item);
    }

    println!("\ndead:");
    for item in conn.dead()? {
        println!("- {:?}", item);
    }

    Ok(())
}
