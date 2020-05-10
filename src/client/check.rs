use anyhow::Result;

pub(crate) fn check_key_len(key: &str) -> Result<()> {
    if key.len() > 250 {
        Err(anyhow! {"key is too long"})
    } else {
        Ok(())
    }
}
