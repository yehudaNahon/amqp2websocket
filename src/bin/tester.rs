use core::time;
use std::thread::{sleep, spawn};

use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
};
use anyhow::{Context, Result};
use log::{debug, info};
use structopt::StructOpt;

fn queue_listener(rabbit: &str, queue_name: &str, secure: bool) -> Result<()> {
    let mut connection = match secure {
        true => Connection::open(rabbit),
        false => Connection::insecure_open(rabbit),
    }
    .context("failed to connect to rabbitmq server")?;

    let channel = connection.open_channel(None).unwrap();

    let queue = channel
        .queue_declare(queue_name, QueueDeclareOptions::default())
        .expect("failed to consume messages from queue");

    let consumer = queue
        .consume(ConsumerOptions::default())
        .expect("failed to consume messages from queue");

    info!("Waiting for messages from queue");

    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                debug!("received new message from queue");
                let message = String::from_utf8(delivery.body.clone());

                if let Ok(message) = message {
                    println!("Received: {}", message);
                }

                consumer.ack(delivery).unwrap();
            }
            other => {
                info!("Stoping because of: {:?}", other);
                break;
            }
        }
    }

    connection
        .close()
        .context("Failed closing rabbit connection")?;

    Ok(())
}

fn queue_pusher(rabbit_address: &str, queue: &str, secure: bool) -> Result<()> {
    // Open connection
    let mut connection = match secure {
        true => Connection::open(rabbit_address),
        false => Connection::insecure_open(rabbit_address),
    }
    .context("Failed opening connection to rabbitmq")?;

    // Open a channnel - None says let the library choose the channel ID.
    let channel = connection
        .open_channel(None)
        .context("Failed opening channel to rabbitmq")?;

    // Get a handle to the direct exchange on out channel
    let exchange = Exchange::direct(&channel);

    let message = "this is a test message to pass to the program";

    loop {
        println!("Publishing: {}", &message);
        // Publish a message to queue.
        exchange
            .publish(Publish::new(&message.as_bytes(), queue))
            .context("Failed publishing to queue")?;

        let time = time::Duration::from_secs(1);
        sleep(time);
    }

    Ok(())
}

#[derive(StructOpt, Debug)]
struct Arguments {
    /// the address of the rabbit server (for example: amqp://guest@localhost:5672)
    rabbit: String,

    /// the name of the queue to send messages to
    in_queue: String,

    /// the name of the queue to listen for incoming messages from
    out_queue: String,

    /// open secure connections
    #[structopt(short, long)]
    secure: bool,
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let args = Arguments::from_args();

    let rabbit = args.rabbit.clone();
    let queue = args.in_queue.clone();
    let secure = args.secure.clone();
    let pusher = spawn(move || queue_pusher(&rabbit, &queue, secure));

    let rabbit = args.rabbit.clone();
    let queue = args.out_queue.clone();
    let secure = args.secure.clone();
    let listener = spawn(move || queue_listener(&rabbit, &queue, secure));

    pusher.join();
    listener.join();
}
