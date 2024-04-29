use rumqttc::tokio_rustls::rustls::internal::msgs::message::Message;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use crate::{config::CONFIG, knmi::{download::download_and_parse, notifications::MessageData}};

#[derive(Serialize, Deserialize, Debug)]
struct TestStruct {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DatasetFile {
    pub filename: String,
    pub size: i32,
    pub created: String,
    pub last_modified: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Datasets {
    pub is_truncated: bool,
    pub result_count: i32,
    pub files: Vec<DatasetFile>,
    pub max_results: Option<i32>,
    pub start_after_filename: Option<String>,
    pub next_page_token: Option<String>,
}

pub async fn list_latest_files() -> Result<Vec<DatasetFile>, ()> {

    // let url = "https://api.dataplatform.knmi.nl/open-data/v1/datasets/harmonie_arome_cy40_p1/versions/0.2/files";
    let url = "https://api.dataplatform.knmi.nl/open-data/v1/datasets/harmonie_arome_cy43_p1/versions/1.0/files";

    let reponse = reqwest::Client::new()
        .get(url)
        .query(&[("maxKeys", "1000")])
        .header("Authorization", &CONFIG.knmi.open_data_api_token)
        .send()
        .await;

    let raw_data = match reponse {
        Ok(res) => res,
        Err(err) => {
            error!("{err}");
            return Err(());
        }
    };

    let data = match raw_data.json::<Datasets>().await {
        Ok(res) => res,
        Err(err) => {
            error!("{err}");
            return Err(());
        }
    };

    info!(result_count = data.result_count, "Returned:");
    info!(created = data.files[data.files.len() - 1].created, "Newest file:");

    // download_and_parse(MessageData {
    //     filename: format!("{}", data.files[data.files.len() - 1].filename),
    //     dataset_name: "harmonie_arome_cy40_p1".to_string(),
    //     dataset_version: "0.2".to_string(),
    //     url: format!("https://api.dataplatform.knmi.nl/open-data/v1/datasets/harmonie_arome_cy40_p1/versions/0.2/files/{}/url",  data.files[9].filename),
        
    // }).await;

    Ok(data.files)
}