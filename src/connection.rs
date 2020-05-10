use crate::protocol::BinaryProtocol;
use crate::stream::Stream;
use anyhow::Result;
use async_std::net::TcpStream;
use mobc::{async_trait, Manager};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use url::Url;

/// A connection to the memcached server
pub struct Connection {
    pub protocol: BinaryProtocol,
    pub url: Arc<String>,
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.protocol
    }
}

impl Deref for Connection {
    type Target = BinaryProtocol;
    fn deref(&self) -> &Self::Target {
        &self.protocol
    }
}

async fn tcp_stream(url: &Url) -> Result<Stream> {
    Ok(Stream::Tcp(
        TcpStream::connect(&*url.socket_addrs(|| None)?).await?,
    ))
}

impl Connection {
    async fn connect(url: &Url) -> Result<Self> {
        let stream = tcp_stream(url).await?;
        let protocol = BinaryProtocol { stream };
        Ok(Connection {
            url: Arc::new(url.to_string()),
            protocol,
        })
    }
}

pub(crate) struct ConnectionManager {
    pub url: Url,
}

#[async_trait]
impl Manager for ConnectionManager {
    type Connection = Connection;
    /// The error type returned by `Connection`s.
    type Error = anyhow::Error;
    /// Attempts to create a new connection.
    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let url = &self.url;
        let connection = Connection::connect(url).await?;
        if url.has_authority() && !url.username().is_empty() && url.password().is_some() {
            // let username = url.username();
            // let password = url.password().unwrap();
            // connection.auth(username, password)?;
        }
        Ok(connection)
    }
    /// Determines if the connection is still connected to the database.
    ///
    /// A standard implementation would check if a simple query like `SELECT 1`
    /// succeeds.
    async fn check(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        let _ = conn.version().await?;
        Ok(conn)
    }
}
