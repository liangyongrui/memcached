//! bincode 和 memcached 支持的 &[u8] 的转化层
//! bincode 规则：
//! 1. 数字类型视为字符串 直接转化 (为了适配自增功能)
//! 1. 字符串类型前 会 加上字节数 (为了适配追加)
//! 1. 其他暂时未知, 但是统一用bincode 序列化 和 反序列化 理论上不会有问题

use crate::Result;
use byteorder::{ByteOrder, LittleEndian};
use std::{
    any::{Any, TypeId},
    ptr,
};

/// 对于字符串，跳过前8个字节
pub(crate) fn serialize_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize + 'static,
{
    Ok(match parse_as_str(value) {
        Some(s) => bincode::serialize(&s)?.into_iter().skip(8).collect(),
        None => bincode::serialize(value)?,
    })
}

/// 反序列化, 如果可以视为字符串，则用小端表示，把字符串前面增加8个字节, 表示长度
pub(crate) fn deserialize_bytes<T>(bytes: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned + 'static,
{
    Ok(if can_as_str::<T>() {
        let mut buf = Vec::with_capacity(8 + bytes.len());
        buf.append(&mut vec![0; 8]);
        LittleEndian::write_u64(&mut buf, bytes.len() as u64);
        for b in bytes {
            buf.push(*b);
        }
        try_parse_number(&buf)?
    } else {
        bincode::deserialize(bytes)?
    })
}

/// 如果是数字，则返回对于的数字，否反正对应的类型
fn try_parse_number<T>(bytes: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned + 'static,
{
    let t_id = TypeId::of::<T>();
    macro_rules! downcast {
            ($($ty:ty,)*) => {
                $(if t_id == TypeId::of::<$ty>() {
                    let s: String = bincode::deserialize(&bytes)?;
                    let num: $ty = s.trim().parse()?;
                    let p = std::ptr::from_ref::<$ty>(&num).cast::<T>();
                    return Ok(unsafe { ptr::read(p) });
                })*
            };
        }
    downcast![u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64,];
    Ok(bincode::deserialize(bytes)?)
}

/// 判断 value 是否可以视为str
/// 如果 可以 返回 Some(String)
/// 否则 返回 None
fn parse_as_str<T: 'static>(value: &T) -> Option<String> {
    macro_rules! downcast {
        ($($ty:ty,)*) => {
            $(if let Some(t) = <dyn Any>::downcast_ref::<$ty>(value) {
                return Some(t.to_string());
            })*
        };
    }
    downcast![String, &str, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool, f32, f64,];
    None
}
fn can_as_str<T: 'static>() -> bool {
    let t_id = TypeId::of::<T>();
    macro_rules! downcast {
        ($($ty:ty,)*) => {
            $(if t_id == TypeId::of::<$ty>() {
                return true;
            })*
        };
    }
    downcast![String, &str, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool, f32, f64,];
    false
}
