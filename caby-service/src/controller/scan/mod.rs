use std::{collections::BTreeSet, path::PathBuf};

use anyhow::{anyhow, Context};
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::{
    config::Config,
    controller::PathLocks,
    files::{has_ext, CABY_SHARE_SPEC_EXT},
    Result,
};

pub mod shares;

#[derive(Default)]
struct ScanResult {
    share_specs: BTreeSet<PathBuf>,
}

// Walk the given space and generate a target list of files that need to be worked on
async fn try_scan_dir(path: PathBuf) -> Result<ScanResult> {
    let label = path.clone();
    tokio::task::spawn_blocking(move || {
        let mut report = ScanResult::default();

        for entry in WalkDir::new(&path) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    // todo: see how frequent this is and turn off or make debug if noisy
                    warn!("scan: skipping unreadable path: {:#}", err);
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let Ok(rel) = entry.path().strip_prefix(&path) else {
                continue;
            };

            match rel {
                _ if has_ext(rel, CABY_SHARE_SPEC_EXT) => {
                    report.share_specs.insert(rel.to_path_buf());
                }
                _ => {}
            }
        }

        report
    })
    .await
    .with_context(|| format!("scan of {:?} failed", label))
}

pub async fn try_scan_space(cfg: &Config, locks: &PathLocks, space_name: &str) -> Result<()> {
    info!("controller: Starting ScanSpace on {}", space_name);

    let Some(space) = cfg.find_space(space_name) else {
        return Err(anyhow!("unknown space {}", space_name));
    };

    let report = try_scan_dir(space.live.clone()).await?;
    shares::handle_scan(cfg, locks, &space, report.share_specs).await?;

    Ok(())
}
