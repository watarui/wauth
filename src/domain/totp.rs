use base32::Alphabet;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Totp {
    secret: String,
}

impl Totp {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_code(&self) -> Result<String, anyhow::Error> {
        let decoded_secret = base32::decode(Alphabet::Rfc4648 { padding: false }, &self.secret)
            .ok_or_else(|| anyhow::anyhow!("Invalid secret"))?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 30;

        let msg = timestamp.to_be_bytes();

        let mut mac = Hmac::<Sha1>::new_from_slice(&decoded_secret)?;
        mac.update(&msg);
        let result = mac.finalize().into_bytes();

        let offset = (result[19] & 0xf) as usize;
        let code = ((result[offset] & 0x7f) as u32) << 24
            | (result[offset + 1] as u32) << 16
            | (result[offset + 2] as u32) << 8
            | (result[offset + 3] as u32);

        Ok(format!("{:06}", code % 1_000_000))
    }

    pub fn remaining_seconds() -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        30 - (now % 30)
    }
}
