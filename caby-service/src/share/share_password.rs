use anyhow::anyhow;
use argon2::{Argon2, PasswordVerifier};

use crate::{user::try_hash_password, Result};

use super::ShareAuth;

pub(crate) enum HashFormat {
    Argon2,
    Bcrypt,
    Sha512Crypt,
}

pub(crate) fn hash_format(hash: &str) -> Option<HashFormat> {
    if hash.starts_with("$argon2") {
        Some(HashFormat::Argon2)
    } else if hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        Some(HashFormat::Bcrypt)
    } else if hash.starts_with("$6$") {
        Some(HashFormat::Sha512Crypt)
    } else {
        None
    }
}

fn verify_hash(hash: &str, plaintext: &str) -> Result<bool> {
    match hash_format(hash) {
        Some(HashFormat::Argon2) => {
            let parsed = argon2::PasswordHash::new(hash)
                .map_err(|err| anyhow!("could not parse argon2 hash: {}", err))?;
            Ok(Argon2::default()
                .verify_password(plaintext.as_bytes(), &parsed)
                .is_ok())
        }
        Some(HashFormat::Bcrypt) => bcrypt::verify(plaintext, hash)
            .map_err(|err| anyhow!("could not verify bcrypt hash: {}", err)),
        Some(HashFormat::Sha512Crypt) => Ok(sha_crypt::sha512_check(plaintext, hash).is_ok()),
        None => Err(anyhow!("unsupported password hash format")),
    }
}

impl ShareAuth {
    pub fn password(plaintext: &str) -> Result<Self> {
        Ok(Self::Password {
            hash: try_hash_password(plaintext)?,
        })
    }

    pub fn try_verify(&self, plaintext: &str) -> Result<bool> {
        match self {
            Self::Open => Ok(true),
            Self::Password { hash } => verify_hash(hash, plaintext),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_accepts_argon2_bcrypt_and_sha512crypt() {
        // each is a hash of "sharepass" in its respective format
        for hash in [
            "$argon2id$v=19$m=19456,t=2,p=1$2wZGyG/UHzKi38a9Pydu2A$yPXx0OjYTOCjoKmSCfEKGBuoNkV6dClbHyYzyNzSGbY",
            "$2b$10$uy7PcJEUHqLpzk9H.QE0JeiCx0cs5MLXlyOdMkLnOwns1xqJPMcUK",
            "$6$abcd1234$thjHjXmLs1exuU707Xf57m1TIjqLDWtvxoVWS2L1S.m59iBJZlLujLGZxg24M/u3qEy2PDnqsYmT8l8PlziHq1",
        ] {
            let auth = ShareAuth::Password {
                hash: hash.to_owned(),
            };
            assert!(auth.try_verify("sharepass").unwrap(), "should accept: {hash}");
            assert!(!auth.try_verify("wrong").unwrap(), "should reject wrong pw: {hash}");
        }
    }

    #[test]
    fn verify_rejects_unsupported_hash_format() {
        assert!(hash_format("$argon2id$v=19$m=1$x$y").is_some());
        assert!(hash_format("$2b$10$abc").is_some());
        assert!(hash_format("$6$salt$hash").is_some());
        assert!(hash_format("plaintext").is_none());
        // bare sha256 hex is easy to make but unsalted/fast — not accepted
        let sha256hex = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";
        assert!(hash_format(sha256hex).is_none());
        let auth = ShareAuth::Password {
            hash: sha256hex.to_owned(),
        };
        assert!(auth.try_verify("sharepass").is_err());
    }
}
