use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::knmi::sources::KnmiSource;
use crate::config::CONFIG;
use crate::knmi::notifications::MessageData;
use crate::util::tar::unpack_tar;

pub trait Download {
    async fn download (&self, file_data: MessageData) -> Result<()>;
}

impl Download for KnmiSource {

    async fn download (&self, file_data: MessageData) -> Result<()> {
        
        let file_path = format!("./download/{}/{}", self.id, file_data.filename);
        let dir_path = file_path.replace(".tar", "");
        let downlaod_link = get_download_link(&file_data.url).await?;

        download_file(&downlaod_link.temporary_download_url, &file_path).await?;

        unpack_tar(&file_path, &dir_path).await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct KnmiDownloadReponse {
    content_type: String,
    size: String,
    last_modified: String,
    temporary_download_url: String,
}

async fn get_download_link (url: &str) -> Result<KnmiDownloadReponse> {

    let reponse = reqwest::Client::new()
        .get(url)
        .header("Authorization", &CONFIG.knmi.open_data_api.token)
        .send()
        .await?
        .json::<KnmiDownloadReponse>()
        .await?;

    Ok(reponse)
}

async fn download_file (url: &str, path: &str) -> Result<()> {

    let mut file = File::create(path).await?;

    let mut stream = reqwest::get(url).await?.error_for_status()?;

    while let Some(chunk) = stream.chunk().await? {
        file.write(&chunk).await?;
    }

    file.flush().await?;

    Ok(())
}