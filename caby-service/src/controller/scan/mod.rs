use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::{
    config::Config,
    controller::{PathLocks, Priority},
    files::{has_ext, media_type::FileKind, thumbnail::needs_thumb, CABY_SHARE_SPEC_EXT},
    job::Input,
    space::Space,
    Result,
};

pub mod shares;

#[derive(Default)]
struct ScanResult {
    share_specs: BTreeSet<PathBuf>,
    thumb_dirs: BTreeSet<PathBuf>,
}

// Walk the given space and generate a target list of files that need to be worked on
async fn try_scan_dir(space: Space) -> Result<ScanResult> {
    let label = space.live.clone();
    tokio::task::spawn_blocking(move || {
        let path = &space.live;
        let mut report = ScanResult::default();

        for entry in WalkDir::new(path) {
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
            let Ok(rel) = entry.path().strip_prefix(path) else {
                continue;
            };

            match rel {
                _ if has_ext(rel, CABY_SHARE_SPEC_EXT) => {
                    report.share_specs.insert(rel.to_path_buf());
                }
                _ if FileKind::from_path(rel) == Some(FileKind::Image)
                    && needs_thumb(&space, rel) =>
                {
                    let dir = rel.parent().unwrap_or(Path::new(""));
                    report.thumb_dirs.insert(dir.to_path_buf());
                }
                _ => {}
            }
        }

        report
    })
    .await
    .with_context(|| format!("scan of {:?} failed", label))
}

pub async fn try_scan_space(
    cfg: &Config,
    locks: &PathLocks,
    space_name: &str,
) -> Result<Vec<(Priority, Input)>> {
    info!("controller: Starting ScanSpace on {}", space_name);

    let Some(space) = cfg.find_space(space_name) else {
        return Err(anyhow!("unknown space {}", space_name));
    };

    let report = try_scan_dir(space.clone()).await?;
    if let Err(err) = shares::handle_scan(cfg, locks, &space, report.share_specs).await {
        warn!(
            "scan: could not reconcile shares in {}: {:#}",
            space_name, err
        );
    }

    let next_jobs = report
        .thumb_dirs
        .into_iter()
        .map(|dir| {
            (
                Priority::Background,
                Input::GenerateDirThumbs {
                    space: space_name.to_owned(),
                    dir,
                    recursive: false,
                    after: None,
                },
            )
        })
        .collect();

    Ok(next_jobs)
}
