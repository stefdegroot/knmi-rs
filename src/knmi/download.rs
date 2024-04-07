use futures_util::StreamExt;
use ndarray::{Array3, Array2,Array1, ArrayView, Axis};
use reqwest::StatusCode;
use tokio::{
    fs::{File},
    io::{AsyncWriteExt,},
};
use std::{collections::hash_map, ffi::OsString, fmt::format, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use axum::{
    response::{Response, IntoResponse}
};
use anyhow::{Result, Error};
use tokio_tar::Archive;
use eccodes::{codes_handle::GribFile, CodesHandle, KeyType, KeyedMessage, ProductKind};
use eccodes::FallibleStreamingIterator;
use chrono::{DateTime, TimeZone, Utc};
use peroxide::prelude::*;
use peroxide::fuga::*;
use crate::knmi::models::arome;
use netcdf;
use tokio::fs;
use std::collections::HashMap;
use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

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

    // match read_nc().await {
    match parse_grib().await {
        Ok(_) => (),
        Err(err) => {
            println!("{:?}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read grib").into_response();
        }
    }

    (StatusCode::OK, "File downloaded successfully.").into_response()
    
}

async fn read_nc () -> Result<()> {
    let current_mem = PEAK_ALLOC.current_usage_as_mb();
	println!("This program currently uses {} MB of RAM.", current_mem);
	let peak_mem = PEAK_ALLOC.peak_usage_as_gb();
	println!("The max amount that was used {}", peak_mem);

    let nc_file = netcdf::open("./download/nc/test.nc")?;

    let var_surface_global_radiation = &nc_file.variable("surface_global_radiation").unwrap();
    let mut surface_global_radiation = Array3::<f64>::zeros((48, 300, 300));
    var_surface_global_radiation.get_into((48.., .., ..), surface_global_radiation.view_mut()).unwrap();
 
    let for_coords: Vec<f64> = surface_global_radiation.outer_iter().map(|f| f[[10, 10]]).collect();

    println!("surface_global_radiation: ");
    println!("{:?}",  for_coords);

    let var_prediction_date = &nc_file.variable("prediction_date").unwrap();
    let mut prediction_date = Array1::<i64>::zeros(48);
    var_prediction_date.get_into(48.., prediction_date.view_mut()).unwrap();

    println!("prediction_date: ");
    println!("{:?}", prediction_date.to_vec());

    let var_latitudes = &nc_file.variable("latitudes").unwrap();
    let mut latitudes = Array1::<f64>::zeros(300);
    var_latitudes.get_into(.., latitudes.view_mut()).unwrap();

    println!("latitudes: ");
    println!("{:?}", latitudes.to_vec());

    let var_longitudes = &nc_file.variable("longitudes").unwrap();
    let mut longitudes = Array1::<f64>::zeros(300);
    var_longitudes.get_into(.., longitudes.view_mut()).unwrap();

    println!("longitudes: ");
    println!("{:?}", longitudes.to_vec());

    let current_mem = PEAK_ALLOC.current_usage_as_mb();
	println!("This program currently uses {} MB of RAM.", current_mem);
	let peak_mem = PEAK_ALLOC.peak_usage_as_gb();
	println!("The max amount that was used {}", peak_mem);

    Ok(())
}

async fn parse_grib () -> Result<()> {

    let path = "./download/harm40_v1_p1_2024032806";
    let mut nc_file = netcdf::create("./download/nc/test.nc")?;
    let files = list_dir(path).await?;
    let mut field_map = HashMap::<String, Array3<f64>>::new();
    let mut latitudes: Vec<f64> = vec![];
    let mut longitudes: Vec<f64> = vec![];
    let mut prediction_dates: Vec<i64> = vec![];

    nc_file.add_unlimited_dimension("time")?;
    nc_file.add_dimension("lat", 300)?;
    nc_file.add_dimension("lon", 300)?;
    
    let mut i = 0;

    for (file_path, file_name) in files {

        if i == 0 {
            i = 1;
            continue;
        }
        
        let mut handle = CodesHandle::new_from_file(&file_path, ProductKind::GRIB)?;
        let (date, prediction_date) = filename_to_dates(&file_name)?;
        
        println!("filename: {:?} - {:?} - {:?}", file_name, date, prediction_date);

        prediction_dates.push(prediction_date.timestamp_millis());

        while let Some(msg) = handle.next()? {

            if !(read_grib_string(msg, "gridType")? == "regular_ll") {
                continue;
            }

            let parameter_name = read_grib_string(msg, "parameterName")?;
            let mut field_name = read_field_name(msg, &parameter_name)?;

            if read_grib_i64(msg, "level")? == 0 && read_grib_string(msg, "typeOfLevel")? == "heightAboveGround" {
                field_name = format!("surface_{field_name}");
            } else {
                let level = read_grib_i64(msg, "level")?;
                field_name = format!("{level}m_above_ground_{field_name}");
            }

            if latitudes.is_empty() && longitudes.is_empty() {
                println!("Building cooridinates.");
                (latitudes, longitudes) = build_coordinates_grid(msg)?;
            }

            let array2 = msg.to_ndarray()?;

            if field_map.contains_key(&field_name) {
                let mut array3 = field_map.get_mut(&field_name).unwrap();
                array3.push(Axis(0), ArrayView::from(&array2)).unwrap();
            } else {
                let mut array3 = Array3::<f64>::default((0, 300, 300));
                array3.push(Axis(0), ArrayView::from(&array2)).unwrap();
                field_map.insert(field_name, array3);
            }
        }
    }

    for (key, value) in field_map.into_iter() {
        println!("saving {} to nc file", key);
        let mut var = nc_file.add_variable::<f64>(&key, &["time", "lat", "lon"])?;
        var.put((48.., .., ..), value.view())?;
    }

    let mut prediction_date_var = nc_file.add_variable::<i64>("prediction_date", &["time"])?;
    prediction_date_var.put_values(&prediction_dates, 48..)?;

    let mut latitudes_var = nc_file.add_variable::<f64>("latitudes", &["lat"])?;
    latitudes_var.put_values(&latitudes, ..)?;

    let mut longitudes_var = nc_file.add_variable::<f64>("longitudes", &["lon"])?;
    longitudes_var.put_values(&longitudes, ..)?;
    
    println!("finished parsing grib files to nc");
    
    Ok(())
}

fn build_coordinates_grid (msg: &KeyedMessage) -> Result<(Vec<f64>, Vec<f64>)> {

    let min_lat = read_grib_f64(msg, "latitudeOfFirstGridPointInDegrees")?;
    let max_lat = read_grib_f64(msg, "latitudeOfLastGridPointInDegrees")?;

    let min_lon = read_grib_f64(msg, "longitudeOfFirstGridPointInDegrees")?;
    let max_lon = read_grib_f64(msg, "longitudeOfLastGridPointInDegrees")?;

    let step_lat = read_grib_u32(msg, "jDirectionIncrement")?;
    let step_lon = read_grib_u32(msg, "iDirectionIncrement")?;

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

    Ok((latitudes, longitudes))
}

fn read_field_name (msg: &KeyedMessage, parameter_name: &str) -> Result<String> {
    let name = match arome::VAR_MAP.get_key_value(&parameter_name[..]) {
        Some((_key, value)) => {

            let name;

            if value.starts_with("_") {
                name = format!("{}{}", read_grib_string(msg, "stepType")?, value);
            } else {
                name = value.to_string();
            }
            
            name
        },
        None => format!("unknown_code_{}", parameter_name),
    };

    Ok(name)
}

fn read_grib_string (msg: &KeyedMessage, key: &str) -> Result<String> {
    let string = format!("{:?}", msg.read_key(key)?.value)
        .replace("Str(\"", "")
        .replace("\")", "");
    Ok(string)
}

fn read_grib_f64 (msg: &KeyedMessage, key: &str) -> Result<f64> {
    let float =  format!("{:?}", msg.read_key(key)?.value)
        .replace("Float(", "")
        .replace(")", "")
        .parse::<f64>()?;
    Ok(float)
}

fn read_grib_i64 (msg: &KeyedMessage, key: &str) -> Result<i64> {
    let int =  format!("{:?}", msg.read_key(key)?.value)
        .replace("Int(", "")
        .replace(")", "")
        .parse::<i64>()?;
    Ok(int)
}

fn read_grib_u32 (msg: &KeyedMessage, key: &str) -> Result<u32> {
    let int =  format!("{:?}", msg.read_key(key)?.value)
        .replace("Int(", "")
        .replace(")", "")
        .parse::<u32>()?;
    Ok(int)
}

async fn list_dir (path: &str) -> Result<Vec<(PathBuf, String)>> {

    let mut dir = tokio::fs::read_dir(path).await?;
    let mut files = Vec::new();

    while let Some(entry) = dir.next_entry().await? {
        if let Ok(name) = entry.file_name().into_string() {
            files.push((entry.path(), name))
        } else {
            println!("failed to parse name")
        }
    }

    files.sort();

    Ok(files)
}

fn filename_to_dates (filename: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {

    let year = filename[9..13].parse::<i32>()?;
    let month = filename[13..15].parse::<u32>()?;
    let day = filename[15..17].parse::<u32>()?;
    let hour = filename[17..19].parse::<u32>()?;
    let prediction_hour = filename[22..25].parse::<i64>()?;
    let date = Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap();
    let prediction_date = Utc.timestamp_millis_opt(date.timestamp_millis() + (prediction_hour * 3600 * 1000)).unwrap();

    Ok((date, prediction_date))
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

    let mut lat: usize = 0;
    let mut lon: usize = 0;
    let mut array_2 = Array3::<f64>::default((0, 300, 300));

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
       
        // println!("min_lat: {min_lat}");
        // println!("max_lat: {max_lat}");
        // println!("min_lon: {min_lon}");
        // println!("max_lon: {max_lon}");
        // println!("step_lat: {step_lat}");
        // println!("step_lon: {step_lon}");

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
        // println!("{:?}", msg.to_lons_lats_values());

        lat = latitudes.iter().position(|&l| l == 52.036).unwrap();
        lon = longitudes.iter().position(|&l| l == 5.661).unwrap();

        println!("latitude index: {:?}", lat);
        println!("longitude index: {:?}", lon);
        println!("ndarray ({}, {}): {:?}", lat, lon, array.get((lat, lon)));
        println!("ndmi: {:?}", array.ndim());
        
        if msg.read_key("parameterName")?.value == KeyType::Str("117".to_string()) {

            array_2.push(Axis(0), ArrayView::from(&array)).unwrap();
            array_2.push(Axis(0), ArrayView::from(&array)).unwrap();
            println!("{:?}", array_2);
            message_dataset.push(&field_name, Series::new(array_2.clone().into_raw_vec()));
            
            // let mut a3 = Array3::default(shape)

            // println!("{:?}",);
            
            // println!("{:?}", msg.read_key("typeOfLevel")?.value);
            // println!("{:?}", msg.read_key("level")?.value);
            // println!("{:?}", msg.read_key("parameterName")?.value);
            // println!("{:?}", msg.read_key("stepType")?.value);
            // println!("{:?}", msg.read_key("gridType")?.value);
            
            // let nearest_gridpoints = msg
            //     .codes_nearest()?
            //     .find_nearest(52.0402, 5.6649)?;
            
            // println!("{:?}", nearest_gridpoints);
            
            // println!("value: {}, distance: {}",
            //     nearest_gridpoints[3].value,
            //     nearest_gridpoints[3].distance);
        } else {
            message_dataset.push(&field_name, Series::new(array.into_raw_vec()));
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

    // println!("{:?}", message_dataset["surface_global_radiation"].len());

    // let mut test_data = ndarray::Array2::<f64>::zeros((0, 0));

    // println!("{:?}", message_dataset.header());

    // let empty = match message_dataset.write_nc(&format!("/home/stef/rust/knmi-rs/download/nc/{filename}.nc").to_string()) {
    //     _ => ""
    // };

    let file = netcdf::open(&format!("/home/stef/rust/knmi-rs/download/nc/{filename}.nc").to_string())?;
    let global_radiation = &file.variable("surface_global_radiation").expect("");

    let mut data = ndarray::Array1::<f64>::zeros((90000 * 2));
    global_radiation.get_into((..), data.view_mut()).unwrap();
    let mut shaped_data = data.into_shape((2, 300, 300)).unwrap();

    println!("{:?}", shaped_data);
    
    println!("{:?}", shaped_data.get((0, lat, lon)));
    println!("{:?}", shaped_data.get((1, lat, lon)));

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