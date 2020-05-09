use crate::protocol::binary::BinaryProtocol;
use crate::{protocol::Protocol, stream::Stream};
use anyhow::Result;
use async_std::net::TcpStream;
use mobc::{async_trait, Manager};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};
use url::Url;

/// A connection to the memcached server
pub struct Connection {
    pub protocol: Protocol,
    pub url: Arc<String>,
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.protocol
    }
}

impl Deref for Connection {
    type Target = Protocol;
    fn deref(&self) -> &Self::Target {
        &self.protocol
    }
}

enum Transport {
    Tcp(TcpOptions),
}
impl Transport {
    fn from_url(url: &Url) -> Result<Self> {
        let mut parts = url.scheme().splitn(2, "+");
        match parts.next() {
            Some(part) if part == "memcache" => (),
            _ => {
                return Err(anyhow!(
                    "MemcacheError::BadURL(memcache URL's scheme should start with 'memcache')"
                ))
            }
        }

        // scheme has highest priority
        if let Some(proto) = parts.next() {
            return match proto {
                "tcp" => Ok(Transport::Tcp(TcpOptions::from_url(url))),
                _ => Err(anyhow!("MemcacheError::BadURL")),
            };
        }

        Ok(Transport::Tcp(TcpOptions::from_url(url)))
    }
}
struct TcpOptions {
    timeout: Option<Duration>,
    nodelay: bool,
}
impl TcpOptions {
    fn from_url(url: &Url) -> Self {
        let nodelay = !url
            .query_pairs()
            .any(|(ref k, ref v)| k == "tcp_nodelay" && v == "false");
        let timeout = url
            .query_pairs()
            .find(|&(ref k, ref _v)| k == "timeout")
            .and_then(|(ref _k, ref v)| v.parse::<u64>().ok())
            .map(Duration::from_secs);
        TcpOptions { nodelay, timeout }
    }
}
async fn tcp_stream(url: &Url, opts: &TcpOptions) -> Result<TcpStream> {
    let tcp_stream = TcpStream::connect(&*url.socket_addrs(|| None)?).await?;
    // if opts.timeout.is_some() {
    //     tcp_stream.set_read_timeout(opts.timeout)?;
    //     tcp_stream.set_write_timeout(opts.timeout)?;
    // }
    tcp_stream.set_nodelay(opts.nodelay)?;
    Ok(tcp_stream)
}

impl Connection {
    async fn connect(url: &Url) -> Result<Self> {
        let transport = Transport::from_url(url)?;
        let is_ascii = url
            .query_pairs()
            .any(|(ref k, ref v)| k == "protocol" && v == "ascii");
        let stream: Stream = match transport {
            Transport::Tcp(options) => Stream::Tcp(tcp_stream(url, &options).await?),
        };

        let protocol = Protocol::Binary(BinaryProtocol { stream });

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
    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        todo!()
    }
}
