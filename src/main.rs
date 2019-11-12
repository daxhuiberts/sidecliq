use std::collections::HashMap;
use redis::Commands;

static REDIS_URL: &str = "redis://127.0.0.1/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open(REDIS_URL)?;
    let mut connection = client.get_connection()?;

    let processes = connection.smembers::<_, Vec<String>>("processes")?;
    println!("processes: {:#?}", processes);

    for process_name in &processes {
        let process = connection.hgetall::<_, HashMap<String, String>>(process_name)?;
        println!("\nprocess ({}): {:#?}", process_name, process);

        let workers = connection.hgetall::<_, HashMap<String, String>>(format!("{}:workers", process_name))?;
        println!("\nworkers ({}):", process_name);
        for (id, worker) in &workers {
            let record: serde_json::Value = serde_json::from_str(worker)?;
            println!("- {}: {:?}", id, record);
        }
    }

    let queues = connection.smembers::<_, Vec<String>>("queues")?;
    println!("\nqueues: {:#?}", queues);

    for queue_name in &queues {
        let queue = connection.lrange::<_, Vec<String>>(format!("queue:{}", queue_name), 0, -1)?;
        println!("\nqueue ({}):", queue_name);
        for item in &queue {
            let record: serde_json::Value = serde_json::from_str(item)?;
            println!("- {:?}", record);
        }
    }

    let retry = connection.zrange::<_, Vec<String>>("retry", 0, -1)?;
    println!("\nretry:");
    for item in &retry {
        let record: serde_json::Value = serde_json::from_str(item)?;
        println!("- {:?}", record);
    }

    let schedule = connection.zrange::<_, Vec<String>>("schedule", 0, -1)?;
    println!("\nschedule:");
    for item in &schedule {
        let record: serde_json::Value = serde_json::from_str(item)?;
        println!("- {:?}", record);
    }

    let dead = connection.zrange::<_, Vec<String>>("dead", 0, -1)?;
    println!("\ndead:");
    for item in &dead {
        let record: serde_json::Value = serde_json::from_str(item)?;
        println!("- {:?}", record);
    }

    Ok(())
}
