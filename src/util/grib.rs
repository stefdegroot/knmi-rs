use std::{fmt::Debug, path::PathBuf};
use eccodes::{CodesFile, FallibleIterator, KeyRead, ProductKind, codes_message::CodesMessage};
use ndarray::Array2;
use peroxide::prelude::*;
use peroxide::fuga::*;

use anyhow::Result;

#[derive(Debug)]
pub struct GribDataParam {
    pub grid_type: String,
    pub parameter_name: String,
    pub level_type: String,
    pub level: i64,
    pub step_type: String,
    pub grid_data: Array2<f64>
}

fn parse_level_type (level_type: String) -> i32 {
    if level_type == "heightAboveSea" {
        103
    } else if level_type == "heightAboveGround" {
        105
    } else if level_type == "entireAtmosphere" {
        200
    } else {
        0
    }
}

fn parse_step_type (step_type: String) -> i32 {
    if step_type == "accum" {
        4
    } else if step_type == "instant" {
        0
    } else {
        99
    }
}

fn extract_grid_coordinates <P: Debug> (msg: CodesMessage<P>) -> Result<(Vec<f64>, Vec<f64>)> {

    let min_lat: f64 = msg.read_key("latitudeOfFirstGridPointInDegrees")?;
    let max_lat: f64 = msg.read_key("latitudeOfLastGridPointInDegrees")?;

    let min_lon: f64 = msg.read_key("longitudeOfFirstGridPointInDegrees")?;
    let max_lon: f64 = msg.read_key("longitudeOfLastGridPointInDegrees")?;
    
    let step_lat: i64 = msg.read_key("jDirectionIncrement")?;
    let step_lon: i64 = msg.read_key("iDirectionIncrement")?;

    let latitudes = seq(
        min_lat * 1_000.0,
        max_lat * 1_000.0,
        step_lat as u32
    ).fmap(|v| v / 1_000.0);

    let longitudes = seq(
        min_lon * 1_000.0,
        max_lon * 1_000.0,
        step_lon as u32
    ).fmap(|v| v / 1_000.0);

    Ok((latitudes, longitudes))
}

pub async fn parse_file (path: PathBuf, name: &str, extract_coordinates: bool) -> 
    Result<(
        Vec<(String, Array2<f64>)>,
        Option<(Vec<f64>, Vec<f64>)>
    )> 
{

    tracing::info!("Parsing GRIB file: {}", name);

    let mut handle = CodesFile::new_from_file(path, ProductKind::GRIB)?;
    let mut data = vec![];
    let mut coordinates = None;

    while let Some(msg) = handle.ref_message_iter().next()? {

        // let grid_type: String = msg.read_key("gridType")?;
        let parameter_name: String = msg.read_key("parameterName")?;
        let level_type: String = msg.read_key("typeOfLevel")?;
        let level: i64 = msg.read_key("level")?;
        let step_type: String = msg.read_key("stepType")?;

        let grid_data = msg.to_ndarray()?;

        let key = format!("{}_{}_{}_{}",
            parameter_name,
            parse_level_type(level_type),
            level,
            parse_step_type(step_type),
        );

        data.push(( key, grid_data ));

        if extract_coordinates && coordinates.is_none() {
            coordinates = Some(extract_grid_coordinates(msg)?);
        }
    }

    Ok((data, coordinates))
}


#[tokio::test]
async fn test_parse_file () {

    parse_file(PathBuf::from("./download/grib/HA43_N20_202602241600_00000_GB"), "test", true).await.unwrap();

}