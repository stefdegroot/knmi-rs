use serde::{Deserialize, Serialize};
use axum::{
    response::{IntoResponse, Response},
    Json
};

#[derive(Serialize, Deserialize, Debug)]
struct TestStruct {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DatasetFile {
    filename: String,
    size: i32,
    created: String,
    last_modified: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Datasets {
    is_truncated: bool,
    result_count: i32,
    files: Vec<DatasetFile>,
    max_results: Option<i32>,
    start_after_filename: Option<String>,
    next_page_token: Option<String>,
}

pub async fn pull_with_reqwest() -> Response {

    let token = "";
    let url = "https://api.dataplatform.knmi.nl/open-data/v1/datasets/harmonie_arome_cy40_p1/versions/0.2/files";
    
    let reponse = reqwest::Client::new()
        .get(url)
        .header("Authorization", token)
        .send()
        .await;

    let raw_data = match reponse {
        Ok(res) => res,
        Err(err) => {
            println!("{:?}", err);

            return Json(TestStruct {
                name: "test model".to_string(),
            }).into_response();
        }
    };

    // let test = match raw_data.text().await {
    //     Ok(res) => res,
    //     Err(err) => {
    //         println!("{:?}", err);

    //         return Json(TestStruct {
    //             name: "test model".to_string(),
    //         }).into_response();
    //     }
    // };

    // println!("{:?}", test);

    // Json(TestStruct {
    //     name: "test model".to_string(),
    // }).into_response()

    let data = match raw_data.json::<Datasets>().await {
        Ok(res) => res,
        Err(err) => {
            println!("{:?}", err);

            return Json(TestStruct {
                name: "test model".to_string(),
            }).into_response();
        }
    };  

    println!("{:?}", data);

    Json(data).into_response()
}