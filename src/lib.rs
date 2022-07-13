use amiquip::Connection;
use anyhow::Result;

#[cfg(feature = "wss")]
use native_tls::{TlsAcceptor, TlsStream};

use native_tls::Identity;
use std::net::TcpStream;

#[cfg(feature = "amqp-secure")]
pub fn connect_to_rabbit(url: &str) -> Result<Connection, amiquip::Error> {
    Connection::open(url)
}

#[cfg(not(feature = "amqp-secure"))]
pub fn connect_to_rabbit(url: &str) -> Result<Connection, amiquip::Error> {
    Connection::insecure_open(url)
}

#[cfg(feature = "wss")]
pub fn accept(stream: TcpStream, identity: Identity) -> Result<TlsStream<TcpStream>> {
    use anyhow::Context;

    let acceptor = TlsAcceptor::new(identity).context("Failed creating tls acceptor")?;

    let stream = acceptor.accept(stream)?;

    Ok(stream)
}

#[cfg(not(feature = "wss"))]
pub fn accept(stream: TcpStream, _identity: Identity) -> Result<TcpStream> {
    Ok(stream)
}
