pub mod binary_packet;

use self::binary_packet::{Magic, Opcode, PacketHeader};
use crate::{client::value::ToMemcacheValue, stream::Stream, Result};
use binary_packet::Response;
pub struct BinaryProtocol {
    pub stream: Stream,
}

impl BinaryProtocol {
    pub async fn version(&mut self) -> Result<String> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Version as u8,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.flush().await?;
        let version = binary_packet::parse_version_response(&mut self.stream).await?;
        Ok(version)
    }

    pub async fn flush(&mut self) -> Result<()> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Flush as u8,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.flush().await?;
        binary_packet::parse_response(&mut self.stream)
            .await?
            .err()
            .map(|_| ())
    }

    /// Flush all cache on memcached server with a delay seconds.
    pub async fn flush_with_delay(&mut self, delay: u32) -> Result<()> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Flush as u8,
            extras_length: 4,
            total_body_length: 4,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_u32(delay).await?;
        self.stream.flush().await?;
        binary_packet::parse_response(&mut self.stream)
            .await?
            .err()
            .map(|_| ())
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<Response>> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Get as u8,
            key_length: key.len() as u16,
            total_body_length: key.len() as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.flush().await?;
        binary_packet::parse_get_response(&mut self.stream).await
    }

    pub async fn set<V: ToMemcacheValue>(
        &mut self,
        key: &str,
        value: V,
        expiration: u32,
    ) -> Result<()> {
        self.store(Opcode::Set, key, value, expiration, None).await
    }

    pub async fn add<V: ToMemcacheValue>(
        &mut self,
        key: &str,
        value: V,
        expiration: u32,
    ) -> Result<()> {
        self.store(Opcode::Add, key, value, expiration, None).await
    }

    pub async fn replace<V: ToMemcacheValue>(
        &mut self,
        key: &str,
        value: V,
        expiration: u32,
    ) -> Result<()> {
        self.store(Opcode::Replace, key, value, expiration, None)
            .await
    }

    async fn send_request<T: ToMemcacheValue>(
        &mut self,
        opcode: Opcode,
        key: &str,
        value: T,
        expiration: u32,
        cas: Option<u64>,
    ) -> Result<()> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: opcode as u8,
            key_length: key.len() as u16,
            extras_length: 8,
            total_body_length: (8 + key.len() + value.get_length()) as u32,
            cas: cas.unwrap_or(0),
            ..Default::default()
        };
        let extras = binary_packet::StoreExtras {
            flags: value.get_flags(),
            expiration,
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_u32(extras.flags).await?;
        self.stream.write_u32(extras.expiration).await?;
        self.stream.write_all(key.as_bytes()).await?;
        value.write_to(&mut self.stream).await?;
        self.stream.flush().await.map_err(Into::into)
    }

    async fn store<V: ToMemcacheValue>(
        &mut self,
        opcode: Opcode,
        key: &str,
        value: V,
        expiration: u32,
        cas: Option<u64>,
    ) -> Result<()> {
        self.send_request(opcode, key, value, expiration, cas)
            .await?;
        binary_packet::parse_response(&mut self.stream)
            .await?
            .err()
            .map(|_| ())
    }
}
