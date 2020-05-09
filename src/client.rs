use crate::connection::ConnectionManager;
use crate::protocol::ProtocolTrait;
use anyhow::Result;
use mobc::Pool;
use std::{
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    hash::{Hash, Hasher},
};
use url::Url;

pub trait Connectable {
    fn get_urls(self) -> Vec<String>;
}

#[derive(Clone)]
pub struct Client {
    connections: Vec<Pool<ConnectionManager>>,
    pub hash_function: fn(&str) -> usize,
}

fn default_hash_function(key: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish().try_into().unwrap()
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
            let mut connection = connection.get().await.map_err(|e| anyhow!(e))?;
            result.push(connection.version().await?);
        }
        Ok(result)
    }
}
