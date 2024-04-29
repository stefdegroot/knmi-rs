use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use lazy_static::lazy_static;
use peroxide::fuga::{seq, FPVector};
use ndarray::Array3;
use tracing::{error, info};

use crate::knmi::{
    download::download_and_parse,
    notifications::MessageData,
    files::list_latest_files,
};

pub type NCMap = Arc<RwLock<HashMap<String, Array3<f64>>>>;

#[derive(Debug, Clone)]
pub struct Arome {
    pub last_update: Arc<RwLock<String>>,
    pub nc_map: NCMap,
    pub latitudes: Vec<f64>,
    pub longitudes: Vec<f64>,
}

impl Arome {
    pub async fn new () -> Self {

        // let min_lat = 49.0;
        // let max_lat = 55.877;
    
        // let min_lon = 0.0;
        // let max_lon = 11.063;
    
        // let step_lat = 23;
        // let step_lon = 37;

        let min_lat = 49.0;
        let max_lat = 56.002;
    
        let min_lon = 0.0;
        let max_lon = 11.281;
    
        let step_lat = 18;
        let step_lon = 29;

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

        let files = list_latest_files().await.unwrap();

        // let latest_date: String = download_and_parse(MessageData {
        //     filename: format!("{}", files[files.len() - 1].filename),
        //     dataset_name: "harmonie_arome_cy40_p1".to_string(),
        //     dataset_version: "0.2".to_string(),
        //     url: format!("https://api.dataplatform.knmi.nl/open-data/v1/datasets/harmonie_arome_cy40_p1/versions/0.2/files/{}/url",  files[files.len() - 1].filename),
            
        // }).await.unwrap();

        let latest_date: String = download_and_parse(MessageData {
            filename: format!("{}", files[files.len() - 1].filename),
            dataset_name: "harmonie_arome_cy43_p1".to_string(),
            dataset_version: "1.0".to_string(),
            url: format!("https://api.dataplatform.knmi.nl/open-data/v1/datasets/harmonie_arome_cy43_p1/versions/1.0/files/{}/url",  files[files.len() - 1].filename),
            
        }).await.unwrap();

        Self {
            last_update: Arc::new(RwLock::new(latest_date)),
            nc_map: Arc::new(RwLock::new(HashMap::new())),
            latitudes,
            longitudes,
        }
    }

    pub async fn update_model (&self, message: MessageData) -> () {

        let latest_date = match download_and_parse(message).await {
            Ok(date) => date,
            Err(_) => {
                error!("Failed to download latest Arome model.");
                return
            }
        };
        
        let mut last_update = self.last_update.write().unwrap();
        *last_update = latest_date.clone();

        self.nc_map.write().unwrap().clear();

        info!(latest_date = latest_date, "Arome model updated with latest data: ");
    }

    pub fn closest_coords_position (&self, lat: f64, lon: f64) -> (usize, usize) {
        (
            closest(&self.latitudes, lat), 
            closest(&self.longitudes, lon)
        )
    }
}

fn closest (vec: &Vec<f64>, value: f64) -> usize {

    let closest;

    let max = vec.iter().position(|&l| l >= value).unwrap_or(0);

    if value != vec[max] && max > 0 {
        let min_dif = vec[max] - value;
        let max_dif = value - vec[max - 1];

        if min_dif > max_dif {
            closest = max - 1
        } else {
            closest = max
        }
    } else {
        closest = max
    }

    closest
}

lazy_static! {
    pub static ref VAR_MAP: HashMap<&'static str, &'static str> = {
        HashMap::from([
            ("1", "mean_sea_level_pressure"),
            ("6", "geopotential"),
            ("11", "temperature"),
            ("17", "dew_point_temperature"),
            ("20", "visibility"),
            ("33", "u_component_of_wind"),
            ("34", "v_component_of_wind"),
            ("52", "relative_humidity"),
            ("66", "snow_cover"),
            ("67", "boundary_layer_height"),
            ("71", "cloud_cover"),
            ("73", "low_cloud_cover"),
            ("74", "medium_cloud_cover"),
            ("75", "high_cloud_cover"),
            ("111", "net_short_wave_radiation"),
            ("112", "net_long_wave_radiation"),
            ("117", "global_radiation"),
            ("122", "sensible_heat_flux"),
            ("132", "latent_heat_flux"),
            ("162", "u_component_max_squall"),
            ("163", "v_component_max_squall"),
            ("181", "_rain_water"),
            ("184", "_snow_water"),
            ("186", "cloud_base"),
            ("201", "_graupel"),
            ("SD Snow depth m", "snow_depth"),
            ("T Temperature K", "temperature - Extra copy"),
        ])
    };
}