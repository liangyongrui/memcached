use super::code::{Magic, Opcode, OK_STATUS};
use crate::stream::Stream;
use crate::{
    // client::values::FromMemcachedValueExt,
    error::{CommandError, MemcachedError, ServerError},
    Result,
};
use byteorder::ByteOrder;
use byteorder::{BigEndian, ReadBytesExt, LittleEndian};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, io::Cursor};
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct PacketHeader {
    pub(crate) magic: u8,
    pub(crate) opcode: u8,
    pub(crate) key_length: u16,
    pub(crate) extras_length: u8,
    pub(crate) data_type: u8,
    pub(crate) vbucket_id_or_status: u16,
    pub(crate) total_body_length: u32,
    pub(crate) opaque: u32,
    pub(crate) cas: u64,
}

#[derive(Debug)]
pub(crate) struct StoreExtras {
    pub(crate) flags: u32,
    pub(crate) expiration: u32,
}

#[derive(Debug)]
pub(crate) struct CounterExtras {
    pub(crate) amount: u64,
    pub(crate) initial_value: u64,
    pub(crate) expiration: u32,
}

impl PacketHeader {
    pub(crate) async fn write(self, writer: &mut Stream) -> Result<()> {
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

    pub(crate) async fn read(reader: &mut Stream) -> Result<PacketHeader> {
        let magic = reader.read_u8().await?;
        if magic != Magic::Response as u8 {
            return Err(ServerError::BadMagic(magic).into());
        }
        Ok(PacketHeader {
            magic,
            opcode: reader.read_u8().await?,
            key_length: reader.read_u16().await?,
            extras_length: reader.read_u8().await?,
            data_type: reader.read_u8().await?,
            vbucket_id_or_status: reader.read_u16().await?,
            total_body_length: reader.read_u32().await?,
            opaque: reader.read_u32().await?,
            cas: reader.read_u64().await?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Response {
    header: PacketHeader,
    key: Vec<u8>,
    extras: Vec<u8>,
    value: Vec<u8>,
}

impl Response {
    pub(crate) fn err(self) -> Result<Self> {
        let status = self.header.vbucket_id_or_status;
        if status == OK_STATUS {
            Ok(self)
        } else {
            Err(CommandError::from(status).into())
        }
    }
}

pub(crate) async fn parse_response(reader: &mut Stream) -> Result<Response> {
    let header = PacketHeader::read(reader).await?;
    let mut extras = vec![0x0; header.extras_length as usize];
    reader.read_exact(extras.as_mut_slice()).await?;

    let mut key = vec![0x0; header.key_length as usize];
    reader.read_exact(key.as_mut_slice()).await?;

    let value_len = (header.total_body_length
        - u32::from(header.key_length)
        - u32::from(header.extras_length)) as usize;
    // TODO: return error if total_body_length < extras_length + key_length
    // bincode 头8个字节为数据大小
    let mut value = vec![0x0; value_len + 8];
    LittleEndian::write_u64(&mut value[..8], value_len as u64);
    dbg!(value_len, &value);
    reader.read_exact(&mut value[8..]).await?;

    Ok(Response {
        header,
        key,
        extras,
        value,
    })
}

pub(crate) async fn parse_cas_response(reader: &mut Stream) -> Result<bool> {
    match parse_response(reader).await?.err() {
        Err(MemcachedError::CommandError(e))
            if e == CommandError::KeyNotFound || e == CommandError::KeyExists =>
        {
            Ok(false)
        }
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

pub(crate) async fn parse_version_response(reader: &mut Stream) -> Result<String> {
    let Response { value, .. } = parse_response(reader).await?.err()?;
    Ok(String::from_utf8(value)?)
}

pub(crate) async fn parse_get_response<T: DeserializeOwned>(
    reader: &mut Stream,
) -> Result<Option<T>> {
    match parse_response(reader).await?.err() {
        Ok(Response {
            header,
            extras,
            value,
            ..
        }) => {
            let flags = Cursor::new(extras).read_u32::<BigEndian>()?;
            Ok(Some(bincode::deserialize(&value).unwrap()))
        }
        Err(MemcachedError::CommandError(CommandError::KeyNotFound)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub(crate) async fn parse_gets_response<V: DeserializeOwned>(
    reader: &mut Stream,
    max_responses: usize,
) -> Result<HashMap<String, (V, u32, Option<u64>)>> {
    let mut result = HashMap::new();
    for _ in 0..=max_responses {
        let Response {
            header,
            key,
            extras,
            value,
        } = parse_response(reader).await?.err()?;
        if header.opcode == Opcode::Noop as u8 {
            return Ok(result);
        }
        let flags = Cursor::new(extras).read_u32::<BigEndian>()?;
        let key = String::from_utf8(key)?;
        let _ = result.insert(
            key,
            (
                bincode::deserialize(&value).unwrap(),
                flags,
                Some(header.cas),
            ),
        );
    }
    Err(ServerError::BadResponse(Cow::Borrowed("Expected end of gets response")).into())
}

pub(crate) async fn parse_delete_response(reader: &mut Stream) -> Result<bool> {
    match parse_response(reader).await?.err() {
        Ok(_) => Ok(true),
        Err(MemcachedError::CommandError(CommandError::KeyNotFound)) => Ok(false),
        Err(e) => Err(e),
    }
}

pub(crate) async fn parse_counter_response(reader: &mut Stream) -> Result<u64> {
    let Response { value, .. } = parse_response(reader).await?.err()?;
    Ok(Cursor::new(value).read_u64::<BigEndian>()?)
}

pub(crate) async fn parse_touch_response(reader: &mut Stream) -> Result<bool> {
    match parse_response(reader).await?.err() {
        Ok(_) => Ok(true),
        Err(MemcachedError::CommandError(CommandError::KeyNotFound)) => Ok(false),
        Err(e) => Err(e),
    }
}

pub(crate) async fn parse_stats_response(reader: &mut Stream) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    loop {
        let Response { key, value, .. } = parse_response(reader).await?.err()?;
        let key = String::from_utf8(key)?;
        let value = String::from_utf8(value)?;
        if key.is_empty() && value.is_empty() {
            break;
        }
        let _ = result.insert(key, value);
    }
    Ok(result)
}

pub(crate) async fn parse_start_auth_response(reader: &mut Stream) -> Result<bool> {
    parse_response(reader).await?.err().map(|_| true)
}
