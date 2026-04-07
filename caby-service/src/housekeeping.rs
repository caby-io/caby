use std::str::FromStr;

use tokio::fs;
use tracing::{debug, info, warn};

use crate::{auth::Token, config::Config, Result};

pub async fn housekeeping(cfg: &Config) -> Result<()> {
    info!("starting housekeeping...");

    // Sessions
    let mut sessions_removed: u32 = 0;
    let mut session_errors: Vec<String> = vec![];
    for (_, user) in cfg.users.iter() {
        let mut dir = match fs::read_dir(&user.path).await {
            Ok(d) => d,
            Err(err) => {
                // this is a debug because a uninitilized user will have no user dir
                debug!("could not read user directory for {}: {}", user.name, err);
                continue;
            }
        };

        while let Ok(Some(entry)) = dir.next_entry().await {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };

            if !name.starts_with("session_") {
                continue;
            }

            let content = match fs::read_to_string(entry.path()).await {
                Ok(c) => c,
                Err(err) => {
                    session_errors.push(format!(
                        "could not read session file {}/{}: {:#}",
                        user.name, name, err
                    ));
                    continue;
                }
            };

            let token = match Token::from_str(&content) {
                Ok(t) => t,
                Err(err) => {
                    session_errors.push(format!(
                        "could not parse session file {}/{}: {:#}",
                        user.name, name, err
                    ));
                    continue;
                }
            };

            if token.is_expired() {
                if let Err(err) = fs::remove_file(entry.path()).await {
                    session_errors.push(format!(
                        "could not remove expired session file {}/{}: {:#}",
                        user.name, name, err
                    ));
                    continue;
                }
                sessions_removed += 1;
            }
        }
    }

    info!(
        "housekeeping complete. removed {} expired sessions",
        sessions_removed
    );

    if !session_errors.is_empty() {
        warn!(
            "encountered {} session errors:\n\t{}",
            session_errors.len(),
            session_errors.join("\n\t")
        );
    }

    // todo: cleanup expired uploads
    Ok(())
}
