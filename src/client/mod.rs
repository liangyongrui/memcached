mod check;
mod client_hash;
pub mod connectable;
pub mod value;

use crate::Result;
use crate::{connection::ConnectionManager, protocol::binary_packet::Response};
use client_hash::default_hash_function;
use mobc::Pool;
use url::Url;
use value::ToMemcacheValue;

#[derive(Clone)]
pub struct Client {
    connections: Vec<Pool<ConnectionManager>>,
    pub hash_function: fn(&str) -> usize,
}

impl Client {
    pub fn connect(url: String) -> Result<Self> {
        Self::with_pool_size(url, 1)
    }

    pub fn with_pool_size(url: String, size: u64) -> Result<Self> {
        let urls = vec![url];
        let mut connections = vec![];
        for url in urls {
            let parsed = Url::parse(url.as_str())?;
            let pool = Pool::builder()
                .max_idle(size)
                .build(ConnectionManager { url: parsed });
            connections.push(pool);
        }
        Ok(Client {
            connections,
            hash_function: default_hash_function,
        })
    }

    fn get_connection(&self, key: &str) -> Pool<ConnectionManager> {
        let connections_count = self.connections.len();
        let hash = (self.hash_function)(key) % connections_count;
        self.connections[hash].clone()
    }

    pub async fn version(&self) -> Result<Vec<String>> {
        let mut result = Vec::with_capacity(self.connections.len());
        for connection in self.connections.iter() {
            let mut connection = connection.get().await?;
            result.push(connection.version().await?);
        }
        Ok(result)
    }

    /// Flush all cache on memcached server immediately.
    pub async fn flush(&self) -> Result<()> {
        for connection in self.connections.iter() {
            connection.get().await?.flush().await?;
        }
        Ok(())
    }

    /// Flush all cache on memcached server with a delay seconds.
    pub async fn flush_with_delay(&self, delay: u32) -> Result<()> {
        for connection in self.connections.iter() {
            connection.get().await?.flush_with_delay(delay).await?;
        }
        Ok(())
    }

    /// Get a key from memcached server.
    pub async fn get(&self, key: &str) -> Result<Option<Response>> {
        check::check_key_len(key)?;
        self.get_connection(key).get().await?.get(key).await
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    pub async fn set<V: ToMemcacheValue>(
        &self,
        key: &str,
        value: V,
        expiration: u32,
    ) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .set(key, value, expiration)
            .await
    }

    /// Add a key with associate value into memcached server with expiration seconds.
    pub async fn add<V: ToMemcacheValue>(
        &self,
        key: &str,
        value: V,
        expiration: u32,
    ) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .add(key, value, expiration)
            .await
    }

    /// Replace a key with associate value into memcached server with expiration seconds.
    pub async fn replace<V: ToMemcacheValue>(
        &self,
        key: &str,
        value: V,
        expiration: u32,
    ) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .replace(key, value, expiration)
            .await
    }
}
