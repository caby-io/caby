use std::{io, path::Path};

use tokio::fs::{read_dir, ReadDir};

pub async fn get_files(path: &Path) -> io::Result<ReadDir> {
    let full_path = Path::new("/").join(path);
    read_dir(path).await
}
