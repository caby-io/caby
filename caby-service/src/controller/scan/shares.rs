use std::{collections::BTreeSet, path::PathBuf};

use tracing::{info, warn};

use crate::{
    config::Config,
    controller::PathLocks,
    share::{get_shares_in_space, reconcile_spec},
    space::Space,
    Result,
};

pub async fn handle_scan(
    cfg: &Config,
    locks: &PathLocks,
    space: &Space,
    mut spec_paths: BTreeSet<PathBuf>,
) -> Result<()> {
    for share in get_shares_in_space(&cfg.shares_path, space).await? {
        spec_paths.insert(PathBuf::from(&share.spec_path));
    }

    let (mut reconciled, mut failed) = (0usize, 0usize);
    for spec_path in &spec_paths {
        match reconcile_spec(&cfg.shares_path, locks, space, spec_path, None).await {
            Ok(_) => reconciled += 1,
            Err(err) => {
                failed += 1;
                warn!("scan: could not reconcile {:?}: {:#}", spec_path, err);
            }
        }
    }

    info!(
        "controller: reconciled {} share spec(s) in {} ({} failed)",
        reconciled, space.name, failed
    );

    Ok(())
}
