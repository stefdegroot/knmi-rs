use futures_util::StreamExt;
use ndarray::array;
use reqwest::StatusCode;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufReader},
};
use std::path::Path;
use serde::{Deserialize, Serialize};
use axum::{
    response::{IntoResponse, Response}
};
use anyhow::Result;
use tokio_tar::Archive;
use eccodes::{CodesHandle, KeyType, ProductKind};
use eccodes::FallibleStreamingIterator;
use chrono::{TimeZone, Utc};
use peroxide::prelude::*;
use peroxide::fuga::*;
use crate::knmi::models::arome;


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DownloadReponse {
    content_type: String,
    size: String,
    last_modified: String,
    temporary_download_url: String,
}

// Installing ecCodes
// https://gist.github.com/MHBalsmeier/a01ad4e07ecf467c90fad2ac7719844a
//  export PKG_CONFIG_PATH=/usr/src/eccodes/lib/pkgconfig
//  export LD_LIBRARY_PATH=/usr/src/eccodes/lib

pub async fn download () -> Response {

    let token = "";
    let path = "harm40_v1_p1_2024032806.tar";
    let url = format!("https://api.dataplatform.knmi.nl/open-data/v1/datasets/harmonie_arome_cy40_p1/versions/0.2/files/{}/url", path);
    let file_path = format!("/home/stef/rust/knmi-rs/download/{}", path);

    // let download_data =  match download_url(&url, token).await {
    //     Ok(data) => data,
    //     Err(err) => {
    //         println!("{:?}", err);
    //         return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get download link.").into_response()
    //     }
    // };

    // println!("{:?}", download_data);
    // println!("Downloading {}...", url);

    // match download_file(&download_data.temporary_download_url, &file_path).await {
    //     Ok(_) => (),
    //     Err(err) => {
    //         println!("{:?}", err);
    //         return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to download file").into_response()
    //     }
    // }

    // println!("Downloaded {}", url);

    // match unpack(&file_path).await {
    //     Ok(_) => (),
    //     Err(err) => {
    //         println!("{:?}", err);
    //         return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to unzip archive").into_response()
    //     }
    // }

    match read_grib().await {
        Ok(_) => (),
        Err(err) => {
            println!("{:?}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read grib").into_response()
        }
    }

    (StatusCode::OK, "File downloaded successfully.").into_response()
}

async fn read_grib () -> Result<()> {

    let filename = "HA40_N25_202403280600_00100_GB";
    let path = format!("/home/stef/rust/knmi-rs/download/harm40_v1_p1_2024032806/{filename}");
    let file_path = Path::new(&path);

    let product_kind = ProductKind::GRIB;
    let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

    // for (key, val) in arome::VAR_MAP.iter() {
    //     println!("{:?}", val);
    // }

    let year = filename[9..13].parse::<i32>()?;
    let month = filename[13..15].parse::<u32>()?;
    let day = filename[15..17].parse::<u32>()?;
    let hour = filename[17..19].parse::<u32>()?;
    let prediction_hour = filename[22..25].parse::<i64>()?;
    let date = Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap();
    let prediction_date = Utc.timestamp_millis_opt(date.timestamp_millis() + (prediction_hour * 3600 * 1000)).unwrap();

    println!("year: {year}");
    println!("month: {month}");
    println!("day: {day}");
    println!("hour: {hour}");
    println!("prediction_hour: {prediction_hour}");
    println!("date: {date}");
    println!("prediction_date: {prediction_date}");

    let mut field_name;
    let mut message_dataset = DataFrame::new(vec![]);

    while let Some(msg) = handle.next()? {

        if msg.read_key("gridType")?.value != KeyType::Str("regular_ll".to_string()) {
            continue;
        }

        let parameter_name = msg.read_key("parameterName")?.value;
        field_name = match arome::VAR_LIST.into_iter().find(|v|  KeyType::Str(v.0.to_string()) == parameter_name) {
            Some((key, value)) => {

                let name;

                if value.starts_with("_") {
                    let step_type = format!("{:?}", msg.read_key("stepType")?.value);
                    name = format!("{}{}", step_type.replace("Str(\"", "").replace("\")", ""), value);
                } else {
                    name = value.to_string()
                }
                
                name
            },
            None => format!("unknown_code_{}", format!("{:?}", parameter_name).replace("Str(\"", "").replace("\")", "")),
        };


        // println!("level: {:?}", msg.read_key("level")?.value);
        // println!("typeOfLevel: {:?}", msg.read_key("typeOfLevel")?.value);
        
        if msg.read_key("level")?.value == KeyType::Int(0) && msg.read_key("typeOfLevel")?.value == KeyType::Str("heightAboveGround".to_string()) {
            field_name = format!("surface_{field_name}");
        } else {
            let level = format!("{:?}", msg.read_key("level")?.value).replace("Int(", "").replace(")", "");
            // let type_of_level = format!("{:?}", msg.read_key("typeOfLevel")?.value).replace("Str(\"", "").replace("\")", "");
            field_name = format!("{level}m_above_ground_{field_name}");
        }

        println!("field_name: {:?}", field_name);

        // if !arome::VAR_MAP.contains_key(arome::Wrapper(msg.read_key("parameterName")?.value)) {
        //     continue;
        // }
        // if msg.read_key("parameterName")?.value !

        let min_lat = format!("{:?}", msg.read_key("latitudeOfFirstGridPointInDegrees")?.value).replace("Float(", "").replace(")", "").parse::<f32>()?;
        let max_lat = format!("{:?}", msg.read_key("latitudeOfLastGridPointInDegrees")?.value).replace("Float(", "").replace(")", "").parse::<f32>()?;
        
        let min_lon = format!("{:?}", msg.read_key("longitudeOfFirstGridPointInDegrees")?.value).replace("Float(", "").replace(")", "").parse::<f32>()?;
        let max_lon = format!("{:?}", msg.read_key("longitudeOfLastGridPointInDegrees")?.value).replace("Float(", "").replace(")", "").parse::<f32>()?;
        
        let step_lat = format!("{:?}", msg.read_key("jDirectionIncrement")?.value).replace("Int(", "").replace(")", "").parse::<u32>()?;
        let step_lon = format!("{:?}", msg.read_key("iDirectionIncrement")?.value).replace("Int(", "").replace(")", "").parse::<u32>()?;
       
        println!("min_lat: {min_lat}");
        println!("max_lat: {max_lat}");
        println!("min_lon: {min_lon}");
        println!("max_lon: {max_lon}");
        println!("step_lat: {step_lat}");
        println!("step_lon: {step_lon}");

        let latitudes = seq(
            min_lat * 1_000.0,
            max_lat * 1_000.0,
            step_lat,
        ).fmap(|v| v / 1_000.0);

        let longitudes = seq(
            min_lon * 1_000.0,
            max_lon * 1_000.0,
            step_lon,
        ).fmap(|v| v / 1_000.0);

        // println!("latitudes len: {:?}", latitudes.len());
        // println!("longitudes len: {:?}", longitudes.len());
        // println!("expected len: {:?}", latitudes.len() * longitudes.len());
        // println!("real latitudes len: {:?}", msg.to_lons_lats_values()?.latitudes.len());
        // println!("real longitudes len: {:?}", msg.to_lons_lats_values()?.longitudes.len());
        // println!("real values len: {:?}", msg.to_lons_lats_values()?);
        
        let array = msg.to_ndarray()?;
        println!("ndarray cols: {}", array.ncols());
        println!("ndarray rows: {}", array.nrows());

        message_dataset.push(&field_name, Series::new(array.into_raw_vec()));
        
        if msg.read_key("parameterName")?.value == KeyType::Str("117".to_string()) {

            // println!("{:?}", msg.read_key("typeOfLevel")?.value);
            // println!("{:?}", msg.read_key("level")?.value);
            // println!("{:?}", msg.read_key("parameterName")?.value);
            // println!("{:?}", msg.read_key("stepType")?.value);
            // println!("{:?}", msg.read_key("gridType")?.value);
        
            // let nearest_gridpoints = msg
            //     .codes_nearest()?
            //     .find_nearest(52.0402, 5.6649)?;

            // println!("value: {}, distance: {}",
            //     nearest_gridpoints[3].value,
            //     nearest_gridpoints[3].distance);
        }

        // let array = msg.to_ndarray()?;

        // println!("{:?}", array);

        // let flags = [
        //     KeysIteratorFlags::AllKeys
        // ];
        
        // let namespace = "parameterName";
        // let mut key_iter = msg.new_keys_iterator(&flags, namespace)?;

        // while let Some(key) = key_iter.next()? {
        //     println!("{:?}", key)
        // }

        // let flags = [
        //     KeysIteratorFlags::AllKeys,
        //     KeysIteratorFlags::SkipOptional,
        //     KeysIteratorFlags::SkipReadOnly,
        //     KeysIteratorFlags::SkipDuplicates,
        // ];
 
        // let mut key_iter = msg.new_keys_iterator(&flags, "paramterName")?;
        // // println!("{:?}", msg.new_keys_iterator(&flags, "parameterName"));

        // while Some(key) = key_iter.next()? {
        //     println!("{:?}", key);
        // }

        // let key = msg.read_key("typeOfLevel")?;

        // println!("{:?}", key);

        // let nearest_gridpoints = msg
        //     .codes_nearest()?
        //     .find_nearest(52.0402, 5.6649)?;

        // Print value and distance of the nearest gridpoint
        // println!("{:?}", nearest_gridpoints[0]);
        // println!("value: {}, distance: {}",
        //     nearest_gridpoints[3].value,
        //     nearest_gridpoints[3].distance);

        // if msg.read_key("shortName")?.value == KeyType::Str("GR".to_string())
        //     && msg.read_key("typeOfLevel")?.value == KeyType::Str("surface".to_string()) {
            
        //     let nearest_gridpoints = msg.codes_nearest()?
        //         // Find the nearest gridpoints to Reykjavik
        //         .find_nearest(52.0402, 5.6649)?;
    
        //     // Print value and distance of the nearest gridpoint
        //     println!("value: {}, distance: {}",
        //         nearest_gridpoints[3].value,
        //         nearest_gridpoints[3].distance);
        // }
        // if msg.read_key("shortName")?.value == KeyType::Str("GR".to_string())
        //     && msg.read_key("typeOfLevel")?.value == KeyType::Str("surface".to_string()) {
            
        //     let nearest_gridpoints = msg.codes_nearest()?
        //         // Find the nearest gridpoints to Reykjavik
        //         .find_nearest(52.0402, 5.6649)?;
    
        //     // Print value and distance of the nearest gridpoint
        //     println!("value: {}, distance: {}",
        //         nearest_gridpoints[3].value,
        //         nearest_gridpoints[3].distance);
        // }
    }

    println!("{:?}", message_dataset.header());

    // let file =  File::open(path).await?;
    // let buffer = BufReader::new(file);
    // let mut reader = Grib1Reader::new(buffer);

    // let result = reader.read_index().await?;

    // for r in result {
    //     println!("{:?}", r);
    // }

    // let result = reader.read(vec![SearchParams { param: 117, level: 0 }]).await?;

    // println!("Results: {}", result.len());
    // for grib in result {
    //     println!("{:#?}", &grib.pds);
    //     if let Some(gds) = grib.gds {
    //         println!("{:#?}", &gds);
    //         println!("{:#?}", &gds.data_representation_type);
    //         println!("{:#?}", &gds.number_of_vertical_coordinate_values);
    //         println!("{:#?}", &gds.pvl_location);
    //     }

    //     grib.bds
    // }
    
    // let grib2 = grib::from_reader(buffer)?;

    
    // for (_index, submessage) in grib2.iter() {
    //     let discipline = submessage.indicator().discipline;
    //     // Parameter category and number are included in the product definition section.
    //     // They are wrapped by `Option` because some GRIB2 data may not contain such
    //     // information.
    //     let category = submessage.prod_def().parameter_category().unwrap();
    //     let parameter = submessage.prod_def().parameter_number().unwrap();

    //     // When using the `lookup()` function, `use grib::codetables::Lookup;` is
    //     // necessary.
    //     let parameter = CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));

    //     // `forecast_time()` returns `ForecastTime` wrapped by `Option`.
    //     let forecast_time = submessage.prod_def().forecast_time().unwrap();

    //     // `fixed_layers()` returns a tuple of two layers wrapped by `Option`.
    //     let (first, _second) = submessage.prod_def().fixed_surfaces().unwrap();
    //     let elevation_level = first.value();

    //     println!(
    //         "{:<31} {:>14} {:>17}",
    //         parameter.to_string(),
    //         forecast_time,
    //         elevation_level
    //     );
    // }

    Ok(())
}



async fn unpack (path: &str) -> Result<()> {

    let tar =  File::open(path).await?;
    let mut archive = Archive::new(tar);
    // let mut entries = archive.entries().unwrap();

    // while let Some(file) = entries.next().await {
    //     let f = file.unwrap();
    //     println!("{}", f.path().unwrap().display());
    // };

    archive.unpack(path.replace(".tar", "")).await?;

    Ok(())
}

async fn download_url (url: &str, token: &str) -> Result<DownloadReponse> {

    let reponse = reqwest::Client::new()
        .get(url)
        .header("Authorization", token)
        .send()
        .await?
        .json::<DownloadReponse>()
        .await?;

    Ok(reponse)
}

async fn download_file (url: &str, path: &str) -> Result<()> {

    println!("{}", path);
    
    let mut file =  File::create(path).await?;

    let mut stream = reqwest::get(url).await?.error_for_status()?;

    while let Some(chunk) = stream.chunk().await? {
        file.write(&chunk).await?;
    }

    file.flush().await?;

    Ok(())
}