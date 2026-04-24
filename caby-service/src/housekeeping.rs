use std::str::FromStr;

use tokio::fs;
use tracing::{debug, info, warn};

use crate::{
    auth::Token as SessionToken, config::Config, download::Token as DownloadToken, Result,
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
    info!("starting housekeeping...");

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

    info!(
        "housekeeping complete. removed {} expired sessions, {} expired download tokens",
        sessions_removed, download_tokens_removed
    );

    if !errors.is_empty() {
        warn!(
            "encountered {} housekeeping errors:\n\t{}",
            errors.len(),
            errors.join("\n\t")
        );
    }

    // todo: cleanup expired uploads
    Ok(())
}
