use tokio_tar::Archive;
use tokio::fs::File;
use anyhow::Result;

pub async fn unpack_tar (path: &str, dest: &str) -> Result<()> {

    let tar = File::open(path).await?;
    let mut archive = Archive::new(tar);

    archive.unpack(dest).await?;

    Ok(())
}

#[tokio::test]
async fn test_unpack_tar () {
    let result = unpack_tar(
        "./download/HARM43_V1_P3_2026031307.tar",
        "./download/HARM43_V1_P3_2026031307",
    ).await;
}