use std::{
    str::FromStr,
    time::{Duration, SystemTime},
};

use tokio::fs;
use tracing::{debug, info, warn};

use crate::{
    auth::Token as SessionToken,
    config::Config,
    download::Token as DownloadToken,
    upload::{decode_upload_token, manifest, UPLOAD_TOKEN_LIFETIME_HOURS},
    Result,
};

enum UserFile {
    Session,
    Download,
}

impl UserFile {
    fn from_filename(name: &str) -> Option<Self> {
        if name.starts_with("session_") {
            Some(Self::Session)
        } else if name.starts_with("download_") {
            Some(Self::Download)
        } else {
            None
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Download => "download",
        }
    }
}

pub async fn housekeeping(cfg: &Config) -> Result<()> {
    debug!("starting housekeeping...");

    let mut sessions_removed: u32 = 0;
    let mut download_tokens_removed: u32 = 0;
    let mut errors: Vec<String> = vec![];

    for (_, user) in cfg.users.iter() {
        let mut dir = match fs::read_dir(&user.path).await {
            Ok(d) => d,
            Err(err) => {
                // this is a debug because an uninitilized user will have no user dir
                debug!("could not read user directory for {}: {}", user.name, err);
                continue;
            }
        };

        while let Ok(Some(entry)) = dir.next_entry().await {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };

            let Some(kind) = UserFile::from_filename(name) else {
                continue;
            };

            let content = match fs::read_to_string(entry.path()).await {
                Ok(c) => c,
                Err(err) => {
                    errors.push(format!(
                        "could not read {} file {}/{}: {:#}",
                        kind.label(),
                        user.name,
                        name,
                        err
                    ));
                    continue;
                }
            };

            let is_expired = match kind {
                UserFile::Session => match SessionToken::from_str(&content) {
                    Ok(t) => t.is_expired(),
                    Err(err) => {
                        errors.push(format!(
                            "could not parse session file {}/{}: {:#}",
                            user.name, name, err
                        ));
                        continue;
                    }
                },
                UserFile::Download => match DownloadToken::from_str(&content) {
                    Ok(t) => t.is_expired(),
                    Err(err) => {
                        errors.push(format!(
                            "could not parse download file {}/{}: {:#}",
                            user.name, name, err
                        ));
                        continue;
                    }
                },
            };

            if !is_expired {
                continue;
            }

            if let Err(err) = fs::remove_file(entry.path()).await {
                errors.push(format!(
                    "could not remove expired {} file {}/{}: {:#}",
                    kind.label(),
                    user.name,
                    name,
                    err
                ));
                continue;
            }

            match kind {
                UserFile::Session => sessions_removed += 1,
                UserFile::Download => download_tokens_removed += 1,
            }
        }
    }

    let mut uploads_removed: u32 = 0;

    for (_, space_config) in cfg.spaces.iter() {
        let uploads_dir = space_config.path.join("uploads");
        let mut dir = match fs::read_dir(&uploads_dir).await {
            Ok(d) => d,
            Err(err) => {
                debug!(
                    "could not read uploads dir for space {}: {}",
                    space_config.name, err
                );
                continue;
            }
        };

        while let Ok(Some(entry)) = dir.next_entry().await {
            let upload_path = entry.path();
            let upload_name = entry.file_name().to_string_lossy().into_owned();

            let metadata = match entry.metadata().await {
                Ok(m) => m,
                Err(err) => {
                    errors.push(format!(
                        "could not stat upload {}/{}: {:#}",
                        space_config.name, upload_name, err
                    ));
                    continue;
                }
            };
            if !metadata.is_dir() {
                continue;
            }

            let token_expired = match manifest::read(&upload_path).await {
                Ok(m) => decode_upload_token(cfg, &m.token).map(|p| p.is_expired()),
                Err(err) => Err(err),
            };

            let expired = match token_expired {
                Ok(e) => e,
                Err(err) => {
                    debug!(
                        "falling back to dir created_at for upload {}/{}: {:#}",
                        space_config.name, upload_name, err
                    );
                    match metadata.created() {
                        Ok(created) => {
                            let age = SystemTime::now()
                                .duration_since(created)
                                .unwrap_or(Duration::ZERO);
                            age > Duration::from_secs(UPLOAD_TOKEN_LIFETIME_HOURS as u64 * 3600)
                        }
                        Err(err) => {
                            errors.push(format!(
                                "could not determine upload age for {}/{}: {:#}",
                                space_config.name, upload_name, err
                            ));
                            continue;
                        }
                    }
                }
            };

            if !expired {
                continue;
            }

            if let Err(err) = fs::remove_dir_all(&upload_path).await {
                errors.push(format!(
                    "could not remove expired upload {}/{}: {:#}",
                    space_config.name, upload_name, err
                ));
                continue;
            }
            uploads_removed += 1;
        }
    }

    let mut parts: Vec<String> = Vec::new();
    if sessions_removed > 0 {
        parts.push(format!("{} expired sessions", sessions_removed));
    }
    if download_tokens_removed > 0 {
        parts.push(format!(
            "{} expired download tokens",
            download_tokens_removed
        ));
    }
    if uploads_removed > 0 {
        parts.push(format!("{} expired uploads", uploads_removed));
    }
    if !parts.is_empty() {
        info!("housekeeping removed:\n\t{}", parts.join("\n\t"));
    }

    if !errors.is_empty() {
        warn!(
            "encountered {} housekeeping errors:\n\t{}",
            errors.len(),
            errors.join("\n\t")
        );
    }

    Ok(())
}
