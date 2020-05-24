use sidemon_lib::connection::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let mut connection = Connection::new(&redis_url)?;

    for process_name in connection.process_names()? {
        let mut process = connection.process(&process_name);
        print!("\nprocess ({}):", process.name());
        println!(" {:?}", process.info()?);

        println!("\nworkers ({}):", process_name);
        for worker in process.workers()? {
            println!("- {:?}", worker);
        }
    }

    for queue_name in connection.queue_names()? {
        let mut queue = connection.queue(&queue_name);
        print!("\n{}", queue.name());
        println!(" ({}):", queue.size()?);
        for item in queue.jobs()? {
            println!("- {:?}", item);
        }
    }

    let mut retry = connection.retry();
    println!("\nretry ({}):", retry.size()?);
    for job in retry.jobs()? {
        println!("- {:?}", job);
    }

    let mut schedule = connection.schedule();
    println!("\nschedule ({}):", schedule.size()?);
    for job in schedule.jobs()? {
        println!("- {:?}", job);
    }

    let mut dead = connection.dead();
    println!("\ndead ({}):", dead.size()?);
    for job in dead.jobs()? {
        println!("- {:?}", job);
    }

    Ok(())
}
