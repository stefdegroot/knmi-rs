use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub enum KnmiSourceTag {
    ForecastNetherlands,
    ForecastEurope,
    RealTimeObservations,
}

#[derive(Deserialize, Debug)]
pub struct KnmiSource {
    pub tag: KnmiSourceTag,
    pub id: Box<str>,
    pub version: Box<str>,
}

pub fn get_source (source: &KnmiSourceTag) -> KnmiSource {
    match source {
        KnmiSourceTag::ForecastNetherlands => KnmiSource {
            tag: KnmiSourceTag::ForecastNetherlands,
            id: "harmonie_arome_cy43_p1".into(),
            version: "1.0".into(),
        },
        KnmiSourceTag::ForecastEurope => KnmiSource {
            tag: KnmiSourceTag::ForecastEurope,
            id: "harmonie_arome_cy43_p3".into(),
            version: "1.0".into(),
        },
        KnmiSourceTag::RealTimeObservations => KnmiSource {
            tag: KnmiSourceTag::RealTimeObservations,
            id: "10-minute-in-situ-meteorological-observations".into(),
            version: "1.0".into(),
        },
    }
}