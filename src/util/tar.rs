use tokio_tar::Archive;
use tokio::fs::File;
use anyhow::Result;

pub async fn unpack_tar (path: &str, dest: &str) -> Result<()> {

    let tar = File::open(path).await?;
    let mut archive = Archive::new(tar);

    archive.unpack(dest).await?;

    Ok(())
}
