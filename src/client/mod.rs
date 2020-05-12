mod check;
mod client_hash;
pub(crate) mod connectable;

use crate::connection::ConnectionManager;
use crate::{error::ClientError, Result};
use client_hash::default_hash_function;
use mobc::Pool;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use url::Url;

/// Client for operating connection pool
#[derive(Clone)]
pub struct Client {
    connections: Vec<Pool<ConnectionManager>>,
    hash_function: fn(&str) -> u64,
}

impl Client {
    /// Create a memcached client instance and connect to memcached server.
    /// The default connection pool has only one connection.    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::Client::connect("memcache://127.0.0.1:12345")?;
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub fn connect(url: &str) -> Result<Self> {
        Self::connects_with(vec![url.to_owned()], 1, default_hash_function)
    }
    /// Create a client, you can specify multiple url, connection pool size, key hash connection pool function.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::Client::connects_with(vec!["memcache://127.0.0.1:12345".to_owned()], 2, |s|1)?;
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub fn connects_with(
        urls: Vec<String>,
        pool_size: u64,
        hash_function: fn(&str) -> u64,
    ) -> Result<Self> {
        let mut connections = vec![];
        for url in urls {
            let parsed = Url::parse(url.as_str())?;
            let pool = Pool::builder()
                .max_idle(pool_size)
                .build(ConnectionManager { url: parsed });
            connections.push(pool);
        }
        if connections.is_empty() {
            return Err(ClientError::ConnectionsIsEmpty.into());
        }
        Ok(Client {
            connections,
            hash_function,
        })
    }

    /// Get server version
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// let version = client.version().await?;
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn version(&self) -> Result<Vec<String>> {
        let mut result = Vec::with_capacity(self.connections.len());
        for connection in &self.connections {
            let mut connection = connection.get().await?;
            result.push(connection.version().await?);
        }
        Ok(result)
    }

    /// Get a value by key
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// let t: Option<String> = client.get("get_none").await?;
    /// assert_eq!(t, None);
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn get<V: DeserializeOwned + 'static>(&self, key: &str) -> Result<Option<V>> {
        check::check_key_len(key)?;
        self.get_connection(key).get().await?.get(key).await
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("abc", "hello", 100).await?;
    /// let t: Option<String> = client.get("abc").await?;
    /// assert_eq!(t, Some("hello".to_owned()));
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn set<V: Serialize + 'static>(
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

    /// Flush all cache on memcached server immediately.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("flush_test", "hello", 100).await?;
    /// client.flush().await?;
    /// let t: Option<String> = client.get("flush_test").await?;
    /// assert_eq!(t, None);
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn flush(&self) -> Result<()> {
        for connection in &self.connections {
            connection.get().await?.flush().await?;
        }
        Ok(())
    }

    /// Flush all cache on memcached server with a delay seconds.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("flush_with_delay_test", "hello", 100).await?;
    /// client.flush_with_delay(2).await?;
    /// let t: Option<String> = client.get("flush_with_delay_test").await?;
    /// assert_eq!(t, Some("hello".to_owned()));
    /// async_std::task::sleep(core::time::Duration::from_secs(2)).await;
    /// let t: Option<String> = client.get("flush_with_delay_test").await?;
    /// assert_eq!(t, None);
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn flush_with_delay(&self, delay: u32) -> Result<()> {
        for connection in &self.connections {
            connection.get().await?.flush_with_delay(delay).await?;
        }
        Ok(())
    }

    /// Add a key with associate value into memcached server with expiration seconds.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.delete("add_test").await?;
    /// client.add("add_test", "hello", 100).await?;
    /// // repeat add KeyExists
    /// client.add("add_test", "hello233", 100).await.unwrap_err();
    /// let t: Option<String> = client.get("add_test").await?;
    /// assert_eq!(t, Some("hello".to_owned()));
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn add<V: Serialize + 'static>(
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
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.delete("replace_test").await?;
    /// // KeyNotFound
    /// client.replace("replace_test", "hello", 100).await.unwrap_err();
    /// client.add("replace_test", "hello", 100).await?;
    /// client.replace("replace_test", "hello233", 100).await?;
    /// let t: Option<String> = client.get("replace_test").await?;
    /// assert_eq!(t, Some("hello233".to_owned()));
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn replace<V: Serialize + 'static>(
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

    /// Append value to the key.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("append_test", "hello", 100).await?;
    /// client.append("append_test", ", 233").await?;
    /// let t: Option<String> = client.get("append_test").await?;
    /// assert_eq!(t, Some("hello, 233".to_owned()));
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn append<V: Serialize + 'static>(&self, key: &str, value: V) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .append(key, value)
            .await
    }
    /// Prepend value to the key.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("prepend_test", "hello", 100).await?;
    /// client.prepend("prepend_test", "233! ").await?;
    /// let t: Option<String> = client.get("prepend_test").await?;
    /// assert_eq!(t, Some("233! hello".to_owned()));
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn prepend<V: Serialize + 'static>(&self, key: &str, value: V) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .prepend(key, value)
            .await
    }

    /// Delete a key from memcached server.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.add("delete_test", "hello", 100).await?;
    /// let t: Option<String> = client.get("delete_test").await?;
    /// assert_eq!(t, Some("hello".to_owned()));
    /// client.delete("delete_test").await?;
    /// let t: Option<String> = client.get("delete_test").await?;
    /// assert_eq!(t, None);
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn delete(&self, key: &str) -> Result<bool> {
        check::check_key_len(key)?;
        self.get_connection(key).get().await?.delete(key).await
    }

    /// Increment the value with amount.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("increment_test", 100, 100).await?;
    /// client.increment("increment_test", 10).await?;
    /// assert_eq!(120, client.increment("increment_test", 10).await.unwrap());
    /// let t: Option<u64> = client.get("increment_test").await?;
    /// assert_eq!(t, Some(120));
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn increment(&self, key: &str, amount: u64) -> Result<u64> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .increment(key, amount)
            .await
    }

    /// Decrement the value with amount.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("decrement_test", 100, 100).await?;
    /// let t = client.decrement("decrement_test", 10).await?;
    /// assert_eq!(80, client.decrement("decrement_test", 10).await.unwrap());
    /// let t: Option<u64> = client.get("decrement_test").await?;
    /// assert_eq!(t.unwrap(), 80);
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn decrement(&self, key: &str, amount: u64) -> Result<u64> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .decrement(key, amount)
            .await
    }

    /// Set a new expiration time for a exist key.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("touch_test", "100", 100).await?;
    /// async_std::task::sleep(core::time::Duration::from_secs(1)).await;
    /// let t: Option<String> = client.get("touch_test").await?;
    /// assert_eq!(t, Some("100".to_owned()));
    /// client.touch("touch_test", 1).await?;
    /// async_std::task::sleep(core::time::Duration::from_secs(1)).await;
    /// let t: Option<String> = client.get("touch_test").await?;
    /// assert_eq!(t, None);
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn touch(&self, key: &str, expiration: u32) -> Result<bool> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .touch(key, expiration)
            .await
    }

    /// Get all servers' statistics.
    /// 暂时不可用
    ///
    /// ## Example
    ///
    /// ```
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// let t = client.stats().await?;
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
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
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("gets_test1", "100", 100).await?;
    /// client.set("gets_test2", "200", 100).await?;
    /// let t = client
    ///    .gets::<String>(&["gets_test1", "gets_test2"])
    ///    .await
    ///    .unwrap();;
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn gets<V: DeserializeOwned + 'static>(
        &self,
        keys: &[&str],
    ) -> Result<HashMap<String, (V, u32, Option<u64>)>> {
        for key in keys {
            check::check_key_len(key)?;
        }
        let mut con_keys: HashMap<usize, Vec<&str>> = HashMap::new();
        let mut result = HashMap::new();
        let connections_count = self.connections.len() as u64;

        for key in keys {
            let connection_index = ((self.hash_function)(key) % connections_count) as usize;
            let array = con_keys.entry(connection_index).or_insert_with(Vec::new);
            array.push(key);
        }
        for (&connection_index, keys) in &con_keys {
            if let Some(connection) = self.connections.get(connection_index) {
                result.extend(connection.get().await?.gets(keys).await?);
            }
        }
        Ok(result)
    }

    /// Compare and swap a key with the associate value into memcached server with expiration seconds.
    /// `cas_id` should be obtained from a previous `gets` call.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async { async fn foo() -> memcached::Result<()> {   
    /// let client = memcached::connect("memcache://127.0.0.1:12345")?;
    /// client.set("cas_test1", "100", 100).await?;
    /// let t = client
    ///     .gets::<String>(&["cas_test1"])
    ///     .await
    ///     ?;
    /// let k = t.get("cas_test1").unwrap();
    /// assert_eq!(&k.0, "100");
    /// let t = client
    ///     .cas("cas_test1", "200", 100, k.2.unwrap() - 1)
    ///     .await
    ///     ?;
    /// let t = client.get::<String>("cas_test1").await?;
    /// assert_eq!(t.unwrap(), "100".to_owned());
    /// let t = client
    ///     .cas("cas_test1", "300", 100, k.2.unwrap())
    ///     .await
    ///     ?;
    /// let t = client.get::<String>("cas_test1").await?;
    /// assert_eq!(t.unwrap(), "300".to_owned());;
    /// # Ok(()) } dbg!(foo().await.unwrap()); });
    /// ```
    pub async fn cas<V: Serialize + 'static>(
        &self,
        key: &str,
        value: V,
        expiration: u32,
        cas_id: u64,
    ) -> Result<bool> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .cas(key, value, expiration, cas_id)
            .await
    }

    /// index < len
    /// 没有风险
    #[allow(clippy::indexing_slicing)]
    fn get_connection(&self, key: &str) -> &Pool<ConnectionManager> {
        let hash = ((self.hash_function)(key) % self.connections.len() as u64) as usize;
        &self.connections[hash]
    }
}
