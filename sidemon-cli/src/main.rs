use sidemon_lib::client::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let mut client = Client::new(&redis_url)?;

    for process_name in client.process_names()? {
        let mut process = client.process(&process_name);
        println!("\nprocess ({}): {:?}", process_name, process.info()?);

        println!("\nworkers ({}):", process_name);
        for worker in process.workers()? {
            println!("- {:?}", worker);
        }
    }

    for queue_name in client.queue_names()? {
        let mut queue = client.queue(&queue_name);
        println!("\nqueue {} ({}):", queue_name, queue.size()?);
        for item in client.queue(&queue_name).jobs()? {
            println!("- {:?}", item);
        }
    }

    let mut retry = client.retry();
    println!("\nretry ({}):", retry.size()?);
    for job in retry.jobs()? {
        println!("- {:?}", job);
    }

    let mut schedule = client.schedule();
    println!("\nschedule ({}):", schedule.size()?);
    for job in schedule.jobs()? {
        println!("- {:?}", job);
    }

    let mut dead = client.dead();
    println!("\ndead ({}):", dead.size()?);
    for job in dead.jobs()? {
        println!("- {:?}", job);
    }

    Ok(())
}
