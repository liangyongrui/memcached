mod check;
mod client_hash;
pub(crate) mod connectable;
pub(crate) mod values;

use crate::connection::ConnectionManager;
use crate::Result;
use client_hash::default_hash_function;
use mobc::Pool;
use std::collections::HashMap;
use url::Url;
use values::FromMemcachedValueExt;
// use values::ToMemcachedValue;

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

    fn get_connection(&self, key: &str) -> &Pool<ConnectionManager> {
        let connections_count = self.connections.len();
        let hash = (self.hash_function)(key) % connections_count;
        self.connections.get(hash).unwrap()
    }

    /// 获取版本号
    /// 
    /// Example
    /// 
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").await.unwrap();
    /// let version = client.version().await.unwrap();
    /// # });
    /// ```
    pub async fn version(&self) -> Result<Vec<String>> {
        let mut result = Vec::with_capacity(self.connections.len());
        for connection in &self.connections {
            let mut connection = connection.get().await?;
            result.push(connection.version().await?);
        }
        Ok(result)
    }

    /// Flush all cache on memcached server immediately.
    pub async fn flush(&self) -> Result<()> {
        for connection in &self.connections {
            connection.get().await?.flush().await?;
        }
        Ok(())
    }

    /// Flush all cache on memcached server with a delay seconds.
    pub async fn flush_with_delay(&self, delay: u32) -> Result<()> {
        for connection in &self.connections {
            connection.get().await?.flush_with_delay(delay).await?;
        }
        Ok(())
    }

    /// Get a key from memcached server.
    pub async fn get<V: FromMemcachedValueExt>(&self, key: &str) -> Result<Option<V>> {
        check::check_key_len(key)?;
        self.get_connection(key).get().await?.get(key).await
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    pub async fn set(&self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .set(key, value, expiration)
            .await
    }

    /// Add a key with associate value into memcached server with expiration seconds.
    pub async fn add(&self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .add(key, value, expiration)
            .await
    }

    /// Replace a key with associate value into memcached server with expiration seconds.
    pub async fn replace(&self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .replace(key, value, expiration)
            .await
    }

    /// Append value to the key.
    pub async fn append(&self, key: &str, value: &[u8]) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .append(key, value)
            .await
    }

    /// Compare and swap a key with the associate value into memcached server with expiration seconds.
    /// `cas_id` should be obtained from a previous `gets` call.
    pub async fn cas(&self, key: &str, value: &[u8], expiration: u32, cas_id: u64) -> Result<bool> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .cas(key, value, expiration, cas_id)
            .await
    }
    /// Prepend value to the key.
    pub async fn prepend(&self, key: &str, value: &[u8]) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .prepend(key, value)
            .await
    }

    /// Delete a key from memcached server.
    pub async fn delete(&self, key: &str) -> Result<bool> {
        check::check_key_len(key)?;
        self.get_connection(key).get().await?.delete(key).await
    }

    /// Increment the value with amount.
    pub async fn increment(&self, key: &str, amount: u64) -> Result<u64> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .increment(key, amount)
            .await
    }

    /// Decrement the value with amount.
    pub async fn decrement(&self, key: &str, amount: u64) -> Result<u64> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .decrement(key, amount)
            .await
    }

    /// Set a new expiration time for a exist key.
    pub async fn touch(&self, key: &str, expiration: u32) -> Result<bool> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .touch(key, expiration)
            .await
    }

    /// Get all servers' statistics.
    pub async fn stats(&self) -> Result<Vec<(String, HashMap<String, String>)>> {
        let mut result: Vec<(String, HashMap<String, String>)> = vec![];
        for connection in &self.connections {
            let mut connection = connection.get().await?;
            let stats_info = connection.stats().await?;
            let url = connection.url.to_string();
            result.push((url, stats_info));
        }
        Ok(result)
    }
    /// Get multiple keys from memcached server. Using this function instead of calling `get` multiple times can reduce netwark workloads.
    pub async fn gets<V: FromMemcachedValueExt>(
        &self,
        keys: &[&str],
    ) -> Result<HashMap<String, V>> {
        for key in keys {
            check::check_key_len(key)?;
        }
        let mut con_keys: HashMap<usize, Vec<&str>> = HashMap::new();
        let mut result: HashMap<String, V> = HashMap::new();
        let connections_count = self.connections.len();

        for key in keys {
            let connection_index = (self.hash_function)(key) % connections_count;
            let array = con_keys.entry(connection_index).or_insert_with(Vec::new);
            array.push(key);
        }
        for (&connection_index, keys) in &con_keys {
            let connection = self.connections.get(connection_index).unwrap();
            result.extend(connection.get().await?.gets(keys).await?);
        }
        Ok(result)
    }
}
