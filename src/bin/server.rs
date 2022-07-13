use std::{
    fs::File,
    io::{Read, Write},
    net::TcpListener,
    thread::spawn,
};

use amiquip::{Exchange, Publish};
use amqp2websocket::{accept, connect_to_rabbit};
use anyhow::{bail, Context, Result};
use log::{debug, error, info};
use native_tls::Identity;
use structopt::StructOpt;
use tungstenite::{
    accept_hdr,
    handshake::client::{Request, Response},
    Message,
};

fn handle_client<S: Read + Write>(stream: S, rabbit_address: &str, queue_name: &str) -> Result<()> {
    // a callback function to inspect the header and change the response if needed
    let callback = |req: &Request, response: Response| {
        debug!("Received a new ws handshake");
        debug!("The request's path is: {}", req.uri().path());
        debug!("The request's version is: {:?}", req.version());
        debug!("The request's headers are:");

        for (ref header, value) in response.headers() {
            debug!("* {} :: {:?}", header, value);
        }
        Ok(response)
    };

    let mut websocket = match accept_hdr(stream, callback) {
        Ok(sock) => sock,
        Err(_) => bail!("Failed accepting connection"),
    };

    let mut connection =
        connect_to_rabbit(rabbit_address).context("failed to connect to rabbitmq server")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection
        .open_channel(None)
        .context("Failec opening channel to rabbitmq")?;

    let exchange = Exchange::direct(&channel);

    loop {
        let msg: Message = websocket
            .read_message()
            .context("Failed reading message from websocket")?;

        match msg {
            Message::Text(_) | Message::Binary(_) => {
                debug!("received: new message");

                let data = msg.into_data();

                // Publish a message to queue.
                exchange
                    .publish(Publish::new(&data, queue_name))
                    .context("Failed publishing to queue")?;
            }
            Message::Ping(_) => {
                debug!("Received ping");
                websocket
                    .write_message(Message::Pong(Vec::new()))
                    .context("Failed sending pong message")?;
            }
            Message::Pong(_) => {
                debug!("Received pong");
            }
            Message::Close(_) => {
                info!("master is closing the connection");
                return Ok(());
            }
        }
    }
}

fn run_slave(identity: Identity, port: u32, rabbit: &str, queue_name: &str) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", &port))
        .context("Failed to open TCP connection for listening")?;

    for stream in listener.incoming() {
        let rabbit_address = rabbit.to_string();
        let queue_name = queue_name.to_string();
        let stream = stream.context("Failed opening TCP connection")?;

        let stream = match accept(stream, identity.clone()) {
            Ok(s) => s,
            Err(e) => {
                error!("failed accepting tls connection bcause of error: {}", e);
                continue;
            }
        };

        spawn(move || {
            match handle_client(stream, &rabbit_address, &queue_name) {
                Ok(_) => {}
                Err(e) => error!("Failed handling client with error: {}", e),
            };
        });
    }

    Ok(())
}

#[derive(StructOpt, Debug)]
struct Arguments {
    /// the path to the pfx file to load
    pfx_file: String,

    /// the password too the pfx file
    password: String,

    /// the address of the rabbit server (for example: amqp://guest:guest@localhost:5672)
    rabbit: String,

    /// the name of the queue to listen for incoming messages from
    queue_name: String,

    /// the port to connect
    port: u32,
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let args = Arguments::from_args();

    // open pfx file to load tls from
    let mut file = File::open(args.pfx_file).unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, &args.password).unwrap();

    match run_slave(identity, args.port, &args.rabbit, &args.queue_name) {
        Ok(_) => info!("Closing master"),
        Err(e) => error!("Master exited because of error: {}", e),
    }
}
