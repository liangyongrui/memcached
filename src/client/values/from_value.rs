use crate::Result;
use std::str::FromStr;
/// determine how the value is unserialize to memcached
pub trait FromMemcachedValue: Sized {
    fn from_memcached_value(value:Vec<u8>, flags:u32) -> Result<Self>;
}

pub trait FromMemcachedValueExt: Sized {
    fn from_memcached_value(value: Vec<u8>, flags: u32, cas: Option<u64>) -> Result<Self>;
}

impl<V: FromMemcachedValue> FromMemcachedValueExt for V {
    fn from_memcached_value(value: Vec<u8>, flags: u32, _cas: Option<u64>) -> Result<Self> {
        FromMemcachedValue::from_memcached_value(value, flags)
    }
}

impl FromMemcachedValueExt for (Vec<u8>, u32, Option<u64>) {
    fn from_memcached_value(value: Vec<u8>, flags: u32, cas: Option<u64>) -> Result<Self> {
        Ok((value, flags, cas))
    }
}

impl FromMemcachedValue for (Vec<u8>, u32) {
    fn from_memcached_value(value: Vec<u8>, flags: u32) -> Result<Self> {
        Ok((value, flags))
    }
}

impl FromMemcachedValue for Vec<u8> {
    fn from_memcached_value(value: Vec<u8>, _: u32) -> Result<Self> {
        Ok(value)
    }
}

impl FromMemcachedValue for String {
    fn from_memcached_value(value: Vec<u8>, _: u32) -> Result<Self> {
        Ok(String::from_utf8(value)?)
    }
}

macro_rules! impl_from_memcached_value_for_number {
    ($ty:ident) => {
        impl FromMemcachedValue for $ty {
            fn from_memcached_value(value: Vec<u8>, _: u32) -> Result<Self> {
                let s: String = FromMemcachedValue::from_memcached_value(value, 0)?;
                Ok(Self::from_str(s.as_str())?)
            }
        }
    };
}

impl_from_memcached_value_for_number!(bool);
impl_from_memcached_value_for_number!(u8);
impl_from_memcached_value_for_number!(u16);
impl_from_memcached_value_for_number!(u32);
impl_from_memcached_value_for_number!(u64);
impl_from_memcached_value_for_number!(i8);
impl_from_memcached_value_for_number!(i16);
impl_from_memcached_value_for_number!(i32);
impl_from_memcached_value_for_number!(i64);
impl_from_memcached_value_for_number!(f32);
impl_from_memcached_value_for_number!(f64);