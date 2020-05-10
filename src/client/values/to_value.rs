use crate::stream::Stream;
use crate::Result;
use mobc::async_trait;

pub enum Flags {
    Bytes = 0,
}

/// determine how the value is serialize to memcache
#[async_trait]
pub trait ToMemcachedValue {
    fn get_flags(&self) -> u32;
    fn get_length(&self) -> usize;
    async fn write_to(&self, stream: &mut Stream) -> Result<()>;
}

#[async_trait]
impl<'a> ToMemcachedValue for &'a [u8] {
    fn get_flags(&self) -> u32 {
        Flags::Bytes as u32
    }

    fn get_length(&self) -> usize {
        self.len()
    }

    async fn write_to(&self, stream: &mut Stream) -> Result<()> {
        stream.write_all(self).await?;
        Ok(())
    }
}

#[async_trait]
impl<'a> ToMemcachedValue for &'a String {
    fn get_flags(&self) -> u32 {
        ToMemcachedValue::get_flags(*self)
    }

    fn get_length(&self) -> usize {
        ToMemcachedValue::get_length(*self)
    }

    async fn write_to(&self, stream: &mut Stream) -> Result<()> {
        ToMemcachedValue::write_to(*self, stream).await
    }
}

#[async_trait]
impl ToMemcachedValue for String {
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
impl<'a> ToMemcachedValue for &'a str {
    fn get_flags(&self) -> u32 {
        Flags::Bytes as u32
    }

    fn get_length(&self) -> usize {
        self.as_bytes().len()
    }

    async fn write_to(&self, stream: &mut Stream) -> Result<()> {
        stream.write_all(self.as_bytes()).await?;
        Ok(())
    }
}

macro_rules! impl_to_memcache_value_for_number {
    ($ty:ident) => {
        #[async_trait]
        impl ToMemcachedValue for $ty {
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
