use crate::{error::ClientError, Result};

pub(crate) fn check_key_len(key: &str) -> Result<()> {
    if key.len() > 250 {
        Err(ClientError::KeyTooLong.into())
    } else {
        Ok(())
    }
}
