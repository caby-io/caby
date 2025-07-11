use base64::prelude::*;
use bitcode::{Decode, Encode};

use crate::Error;

pub type UploadToken = String;

#[derive(Encode, Decode, Debug)]
pub struct TokenFile {
    pub name: String,
    pub size: Option<u64>,
}

#[derive(Encode, Decode, Debug)]
pub struct UploadTokenPayload {
    pub id: String,
    pub chunk_size: u32,
    pub files: Vec<TokenFile>,
}

impl Into<UploadToken> for UploadTokenPayload {
    fn into(self) -> UploadToken {
        BASE64_STANDARD.encode(&bitcode::encode(&self))
    }
}

// struct UploadTokenParseError;

impl TryInto<UploadTokenPayload> for UploadToken {
    type Error = Error;

    fn try_into(self) -> Result<UploadTokenPayload, Self::Error> {
        let bytes = BASE64_STANDARD
            .decode(self.as_bytes())
            .map_err(|_| Error::UploadTokenParseError)?;
        bitcode::decode(&bytes).map_err(|_| Error::UploadTokenParseError)
    }
}
