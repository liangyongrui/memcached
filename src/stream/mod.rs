use crate::Result;
use async_std::io::prelude::{ReadExt, WriteExt};
use async_std::net::TcpStream;
use byteorder::{BigEndian, ByteOrder};

pub(crate) enum Stream {
    Tcp(TcpStream),
}

impl Stream {
    // pub(crate) async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    //     Ok(match self {
    //         Stream::Tcp(ref mut stream) => stream.read(buf).await?,
    //     })
    // }
    pub(crate) async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        match self {
            Stream::Tcp(ref mut stream) => stream.read_exact(buf).await?,
        }
        Ok(())
    }
    // pub(crate) async fn write(&mut self, buf: &[u8]) -> Result<usize> {
    //     Ok(match self {
    //         Stream::Tcp(ref mut stream) => stream.write(buf).await?,
    //     })
    // }
    pub(crate) async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match self {
            Stream::Tcp(ref mut stream) => stream.write_all(buf).await?,
        }
        Ok(())
    }

    pub(crate) async fn flush(&mut self) -> Result<()> {
        match self {
            Stream::Tcp(ref mut stream) => stream.flush().await?,
        }
        Ok(())
    }

    pub(crate) async fn write_u8(&mut self, n: u8) -> Result<()> {
        self.write_all(&[n]).await
    }

    pub(crate) async fn write_u16(&mut self, n: u16) -> Result<()> {
        let mut buf = [0; 2];
        BigEndian::write_u16(&mut buf, n);
        self.write_all(&buf).await
    }

    pub(crate) async fn write_u32(&mut self, n: u32) -> Result<()> {
        let mut buf = [0; 4];
        BigEndian::write_u32(&mut buf, n);
        self.write_all(&buf).await
    }

    pub(crate) async fn write_u64(&mut self, n: u64) -> Result<()> {
        let mut buf = [0; 8];
        BigEndian::write_u64(&mut buf, n);
        self.write_all(&buf).await
    }

    pub(crate) async fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf).await?;
        Ok(buf[0])
    }

    pub(crate) async fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf).await?;
        Ok(BigEndian::read_u16(&buf))
    }

    pub(crate) async fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf).await?;
        Ok(BigEndian::read_u32(&buf))
    }

    pub(crate) async fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf).await?;
        Ok(BigEndian::read_u64(&buf))
    }
}
