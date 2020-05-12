use super::{
    code::{Magic, Opcode},
    parse,
};
use crate::{
    error::{CommandError, MemcachedError, ServerError},
    stream::Stream,
    Result,
};
use byteorder::{BigEndian, ReadBytesExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, io::Cursor};

const OK_STATUS: u16 = 0x0;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub(super) struct PacketHeader {
    pub(super) magic: u8,
    pub(super) opcode: u8,
    pub(super) key_length: u16,
    pub(super) extras_length: u8,
    pub(super) data_type: u8,
    pub(super) vbucket_id_or_status: u16,
    pub(super) total_body_length: u32,
    pub(super) opaque: u32,
    pub(super) cas: u64,
}

#[derive(Debug)]
pub(super) struct StoreExtras {
    pub(super) flags: u32,
    pub(super) expiration: u32,
}

#[derive(Debug)]
pub(super) struct CounterExtras {
    pub(super) amount: u64,
    pub(super) initial_value: u64,
    pub(super) expiration: u32,
}

impl PacketHeader {
    pub(super) async fn write(self, writer: &mut Stream) -> Result<()> {
        writer.write_u8(self.magic).await?;
        writer.write_u8(self.opcode).await?;
        writer.write_u16(self.key_length).await?;
        writer.write_u8(self.extras_length).await?;
        writer.write_u8(self.data_type).await?;
        writer.write_u16(self.vbucket_id_or_status).await?;
        writer.write_u32(self.total_body_length).await?;
        writer.write_u32(self.opaque).await?;
        writer.write_u64(self.cas).await?;
        Ok(())
    }

    pub(super) async fn read(stream: &mut Stream) -> Result<PacketHeader> {
        let magic = stream.read_u8().await?;
        if magic != Magic::Response as u8 {
            return Err(ServerError::BadMagic(magic).into());
        }
        Ok(PacketHeader {
            magic,
            opcode: stream.read_u8().await?,
            key_length: stream.read_u16().await?,
            extras_length: stream.read_u8().await?,
            data_type: stream.read_u8().await?,
            vbucket_id_or_status: stream.read_u16().await?,
            total_body_length: stream.read_u32().await?,
            opaque: stream.read_u32().await?,
            cas: stream.read_u64().await?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Response {
    header: PacketHeader,
    key: Vec<u8>,
    extras: Vec<u8>,
    value: Vec<u8>,
}

impl Response {
    pub(super) fn err(self) -> Result<Self> {
        let status = self.header.vbucket_id_or_status;
        if status == OK_STATUS {
            Ok(self)
        } else {
            Err(CommandError::from(status).into())
        }
    }
}

pub(super) async fn parse_response(stream: &mut Stream) -> Result<Response> {
    let head = PacketHeader::read(stream).await?;
    let mut extras = vec![0x0; head.extras_length as usize];
    stream.read_exact(extras.as_mut_slice()).await?;

    let mut key = vec![0x0; head.key_length as usize];
    stream.read_exact(key.as_mut_slice()).await?;

    let value_len = (head.total_body_length
        - u32::from(head.key_length)
        - u32::from(head.extras_length)) as usize;
    // TODO: return error if total_body_length < extras_length + key_length
    let mut value = vec![0x0; value_len];
    stream.read_exact(&mut value).await?;

    Ok(Response {
        header: head,
        key,
        extras,
        value,
    })
}

pub(super) async fn parse_cas_response(stream: &mut Stream) -> Result<bool> {
    match parse_response(stream).await?.err() {
        Err(MemcachedError::CommandError(e))
            if e == CommandError::KeyNotFound || e == CommandError::KeyExists =>
        {
            Ok(false)
        }
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

pub(super) async fn parse_version_response(stream: &mut Stream) -> Result<String> {
    let Response { value, .. } = parse_response(stream).await?.err()?;
    Ok(parse::deserialize_bytes(&value)?)
}

pub(super) async fn parse_get_response<T: DeserializeOwned + 'static>(
    stream: &mut Stream,
) -> Result<Option<T>> {
    match parse_response(stream).await?.err() {
        Ok(Response { value, .. }) => Ok(Some(parse::deserialize_bytes(&value)?)),
        Err(MemcachedError::CommandError(CommandError::KeyNotFound)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub(super) async fn parse_gets_response<V: DeserializeOwned + 'static>(
    stream: &mut Stream,
    max_responses: usize,
) -> Result<HashMap<String, (V, u32, Option<u64>)>> {
    let mut result = HashMap::new();
    for _ in 0..=max_responses {
        let Response {
            header,
            key,
            extras,
            value,
        } = parse_response(stream).await?.err()?;
        if header.opcode == Opcode::Noop as u8 {
            return Ok(result);
        }
        let flags = Cursor::new(extras).read_u32::<BigEndian>()?;
        let key = parse::deserialize_bytes(&key)?;
        let _ = result.insert(
            key,
            (parse::deserialize_bytes(&value)?, flags, Some(header.cas)),
        );
    }
    Err(ServerError::BadResponse(Cow::Borrowed("Expected end of gets response")).into())
}

pub(super) async fn parse_delete_response(stream: &mut Stream) -> Result<bool> {
    match parse_response(stream).await?.err() {
        Ok(_) => Ok(true),
        Err(MemcachedError::CommandError(CommandError::KeyNotFound)) => Ok(false),
        Err(e) => Err(e),
    }
}

pub(super) async fn parse_counter_response(stream: &mut Stream) -> Result<u64> {
    let Response { value, .. } = parse_response(stream).await?.err()?;
    Ok(Cursor::new(&value).read_u64::<BigEndian>()?)
}

pub(super) async fn parse_touch_response(stream: &mut Stream) -> Result<bool> {
    match parse_response(stream).await?.err() {
        Ok(_) => Ok(true),
        Err(MemcachedError::CommandError(CommandError::KeyNotFound)) => Ok(false),
        Err(e) => Err(e),
    }
}

pub(super) async fn parse_stats_response(stream: &mut Stream) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    loop {
        let Response { key, value, .. } = parse_response(stream).await?.err()?;
        let key: String = parse::deserialize_bytes(&key)?;
        let value: String = parse::deserialize_bytes(&value)?;
        if key.is_empty() && value.is_empty() {
            break;
        }
        let _ = result.insert(key, value);
    }
    Ok(result)
}

pub(super) async fn parse_start_auth_response(stream: &mut Stream) -> Result<bool> {
    parse_response(stream).await?.err().map(|_| true)
}
