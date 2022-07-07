use amiquip::Connection;
use anyhow::{Result};

#[cfg(feature = "wss")]
use native_tls::{Identity, TlsAcceptor, TlsStream};

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
pub fn accept(stream: TcpStream) -> Result<TlsStream<TcpStream>> {
    use native_tls::{Identity, TlsAcceptor};

    let identity = include_bytes!("yorker.pfx").to_vec();
    let identity = Identity::from_pkcs12(&identity, "yorker")
        .context("Failed retriving identity from pfx file")?;

    let acceptor = TlsAcceptor::new(identity).context("Failed creating tls acceptor")?;


    let stream = acceptor.accept(stream)?;

    Ok(stream)
}

#[cfg(not(feature = "wss"))]
pub fn accept(stream: TcpStream) -> Result<TcpStream> {
    Ok(stream)
}
