use serde::Deserialize;
use crate::{config::CONFIG, knmi::{download::Download, notifications::MessageData}};
// use crate::knmi::download::

#[derive(Deserialize, Debug, Clone)]
pub enum KnmiSourceTag {
    ForecastNetherlands,
    ForecastEurope,
    RealTimeObservations,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KnmiSource {
    pub tag: KnmiSourceTag,
    pub id: Box<str>,
    pub version: Box<str>,
}

impl KnmiSource {

    // Called when a notification is recieved to download the new model
    // and replace the current one in memorry
    pub async fn update_model (&self, file_data: MessageData) {

        self.download(file_data).await.unwrap();
        
        // load model

        tracing::info!("Updating model: {}", self.id)
    }

    // Called on startup to load the latest data into memmory
    pub async fn load_model (&self) {
        tracing::info!("Loading model: {}", self.id)
    }
}

fn get_source (source: &KnmiSourceTag) -> KnmiSource {
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

pub fn load_sources_from_config () -> Vec<KnmiSource> {

    let mut sources = vec![];

    for source_tag in &CONFIG.knmi.sources {
        sources.push(get_source(source_tag))
    }

    sources
}