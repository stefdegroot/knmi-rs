use std::collections::HashMap;
use lazy_static::lazy_static;
use axum::{
    extract::{Json, Path, Query, State}, response::{IntoResponse, Response}
};
use ndarray::{Array3, Array2,Array1, ArrayView, Axis};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Number};
use reqwest::StatusCode;
use anyhow::{Result};
use tracing::{error, info};
use std::sync::{Arc, Mutex};
use crate::{config::Knmi, knmi::download::PEAK_ALLOC, AppState, knmi::models::arome::NCMap};
use super::models::arome::Arome;

#[derive(Serialize, Deserialize, Debug)]
struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForecastInput {
    coords: Coordinates,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForecastItem {
    timestamp: i64,
    above_ground_200m_v_component_of_wind: i64,
    above_ground_100m_u_component_of_wind: i64,
    above_ground_0m_mean_sea_level_pressure: i64,
    above_ground_200m_u_component_of_wind: i64,
    above_ground_100m_temperature: i64,
    above_ground_10m_u_component_max_squall: i64,
    surface_low_cloud_cover: i64,
    surface_accum_snow_water: i64,
    surface_instant_snow_water: i64,
    surface_medium_cloud_cover: i64,
    above_ground_2m_temperature: i64,
    surface_global_radiation: i64,
    surface_temperature: i64,
    above_ground_10m_u_component_of_wind: i64,
    surface_unknown_code_81: i64,
    surface_high_cloud_cover: i64,
    surface_boundary_layer_height: i64,
    surface_sensible_heat_flux: i64,
    // surface_net_short_wave_radiation: i64,
    above_ground_0m_cloud_base: i64,
    above_ground_10m_v_component_max_squall: i64,
    above_ground_50m_temperature: i64,
    above_ground_50m_v_component_of_wind: i64,
    surface_latent_heat_flux: i64,
    surface_accum_graupel: i64,
    above_ground_0m_instant_graupel: i64,
    surface_unknown_code_20: i64,
    above_ground_2m_relative_humidity: i64,
    surface_net_long_wave_radiation: i64,
    surface_cloud_cover: i64,
    surface_instant_graupel: i64,
    surface_accum_rain_water: i64,
    above_ground_300m_temperature: i64,
    above_ground_10m_v_component_of_wind: i64,
    above_ground_300m_u_component_of_wind: i64,
    above_ground_300m_v_component_of_wind: i64,
    above_ground_100m_v_component_of_wind: i64,
    above_ground_200m_temperature: i64,
    surface_geopotential: i64,
    surface_mean_sea_level_pressure: i64,
    above_ground_50m_u_component_of_wind: i64,
    surface_snow_depth: i64,
    surface_instant_rain_water: i64,
}

const NC_KEYS: &[&str] = &[
    "200m_above_ground_v_component_of_wind",
    "100m_above_ground_u_component_of_wind",
    "0m_above_ground_mean_sea_level_pressure",
    "200m_above_ground_u_component_of_wind",
    "surface_unknown_code_61",
    "100m_above_ground_temperature",
    "10m_above_ground_u_component_max_squall",
    "surface_low_cloud_cover",
    "surface_accum_snow_water",
    "surface_instant_snow_water",
    "2m_above_ground_unknown_code_17",
    "surface_medium_cloud_cover",
    "2m_above_ground_temperature",
    "surface_global_radiation",
    "surface_temperature",
    "10m_above_ground_u_component_of_wind",
    "surface_unknown_code_81",
    "surface_high_cloud_cover",
    "surface_boundary_layer_height",
    "surface_sensible_heat_flux",
    // "surface_net_short_wave_radiation",
    "0m_above_ground_cloud_base",
    "10m_above_ground_v_component_max_squall",
    "50m_above_ground_temperature",
    "50m_above_ground_v_component_of_wind",
    "surface_latent_heat_flux",
    "surface_accum_graupel",
    "0m_above_ground_instant_graupel",
    "surface_unknown_code_20",
    "2m_above_ground_relative_humidity",
    "surface_net_long_wave_radiation",
    "surface_cloud_cover",
    "surface_instant_graupel",
    "surface_accum_rain_water",
    "300m_above_ground_temperature",
    "10m_above_ground_v_component_of_wind",
    "surface_temperature - Extra copy",
    "300m_above_ground_u_component_of_wind",
    "300m_above_ground_v_component_of_wind",
    "100m_above_ground_v_component_of_wind",
    "200m_above_ground_temperature",
    "surface_geopotential",
    "surface_mean_sea_level_pressure",
    "50m_above_ground_u_component_of_wind",
    "surface_snow_depth",
    "surface_instant_rain_water",
    "surface_visibility",
    "2m_above_ground_dew_point",
];

#[derive(Serialize, Deserialize, Debug)]
pub struct Forecast {
    forecast: Vec<ForecastItem>,
}

pub async fn forecast (
    State(state): State<AppState>,
    Json(payload): Json<ForecastInput>,
) -> Response {
    
    info!(last_update = *state.arome.last_update.read().unwrap());

    info!(lat = payload.coords.lat, lon = payload.coords.lon, "forecast");

    let (lat, lon) = state.arome.closest_coords_position(payload.coords.lat, payload.coords.lon);

    match extract_forecast(state.arome.nc_map, lat, lon).await {
        Ok(forecast) => {
            return Json(forecast).into_response();
        },
        Err(err) => {
            println!("{:?}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read grib").into_response();
        }
    }
}

async fn extract_forecast (nc_map: NCMap , lat: usize, lon: usize) -> Result<Vec<Value>> {

    // let mut data = Forecast {
    //     forecast: vec![],
    // };

    // let current_mem = PEAK_ALLOC.current_usage_as_mb();
	// println!("This program currently uses {} MB of RAM.", current_mem);
	// let peak_mem = PEAK_ALLOC.peak_usage_as_gb();
	// println!("The max amount that was used {}", peak_mem);
    
    let mut data = vec![];

    let nc_file = netcdf::open("./download/nc/arome.nc")?;

    let var_prediction_date = &nc_file.variable("prediction_date").unwrap();
    // let mut prediction_date = Array1::<f64>::zeros(48);
    let mut prediction_date = Array1::<f64>::zeros(53);

    // match var_prediction_date.get_into(48.., prediction_date.view_mut()) {
    match var_prediction_date.get_into(53.., prediction_date.view_mut()) {
        Ok(_) => {},
        Err(_) => {
            error!("Failed to read predicton date")
        }  
    }

    // for index in 0..48 {
    for index in 0..53 {

        let mut map = Map::new();
        
        for key in NC_KEYS.into_iter() {
            map.insert(
                key.to_string(), 
                Value::Number(Number::from_f64(
                    read_nc_field(&nc_map, &nc_file, key, [index, lat, lon]).unwrap()
                ).unwrap())
            );
        }

        // let forecast_item = ForecastItem {
        //     timestamp: prediction_date[index],
        //     surface_cloud_cover: 0.0,
        //     surface_global_radiation: read_nc_field(&nc_file, "surface_global_radiation").unwrap()[[index, lat, lon]],
        // };

        map.insert("timestamp".to_string(), Value::Number(Number::from_f64(
            prediction_date[index]
        ).unwrap()));

        data.push(Value::Object(map));
    }

    Ok(data)
}

fn read_nc_field (nc_map: &NCMap, nc: &netcdf::File, field: &str, index: [usize; 3]) -> Result<f64> {

    let mut map = nc_map.write().unwrap();

    if !map.contains_key(field) {

        if let Some(var) = &nc.variable(field) {
            // let var = &nc.variable(field).unwrap();
            // let mut data = Array3::<f64>::zeros((48, 300, 300));
            let mut data = Array3::<f64>::zeros((53, 390, 390));

            // var.get_into((48.., .., ..), data.view_mut()).unwrap();
            match var.get_into((53.., .., ..), data.view_mut()) {
                Ok(_) => {
                    map.insert(field.to_string(), data);
                },
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
    }

    
    if let Some(map_field) = map.get(field) {
        Ok(map_field[index])
    } else {
        Ok(0.0)
    }

}