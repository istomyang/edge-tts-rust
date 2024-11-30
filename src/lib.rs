use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

pub mod tts;
pub mod voice;

/// Generate Sec-MS-GEC token.
///
/// Use algo from: https://github.com/rany2/edge-tts/issues/290#issuecomment-2464956570
pub(crate) fn gen_sec_ms_gec() -> String {
    let sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let sec = sec + 11644473600;
    let sec = sec - (sec % 300);
    let nsec = sec * 1_000_000_000 / 100;
    let str = format!("{}6A5AA1D4EAFF4E9FB37E23D68491D6F4", nsec);
    let mut hasher = Sha256::new();
    hasher.update(str.as_bytes());
    format!("{:x}", hasher.finalize()).to_uppercase()
}

#[test]
fn test_gen_sec_ms_gec() {
    println!("{}", gen_sec_ms_gec());
}

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.msg)
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self { msg }
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {}
