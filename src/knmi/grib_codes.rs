use std::collections::HashMap;
use lazy_static::lazy_static;


pub struct GribCode {
    code: i32,
    short_name: &'static str,
    description: &'static str,
    units: &'static str,
    level_type: i32,
    level: i32,
    tri: i32,
}

lazy_static! {
    pub static ref GRIB_CODES: HashMap<&'static str, GribCode> = {
        HashMap::from([
            (
                "1_103_0_0",
                GribCode {
                    code: 1,
                    short_name: "PMSL",
                    description: "Pressure altitude above mean sea level",
                    units: "Pa",
                    level_type: 103,
                    level: 0,
                    tri: 0,
                } 
            ),
            (
                "11_105_2_0",
                GribCode {
                    code: 1,
                    short_name: "TMP",
                    description: "Temperature",
                    units: "K",
                    level_type: 105,
                    level: 2,
                    tri: 0,
                } 
            )
        ])
    };
}

