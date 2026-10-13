use std::{
    ffi::{OsStr, OsString},
    io::ErrorKind,
    path::Path,
};

use anyhow::anyhow;
use tokio::{fs, time};
use tracing::{debug, warn};

use crate::{
    config::Config,
    controller::{EventHandler, Priority},
    event::{
        Event,
        EventKind::{FileCreated, FileModified},
    },
    files::{has_ext, media_type::FileKind, thumbnail::try_ensure_thumb, CABY_SHARE_SPEC_EXT},
    job::{Input, THUMBS_BATCH_SIZE, THUMB_TIMEOUT},
    space::SpaceDir,
    Result,
};

fn handle_event(event: &Event) -> Vec<(Priority, Input)> {
    if !matches!(event.kind, FileCreated | FileModified)
        || has_ext(&event.path, CABY_SHARE_SPEC_EXT)
    {
        return vec![];
    }

    let input = match FileKind::from_path(&event.path) {
        Some(FileKind::Image) => Input::GenerateFileThumb {
            space: event.space.clone(),
            path: event.path.clone(),
        },
        Some(_) => return vec![],
        None => Input::GenerateDirThumbs {
            space: event.space.clone(),
            dir: event.path.clone(),
            recursive: true,
            after: None,
        },
    };

    vec![(Priority::Background, input)]
}

pub fn handlers() -> Vec<EventHandler> {
    vec![handle_event]
}

fn next_batch(mut images: Vec<OsString>, after: Option<&OsStr>) -> (Vec<OsString>, bool) {
    if let Some(after) = after {
        images.retain(|name| name.as_os_str() > after);
    }
    images.sort_unstable();

    let batch_size = THUMBS_BATCH_SIZE as usize;
    let has_more = images.len() > batch_size;
    images.truncate(batch_size);

    (images, has_more)
}

pub async fn try_generate_file_thumb(cfg: &Config, space_name: &str, path: &Path) -> Result<()> {
    debug!(
        "controller: Starting GenerateFileThumb on {}/{}",
        space_name,
        path.display()
    );

    let Some(space) = cfg.find_space(space_name) else {
        return Err(anyhow!("unknown space {}", space_name));
    };

    try_ensure_thumb(&space, path, cfg.img_thumbs.max_edge).await
}

pub async fn try_generate_dir_thumbs(
    cfg: &Config,
    space_name: &str,
    dir: &Path,
    recursive: bool,
    after: Option<&OsStr>,
) -> Result<Vec<(Priority, Input)>> {
    debug!(
        "controller: Starting GenerateDirThumbs on {}/{} after {:?}",
        space_name,
        dir.display(),
        after
    );

    let Some(space) = cfg.find_space(space_name) else {
        return Err(anyhow!("unknown space {}", space_name));
    };

    let mut entries = match fs::read_dir(space.join(SpaceDir::LIVE, dir)?).await {
        Ok(entries) => entries,
        Err(err) if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            return Ok(vec![]);
        }
        Err(err) => return Err(anyhow!(err).context(format!("read dir {:?}", dir))),
    };

    let mut images = vec![];
    let mut subdirs = vec![];
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = entry.file_name();

        if file_type.is_dir() {
            subdirs.push(name);
        } else if file_type.is_file()
            && FileKind::from_path(Path::new(&name)) == Some(FileKind::Image)
        {
            images.push(name);
        }
    }

    let (batch, has_more) = next_batch(images, after);
    for name in &batch {
        let rel = dir.join(name);
        match time::timeout(
            THUMB_TIMEOUT,
            try_ensure_thumb(&space, &rel, cfg.img_thumbs.max_edge),
        )
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(err)) => warn!("thumbs: skipping {}: {:#}", rel.display(), err),
            Err(_) => warn!(
                "thumbs: skipping {}, exceeded {:?}",
                rel.display(),
                THUMB_TIMEOUT
            ),
        }
    }

    // since we've already loaded the listing, generate the next set of jobs
    let mut next_jobs = vec![];
    if let Some(last) = batch.last().filter(|_| has_more) {
        next_jobs.push((
            Priority::Background,
            Input::GenerateDirThumbs {
                space: space_name.to_owned(),
                dir: dir.to_path_buf(),
                recursive,
                after: Some(last.clone()),
            },
        ));
    }
    if recursive && after.is_none() {
        next_jobs.extend(subdirs.into_iter().map(|name| {
            (
                Priority::Background,
                Input::GenerateDirThumbs {
                    space: space_name.to_owned(),
                    dir: dir.join(name),
                    recursive: true,
                    after: None,
                },
            )
        }));
    }

    Ok(next_jobs)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::event::Event;

    use super::*;

    fn names(names: &[&str]) -> Vec<OsString> {
        names.iter().map(OsString::from).collect()
    }

    #[test]
    fn created_image_queues_a_single_thumb() {
        let inputs = handle_event(&Event::from_create("rocinante", "holiday/ceres.jpg"));
        assert_eq!(
            inputs,
            vec![(
                Priority::Background,
                Input::GenerateFileThumb {
                    space: "rocinante".to_owned(),
                    path: PathBuf::from("holiday/ceres.jpg"),
                }
            )]
        );
    }

    #[test]
    fn created_path_without_extension_queues_a_recursive_dir_job() {
        let inputs = handle_event(&Event::from_modify("rocinante", "holiday"));
        assert_eq!(
            inputs,
            vec![(
                Priority::Background,
                Input::GenerateDirThumbs {
                    space: "rocinante".to_owned(),
                    dir: PathBuf::from("holiday"),
                    recursive: true,
                    after: None,
                }
            )]
        );
    }

    #[test]
    fn non_image_files_and_share_specs_are_ignored() {
        assert!(handle_event(&Event::from_create("rocinante", "logs/burn.txt")).is_empty());
        assert!(handle_event(&Event::from_create("rocinante", "holiday.share.caby")).is_empty());
    }

    #[test]
    fn removals_and_moves_are_ignored() {
        assert!(handle_event(&Event::from_remove("rocinante", "holiday/ceres.jpg")).is_empty());
        assert!(handle_event(&Event::from_move("rocinante", "a.jpg", "b.jpg")).is_empty());
    }

    #[test]
    fn batch_is_sorted_and_starts_after_the_cursor() {
        let (batch, has_more) = next_batch(
            names(&["eros.jpg", "ceres.jpg", "tycho.jpg"]),
            Some(OsStr::new("ceres.jpg")),
        );
        assert_eq!(batch, names(&["eros.jpg", "tycho.jpg"]));
        assert!(!has_more);
    }

    #[test]
    fn batch_is_capped_and_reports_more() {
        let images = (0..THUMBS_BATCH_SIZE + 1)
            .map(|i| OsString::from(format!("{i:04}.jpg")))
            .collect();
        let (batch, has_more) = next_batch(images, None);
        assert_eq!(batch.len(), THUMBS_BATCH_SIZE as usize);
        assert_eq!(batch.last().unwrap(), "0199.jpg");
        assert!(has_more);
    }
}
