use anyhow::Result;
use crate::util::{tar, grib, dir};

async fn load_model () -> Result<()> {

    let _ = tar::unpack_tar("./downsload/HARM43_V1_P1_2026022416.tar", "./download/grib").await?;

    let files = dir::list("./download/grib").await.unwrap();

    for (file_path, file_name) in files {

        let grib_params = grib::parse_file(file_path, file_name).await?;

        

    }

    Ok(())
}

