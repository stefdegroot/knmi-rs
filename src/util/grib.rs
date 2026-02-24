use std::path::PathBuf;
use eccodes::{CodesFile, FallibleIterator, KeyRead, ProductKind};
use ndarray::Array2;

use anyhow::Result;

pub struct GribDataParam {
    pub grid_type: String,
    pub parameter_name: String,
    pub type_of_level: String,
    pub level: i64,
    pub step_type: String,
    pub grid_data: Array2<f64>
}

pub async fn parse_file (path: PathBuf, name: String) -> Result<Vec<GribDataParam>> {

    tracing::info!("Parsing GRIB file: {}", name);

    let mut handle = CodesFile::new_from_file(path, ProductKind::GRIB)?;
    let mut data = vec![];

    while let Some(msg) = handle.ref_message_iter().next()? {

        let grid_type: String = msg.read_key("gridType")?;
        let parameter_name: String = msg.read_key("parameterName")?;
        let type_of_level: String = msg.read_key("typeOfLevel")?;
        let level: i64 = msg.read_key("level")?;
        let step_type: String = msg.read_key("stepType")?;

        let grid_data = msg.to_ndarray()?;

        data.push(GribDataParam {
            grid_type,
            parameter_name,
            type_of_level,
            level,
            step_type,
            grid_data,
        });
    }

    Ok(data)
}
