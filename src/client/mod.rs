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
    hash_function: fn(&str) -> usize,
}

impl Client {
    /// 获取连接，默认连接池个数为1
    ///
    /// Example
    ///
    /// ```rust
    /// let client = memcached::Client::connect("memcache://127.0.0.1:12345").unwrap();
    /// ```
    pub fn connect(url: &str) -> Result<Self> {
        Self::connects_with(vec![url.to_owned()], 1, default_hash_function)
    }
    /// 创建一个client，可以指定多个url，连接池大小，key hash连接池的函数
    ///
    /// Example
    ///
    /// ```rust
    /// let client = memcached::Client::connects_with(vec!["memcache://127.0.0.1:12345".to_owned()], 2, |s|1).unwrap();
    /// ```
    pub fn connects_with(
        urls: Vec<String>,
        pool_size: u64,
        hash_function: fn(&str) -> usize,
    ) -> Result<Self> {
        let mut connections = vec![];
        for url in urls {
            let parsed = Url::parse(url.as_str())?;
            let pool = Pool::builder()
                .max_idle(pool_size)
                .build(ConnectionManager { url: parsed });
            connections.push(pool);
        }
        Ok(Client {
            connections,
            hash_function,
        })
    }

    /// 获取版本号
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
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

    /// Get a value by key
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// let t: Option<String> = client.get("abc").await.unwrap();
    /// assert_eq!(t, None);
    /// # });
    /// ```
    pub async fn get<V: FromMemcachedValueExt>(&self, key: &str) -> Result<Option<V>> {
        check::check_key_len(key)?;
        self.get_connection(key).get().await?.get(key).await
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// client.set("abc", b"hello", 100).await.unwrap();
    /// let t: Option<String> = client.get("abc").await.unwrap();
    /// assert_eq!(t, Some("hello".to_owned()));
    /// # });
    /// ```
    pub async fn set(&self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .set(key, value, expiration)
            .await
    }

    /// Flush all cache on memcached server immediately.
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// client.set("flush_test", b"hello", 100).await.unwrap();
    /// client.flush().await.unwrap();
    /// let t: Option<String> = client.get("flush_test").await.unwrap();
    /// assert_eq!(t, None);
    /// # });
    /// ```
    pub async fn flush(&self) -> Result<()> {
        for connection in &self.connections {
            connection.get().await?.flush().await?;
        }
        Ok(())
    }

    /// Flush all cache on memcached server with a delay seconds.
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// client.set("flush_with_delay_test", b"hello", 100).await.unwrap();
    /// client.flush_with_delay(2).await.unwrap();
    /// let t: Option<String> = client.get("flush_with_delay_test").await.unwrap();
    /// assert_eq!(t, Some("hello".to_owned()));
    /// async_std::task::sleep(core::time::Duration::from_secs(2)).await;
    /// let t: Option<String> = client.get("flush_with_delay_test").await.unwrap();
    /// assert_eq!(t, None);
    /// # });
    /// ```
    pub async fn flush_with_delay(&self, delay: u32) -> Result<()> {
        for connection in &self.connections {
            connection.get().await?.flush_with_delay(delay).await?;
        }
        Ok(())
    }

    /// Add a key with associate value into memcached server with expiration seconds.
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// client.add("add_test", b"hello", 100).await.unwrap();
    /// // repeat add KeyExists
    /// client.add("add_test", b"hello233", 100).await.unwrap_err();
    /// let t: Option<String> = client.get("add_test").await.unwrap();
    /// assert_eq!(t, Some("hello".to_owned()));
    /// # });
    /// ```
    pub async fn add(&self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .add(key, value, expiration)
            .await
    }

    /// Replace a key with associate value into memcached server with expiration seconds.
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// // KeyNotFound
    /// client.replace("replace_test", b"hello", 100).await.unwrap_err();
    /// client.add("replace_test", b"hello", 100).await.unwrap();
    /// client.replace("replace_test", b"hello233", 100).await.unwrap();
    /// let t: Option<String> = client.get("replace_test").await.unwrap();
    /// assert_eq!(t, Some("hello233".to_owned()));
    /// # });
    /// ```
    pub async fn replace(&self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .replace(key, value, expiration)
            .await
    }

    /// Append value to the key.
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// client.add("append_test", b"hello", 100).await.unwrap();
    /// client.append("append_test", b", 233").await.unwrap();
    /// let t: Option<String> = client.get("append_test").await.unwrap();
    /// assert_eq!(t, Some("hello, 233".to_owned()));
    /// # });
    /// ```
    pub async fn append(&self, key: &str, value: &[u8]) -> Result<()> {
        check::check_key_len(key)?;
        self.get_connection(key)
            .get()
            .await?
            .append(key, value)
            .await
    }
    /// Prepend value to the key.
    ///
    /// Example
    ///
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
    /// client.add("prepend_test", b"hello", 100).await.unwrap();
    /// client.prepend("prepend_test", b"233! ").await.unwrap();
    /// let t: Option<String> = client.get("prepend_test").await.unwrap();
    /// assert_eq!(t, Some("233! hello".to_owned()));
    /// # });
    /// ```
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

    fn get_connection(&self, key: &str) -> &Pool<ConnectionManager> {
        let connections_count = self.connections.len();
        let hash = (self.hash_function)(key) % connections_count;
        self.connections.get(hash).unwrap()
    }
}
