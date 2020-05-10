pub(crate) mod binary_packet;

use self::binary_packet::{Magic, Opcode, PacketHeader};
use crate::{client::values::FromMemcachedValueExt, stream::Stream, Result};
use std::collections::HashMap;

pub(crate) struct BinaryProtocol {
    pub(crate) stream: Stream,
}

impl BinaryProtocol {
    pub(crate) async fn auth(&mut self, username: &str, password: &str) -> Result<()> {
        let key = "PLAIN";
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::StartAuth as u8,
            key_length: key.len() as u16,
            total_body_length: (key.len() + username.len() + password.len() + 2) as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream
            .write_all(format!("\x00{}\x00{}", username, password).as_bytes())
            .await?;
        self.stream.flush().await?;
        binary_packet::parse_start_auth_response(&mut self.stream)
            .await
            .map(|_| ())
    }
    pub(crate) async fn version(&mut self) -> Result<String> {
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

    pub(crate) async fn flush(&mut self) -> Result<()> {
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
    pub(crate) async fn flush_with_delay(&mut self, delay: u32) -> Result<()> {
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

    pub(crate) async fn get<V: FromMemcachedValueExt>(&mut self, key: &str) -> Result<Option<V>> {
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

    pub(crate) async fn set(&mut self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        self.store(Opcode::Set, key, value, expiration, None).await
    }

    pub(crate) async fn add(&mut self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        self.store(Opcode::Add, key, value, expiration, None).await
    }

    pub(crate) async fn replace(&mut self, key: &str, value: &[u8], expiration: u32) -> Result<()> {
        self.store(Opcode::Replace, key, value, expiration, None)
            .await
    }

    async fn send_request(
        &mut self,
        opcode: Opcode,
        key: &str,
        value: &[u8],
        expiration: u32,
        cas: Option<u64>,
    ) -> Result<()> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: opcode as u8,
            key_length: key.len() as u16,
            extras_length: 8,
            total_body_length: (8 + key.len() + value.len()) as u32,
            cas: cas.unwrap_or(0),
            ..Default::default()
        };
        let extras = binary_packet::StoreExtras {
            flags: 0,
            expiration,
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_u32(extras.flags).await?;
        self.stream.write_u32(extras.expiration).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.write_all(value).await?;
        // value.write_to(&mut self.stream).await?;
        self.stream.flush().await.map_err(Into::into)
    }

    async fn store(
        &mut self,
        opcode: Opcode,
        key: &str,
        value: &[u8],
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

    pub(crate) async fn append(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Append as u8,
            key_length: key.len() as u16,
            total_body_length: (key.len() + value.len()) as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.write_all(value).await?;
        self.stream.flush().await?;
        binary_packet::parse_response(&mut self.stream)
            .await?
            .err()
            .map(|_| ())
    }

    pub(crate) async fn cas(
        &mut self,
        key: &str,
        value: &[u8],
        expiration: u32,
        cas: u64,
    ) -> Result<bool> {
        self.send_request(Opcode::Set, key, value, expiration, Some(cas))
            .await?;
        binary_packet::parse_cas_response(&mut self.stream).await
    }

    pub(crate) async fn prepend(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Prepend as u8,
            key_length: key.len() as u16,
            total_body_length: (key.len() + value.len()) as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.write_all(value).await?;
        self.stream.flush().await?;
        binary_packet::parse_response(&mut self.stream)
            .await
            .map(|_| ())
    }

    pub(crate) async fn delete(&mut self, key: &str) -> Result<bool> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Delete as u8,
            key_length: key.len() as u16,
            total_body_length: key.len() as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.flush().await?;
        binary_packet::parse_delete_response(&mut self.stream).await
    }

    pub(crate) async fn increment(&mut self, key: &str, amount: u64) -> Result<u64> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Increment as u8,
            key_length: key.len() as u16,
            extras_length: 20,
            total_body_length: (20 + key.len()) as u32,
            ..Default::default()
        };
        let extras = binary_packet::CounterExtras {
            amount,
            initial_value: 0,
            expiration: 0,
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_u64(extras.amount).await?;
        self.stream.write_u64(extras.initial_value).await?;
        self.stream.write_u32(extras.expiration).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.flush().await?;
        binary_packet::parse_counter_response(&mut self.stream).await
    }

    pub(crate) async fn decrement(&mut self, key: &str, amount: u64) -> Result<u64> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Decrement as u8,
            key_length: key.len() as u16,
            extras_length: 20,
            total_body_length: (20 + key.len()) as u32,
            ..Default::default()
        };
        let extras = binary_packet::CounterExtras {
            amount,
            initial_value: 0,
            expiration: 0,
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_u64(extras.amount).await?;
        self.stream.write_u64(extras.initial_value).await?;
        self.stream.write_u32(extras.expiration).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.flush().await?;
        binary_packet::parse_counter_response(&mut self.stream).await
    }

    pub(crate) async fn touch(&mut self, key: &str, expiration: u32) -> Result<bool> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Touch as u8,
            key_length: key.len() as u16,
            extras_length: 4,
            total_body_length: (key.len() as u32 + 4),
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.write_u32(expiration).await?;
        self.stream.write_all(key.as_bytes()).await?;
        self.stream.flush().await?;
        binary_packet::parse_touch_response(&mut self.stream).await
    }

    pub(crate) async fn stats(&mut self) -> Result<HashMap<String, String>> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Stat as u8,
            ..Default::default()
        };
        request_header.write(&mut self.stream).await?;
        self.stream.flush().await?;
        let stats_info = binary_packet::parse_stats_response(&mut self.stream).await?;
        Ok(stats_info)
    }

    pub(crate) async fn gets<V: FromMemcachedValueExt>(
        &mut self,
        keys: &[&str],
    ) -> Result<HashMap<String, V>> {
        for key in keys {
            let request_header = PacketHeader {
                magic: Magic::Request as u8,
                opcode: Opcode::GetKQ as u8,
                key_length: key.len() as u16,
                total_body_length: key.len() as u32,
                ..Default::default()
            };
            request_header.write(&mut self.stream).await?;
            self.stream.write_all(key.as_bytes()).await?;
        }
        let noop_request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Noop as u8,
            ..Default::default()
        };
        noop_request_header.write(&mut self.stream).await?;
        binary_packet::parse_gets_response(&mut self.stream, keys.len()).await
    }
}
