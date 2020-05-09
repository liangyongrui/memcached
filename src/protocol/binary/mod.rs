mod binary_packet;

use self::binary_packet::{Magic, Opcode, PacketHeader};
use super::ProtocolTrait;
use crate::{stream::Stream, Result};
use mobc::async_trait;
pub struct BinaryProtocol {
    pub stream: Stream,
}

#[async_trait]
impl ProtocolTrait for BinaryProtocol {
    async fn version(&mut self) -> Result<String> {
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
}
