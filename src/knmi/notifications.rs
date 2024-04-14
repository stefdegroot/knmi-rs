use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS, Transport };
use tokio::{task};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::error;
use crate::{
    AppState,
    knmi::download::download_and_parse,
    config::CONFIG,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    pub dataset_name: String,
    pub dataset_version: String,
    pub filename: String,
    pub url: String,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    specversion: String,
    r#type: String,
    source: String,
    id: String,
    time: String,
    datacontenttype: String,
    data: MessageData
}

pub async fn sub_knmi_notifications (app_state: AppState) {

    let mqtt_url: &str = "wss://mqtt.dataplatform.knmi.nl:443";
    let mqtt_client_id: String = Uuid::new_v4().to_string();
    let mut mqtt_options = MqttOptions::new(&mqtt_client_id, mqtt_url, 443);
    mqtt_options.set_transport(Transport::wss_with_default_config());
    mqtt_options.set_keep_alive(Duration::from_secs(60));
    mqtt_options.set_credentials("token",  &CONFIG.knmi.notification_service_token);
    mqtt_options.set_clean_session(false);

    let (mut client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    let dataset_name = "harmonie_arome_cy40_p1";
    let dataset_version = "0.2";
    // let dataset_name = "Actuele10mindataKNMIstations";
    // let dataset_version = "2";

    client.subscribe(format!("dataplatform/file/v1/{dataset_name}/{dataset_version}/created"), QoS::AtLeastOnce).await.unwrap();

    loop {
        let notification = eventloop.poll().await;

        let event = match notification {
            Ok(m) => m,
            Err(err) => {
               error!("{err}");
               continue;
            }
        };

        println!("Received = {:?}", event);

        if let Event::Incoming(Incoming::Publish(packet)) = event {
            
            let message: Message = match serde_json::from_slice(&packet.payload) {
                Ok(m) => m,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };

            println!("{:?}", message);

            tokio::spawn(download_and_parse(message.data));
        }
    }
}