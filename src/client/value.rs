use crate::stream::Stream;
use anyhow::Result;
use mobc::async_trait;

pub enum Flags {
    Bytes = 0,
}

/// determine how the value is serialize to memcache
#[async_trait]
pub trait ToMemcacheValue {
    fn get_flags(&self) -> u32;
    fn get_length(&self) -> usize;
    async fn write_to(&self, stream: &mut Stream) -> Result<()>;
}

#[async_trait]
impl<'a> ToMemcacheValue for &'a [u8] {
    fn get_flags(&self) -> u32 {
        Flags::Bytes as u32
    }

    fn get_length(&self) -> usize {
        self.len()
    }

    async fn write_to(&self, stream: &mut Stream) -> Result<()> {
        match stream.write_all(self).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl<'a> ToMemcacheValue for &'a String {
    fn get_flags(&self) -> u32 {
        ToMemcacheValue::get_flags(*self)
    }

    fn get_length(&self) -> usize {
        ToMemcacheValue::get_length(*self)
    }

    async fn write_to(&self, stream: &mut Stream) -> Result<()> {
        ToMemcacheValue::write_to(*self, stream).await
    }
}

#[async_trait]
impl ToMemcacheValue for String {
    fn get_flags(&self) -> u32 {
        Flags::Bytes as u32
    }

    fn get_length(&self) -> usize {
        self.as_bytes().len()
    }

    async fn write_to(&self, stream: &mut Stream) -> Result<()> {
        match stream.write_all(self.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl<'a> ToMemcacheValue for &'a str {
    fn get_flags(&self) -> u32 {
        Flags::Bytes as u32
    }

    fn get_length(&self) -> usize {
        self.as_bytes().len()
    }

    async fn write_to(&self, stream: &mut Stream) -> Result<()> {
        match stream.write_all(self.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

macro_rules! impl_to_memcache_value_for_number {
    ($ty:ident) => {
        #[async_trait]
        impl ToMemcacheValue for $ty {
            fn get_flags(&self) -> u32 {
                return Flags::Bytes as u32;
            }

            fn get_length(&self) -> usize {
                return self.to_string().as_bytes().len();
            }

            async fn write_to(self: &Self, stream: &mut Stream) -> Result<()> {
                match stream.write_all(self.to_string().as_bytes()).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
        }
    };
}

impl_to_memcache_value_for_number!(bool);
impl_to_memcache_value_for_number!(u8);
impl_to_memcache_value_for_number!(u16);
impl_to_memcache_value_for_number!(u32);
impl_to_memcache_value_for_number!(u64);
impl_to_memcache_value_for_number!(i8);
impl_to_memcache_value_for_number!(i16);
impl_to_memcache_value_for_number!(i32);
impl_to_memcache_value_for_number!(i64);
impl_to_memcache_value_for_number!(f32);
impl_to_memcache_value_for_number!(f64);
