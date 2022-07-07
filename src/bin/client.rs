use amiquip::{ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use amqp2websocket::connect_to_rabbit;
use anyhow::{Context, Result};
use log::{debug, error, info};
use structopt::StructOpt;
use tungstenite::{connect, Message};
use url::Url;

fn run_master(websocket: &str, rabbit: &str, queue_name: &str) -> Result<()> {
    let (mut socket, response) = connect(
        Url::parse(websocket)
            .context("what the fuck did you put in the websocket params brah?!?!?!?")?,
    )
    .context("Failed to connect to websocket receiver")?;

    info!(
        "Successfully connected to the server with response code {}",
        response.status()
    );

    debug!("response headers:");
    for (ref header, ref value) in response.headers() {
        debug!("* {} :: {:?}", header, value);
    }

    // Open connection to rabbitmq (wether secure to not depends on compilation flags)
    let mut connection =
        connect_to_rabbit(rabbit).context("failed to connect to rabbitmq server")?;

    let channel = connection
        .open_channel(None)
        .context("failed opening channel")?;

    let queue = channel
        .queue_declare(queue_name, QueueDeclareOptions::default())
        .expect("failed to open queue");

    let consumer = queue
        .consume(ConsumerOptions::default())
        .expect("failed to consume messages from queue");

    info!("Waiting for messages. Press Ctrl+C to exit.");

    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                debug!("received new message from queue");

                let data = delivery.body.clone();
                if let Err(_) = socket.write_message(Message::Binary(data)) {
                    error!("failed sending message to receiver... nacking message delivery");
                    if let Err(_) = consumer.nack(delivery, true) {
                        error!("failed nacking delivery, i guess all is fucked");
                    }
                } else {
                    debug!("sent successfully. Acking message");
                    consumer.ack(delivery).unwrap();
                }
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

    socket
        .close(None)
        .context("Failed closing websocket connection")?;

    Ok(())
}

#[derive(StructOpt, Debug)]
struct Arguments {
    /// the address of the receiver to connet to (for example: ws://localhost:3012/socket)
    websocket: String,

    /// the address of the rabbit server (for example: amqp://guest@localhost:5672)
    rabbit: String,

    /// the name of the queue to listen for incoming messages from
    queue_name: String,
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let args = Arguments::from_args();

    match run_master(&args.websocket, &args.rabbit, &args.queue_name) {
        Ok(_) => info!("Closing master"),
        Err(e) => error!("Master exited because of error: {}", e),
    }
}
