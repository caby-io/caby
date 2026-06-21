use std::path::Path;

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct MediaType(String);

impl MediaType {
    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        mime_guess::from_path(path)
            .first_raw()
            .map(|m| Self(m.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const PREVIEWABLE_MIMES: &[&str] = &[
    // images
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/avif",
    // video — browser-renderable codecs only
    "video/mp4",
    "video/webm",
    // audio
    "audio/mpeg",
    "audio/mp4",
    "audio/ogg",
    "audio/wav",
    "audio/webm",
];

#[derive(Serialize, PartialEq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Image,
    Video,
    Audio,
    Pdf,
    Archive,
    Document,
    #[default]
    Other,
}

impl FileKind {
    pub fn can_preview(&self, mime: Option<&MediaType>) -> bool {
        if !matches!(self, Self::Image | Self::Video | Self::Audio) {
            return false;
        }
        let Some(m) = mime else { return false };
        PREVIEWABLE_MIMES.contains(&m.as_str())
    }
}

impl From<&MediaType> for FileKind {
    fn from(media_type: &MediaType) -> Self {
        let mime = media_type.0.as_str();

        if let Some((top, _)) = mime.split_once('/') {
            match top {
                "image" => return Self::Image,
                "video" => return Self::Video,
                "audio" => return Self::Audio,
                "text" => return Self::Document,
                _ => {}
            }
        }

        match mime {
            "application/pdf" => Self::Pdf,
            "application/zip"
            | "application/gzip"
            | "application/x-tar"
            | "application/x-7z-compressed"
            | "application/x-rar-compressed"
            | "application/x-bzip2" => Self::Archive,
            "application/msword"
            | "application/rtf"
            | "application/vnd.oasis.opendocument.text"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                Self::Document
            }
            _ => Self::Other,
        }
    }
}
