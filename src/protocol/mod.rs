pub mod binary;

use self::binary::BinaryProtocol;
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use mobc::async_trait;
#[enum_dispatch]
pub enum Protocol {
    Binary(BinaryProtocol),
}

#[async_trait]
#[enum_dispatch(Protocol)]
pub trait ProtocolTrait {
    // fn auth(&mut self, username: &str, password: &str) -> Result<()>;
    async fn version(&mut self) -> Result<String>;
    // fn flush(&mut self) -> Result<()>;
    // fn flush_with_delay(&mut self, delay: u32) -> Result<()>;
    // fn get<V: FromMemcacheValueExt>(&mut self, key: &str) -> Result<Option<V>>;
    // fn gets<V: FromMemcacheValueExt>(&mut self, keys: &[&str]) -> Result<HashMap<String, V>>;
    // fn set<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V, expiration: u32) -> Result<()>;
    // fn cas<V: ToMemcacheValue<Stream>>(
    //     &mut self,
    //     key: &str,
    //     value: V,
    //     expiration: u32,
    //     cas: u64,
    // ) -> Result<bool>;
    // fn add<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V, expiration: u32) -> Result<()>;
    // fn replace<V: ToMemcacheValue<Stream>>(
    //     &mut self,
    //     key: &str,
    //     value: V,
    //     expiration: u32,
    // ) -> Result<()>;
    // fn append<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V) -> Result<()>;
    // fn prepend<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V) -> Result<()>;
    // fn delete(&mut self, key: &str) -> Result<bool>;
    // fn increment(&mut self, key: &str, amount: u64) -> Result<u64>;
    // fn decrement(&mut self, key: &str, amount: u64) -> Result<u64>;
    // fn touch(&mut self, key: &str, expiration: u32) -> Result<bool>;
    // fn stats(&mut self) -> Result<Stats>;
}
