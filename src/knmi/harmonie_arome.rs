use std::collections::HashMap;
use anyhow::Result;
use ndarray::{Array3, Axis, ArrayView};
use chrono::{DateTime, TimeZone, Utc};
use crate::util::{tar, grib, dir};
use crate::knmi::grib_codes::GRIB_CODES;

#[derive(Debug)]
pub struct GribModel {
    publish_date: DateTime<Utc>,
    latitudes: Vec<f64>,
    longitudes: Vec<f64>,
    times: Vec<i64>,
    params: HashMap<String, Array3<f64>>
}

async fn load_model () -> Result<GribModel> {

    // let _ = tar::unpack_tar("./download/HARM43_V1_P1_2026022416.tar", "./download/grib").await?;

    let files = dir::list("./download/grib").await.unwrap();

    let mut params= HashMap::<String, Array3<f64>>::new();
    let mut publish_date: Option<DateTime<Utc>> = None;
    let mut times: Vec<i64> = vec![];
    let mut latitudes: Vec<f64> = vec![];
    let mut longitudes: Vec<f64> = vec![];

    for (file_path, file_name) in files {

        let (data, coordinates) = grib::parse_file(file_path, &file_name, true).await?;
        let (base_date, predicted_date) = parse_date_from_filename(&file_name)?;

        tracing::info!("Loading predicted date: {}", predicted_date);

        times.push(predicted_date.timestamp_millis());

        if publish_date.is_none() {
            publish_date = Some(base_date);
        }

        for (key, grid) in data {

            if GRIB_CODES.get(key.as_str()).is_some() {

                if let Some(array_3) = params.get_mut(key.as_str()) {
                    array_3.push(Axis(0), ArrayView::from(&grid)).unwrap();
                } else {
                    let mut array_3 = Array3::<f64>::default((0, 390, 390));
                    array_3.push(Axis(0), ArrayView::from(&grid)).unwrap();
                    params.insert(key, array_3);
                }
            }
        }

        if let Some((lats, longs)) = coordinates {
            latitudes = lats;
            longitudes = longs;
        }
    }

    if let Some(publish_date) = publish_date {
        Ok(GribModel {
            publish_date,
            latitudes,
            longitudes,
            times,
            params,
        })
    } else {
        Err(anyhow::format_err!("Failed to read publish date from GRIB model."))
    }
}

fn parse_date_from_filename (filename: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {

    let split_name: Vec<&str> = filename.split("_").collect();
    let date_time = split_name.get(2).unwrap();
    
    let year: i32 = date_time[0..4].parse()?;
    let month: u32 = date_time[4..6].parse()?;
    let day: u32 = date_time[6..8].parse()?;
    let hour: u32 = date_time[8..10].parse()?;

    let prediction_order = split_name.get(3).unwrap();
    let prediction_hour: i64 = prediction_order[0..3].parse()?;
    
    let base_date = Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap();
    let predicted_date = Utc.timestamp_millis_opt(base_date.timestamp_millis() + (prediction_hour * 3600 * 1000)).unwrap();

    Ok((base_date, predicted_date))
}

#[tokio::test]
async fn test_load_model () {
   let model = load_model().await.unwrap();

//    println!("{:?}", model);
}

#[tokio::test]
async fn test_parse_date_from_filename () {

    let (
        base_date,
        predicted_date
    ) = parse_date_from_filename("HA43_N20_202602241600_01200_GB").unwrap();

    assert_eq!(base_date.to_rfc3339(), "2026-02-24T16:00:00+00:00");
    assert_eq!(predicted_date.to_rfc3339(), "2026-02-25T04:00:00+00:00");
}