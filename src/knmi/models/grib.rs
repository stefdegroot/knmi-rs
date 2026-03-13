use std::collections::HashMap;
use chrono::{DateTime, Utc};
use ndarray::Array3;

use crate::knmi::models::Model;

pub struct Grib {
    publish_date: DateTime<Utc>,
    latitudes: Vec<f64>,
    longitudes: Vec<f64>,
    times: Vec<i64>,
    params: HashMap<String, Array3<f64>>
}

impl Model for Grib {
    
    fn load_model (&self) -> () {
        
    }

    fn update_model (&self) -> () {
        
    }
}
