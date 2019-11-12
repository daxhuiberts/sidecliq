use connection::Connection;

mod connection;

static REDIS_URL: &str = "redis://127.0.0.1/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::new(REDIS_URL)?;

    for process in conn.processes()? {
        println!("\nprocess ({}): {:#?}", process, conn.process_info(&process)?);

        println!("\nworkers ({}):", process);
        for (id, worker) in conn.workers(&process)? {
            println!("- {}: {:?}", id, worker);
        }
    }

    for queue in conn.queues()? {
        println!("\nqueue ({}):", queue);
        for item in conn.queue(&queue)? {
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
