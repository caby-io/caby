use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngExt;

const RANDOM_TOKEN_BYTES: usize = 32;

pub fn random_token() -> String {
    let mut bytes = [0u8; RANDOM_TOKEN_BYTES];
    rand::rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_token_is_43_url_safe_chars() {
        let token = random_token();
        assert_eq!(token.len(), 43);
        assert!(token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
    }

    #[test]
    fn random_tokens_differ() {
        assert_ne!(random_token(), random_token());
    }
}
