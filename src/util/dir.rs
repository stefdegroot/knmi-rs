use std::path::PathBuf;
use anyhow::Result;

pub async fn list (path: &str) -> Result<Vec<(PathBuf, String)>> {

    let mut dir = tokio::fs::read_dir(path).await?;
    let mut files = Vec::new();

    while let Some(entry) = dir.next_entry().await? {
        if let Ok(name) = entry.file_name().into_string() {
            files.push((entry.path(), name))
        } else {
            tracing::error!("Failed to parse file name {:?}", entry)
        }
    }

    files.sort();

    Ok(files)
}
