use crate::{error::MemcachedError, protocol::BinaryProtocol, stream::Stream, Result};
use async_std::net::TcpStream;
use mobc::{async_trait, Manager};
use std::ops::{Deref, DerefMut};
use url::Url;

/// A connection to the memcached server
pub(crate) struct Connection {
    pub(crate) protocol: BinaryProtocol,
    pub(crate) url: String,
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
            url: url.to_string(),
            protocol,
        })
    }
}
#[derive(Debug)]
pub(crate) struct ConnectionManager {
    pub(crate) url: Url,
}

#[async_trait]
impl Manager for ConnectionManager {
    type Connection = Connection;
    /// The error type returned by `Connection`s.
    type Error = MemcachedError;
    /// Attempts to create a new connection.
    async fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let url = &self.url;
        let mut connection = Connection::connect(url).await?;
        if url.has_authority() && !url.username().is_empty() && url.password().is_some() {
            let username = url.username();
            let password = url.password().unwrap_or("");
            connection.auth(username, password).await?;
        }
        Ok(connection)
    }
    /// Determines if the connection is still connected to the database.
    ///
    /// A standard implementation would check if a simple query like `SELECT 1`
    /// succeeds.
    async fn check(
        &self,
        mut conn: Self::Connection,
    ) -> std::result::Result<Self::Connection, Self::Error> {
        let _ = conn.version().await?;
        Ok(conn)
    }
}
