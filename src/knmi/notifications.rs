use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS, Transport };
use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    AppState,
    knmi::sources::get_source,
    config::CONFIG,
};

pub enum MessageEvent {
    Created,
    Updated,
}

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

    let id = Uuid::new_v4().to_string();
    let host = format!("wss://{}", CONFIG.knmi.notification_service.url);
    let port = CONFIG.knmi.notification_service.port;

    let mut mqtt_options = MqttOptions::new(id, host, port);
    mqtt_options.set_transport(Transport::wss_with_default_config());
    mqtt_options.set_keep_alive(Duration::from_secs(60));
    mqtt_options.set_credentials("token",  &CONFIG.knmi.notification_service.token);
    mqtt_options.set_clean_session(false);

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    for source_tag in &CONFIG.knmi.sources {

        let source = get_source(source_tag);

        match client.subscribe(
            format!("dataplatform/file/v1/{}/{}/#", source.id, source.version), 
            QoS::AtLeastOnce
        ).await {
            Ok(_) => tracing::info!("Successfully subscribed to {}", source.id),
            Err(err) => {
                tracing::error!("Failed to subscribed to {}", source.id);
                tracing::error!("{:?}", err);
            },
        };
    }

    loop {

        let notification = eventloop.poll().await;

        let event = match notification {
            Ok(m) => m,
            Err(err) => {
               tracing::error!("{err}");
               continue;
            }
        };

        if let Event::Incoming(Incoming::Publish(packet)) = event {

            tracing::info!("Recieved message on topic: {}", packet.topic);
            
            let message: Message = match serde_json::from_slice(&packet.payload) {
                Ok(m) => m,
                Err(err) => {
                    tracing::error!("{err}");
                    continue;
                }
            };

            let message_event;

            if packet.topic.ends_with("created") {
                message_event = MessageEvent::Created;
            } else if packet.topic.ends_with("updated") {
                message_event = MessageEvent::Updated;
            } else {
                tracing::warn!("Unkown message event type.");
                continue;
            }

            tokio::spawn(update_source(app_state.clone(), message_event, message));
        }
    }
}

async fn update_source (app_state: AppState, event: MessageEvent, message: Message) {

    if message.data.dataset_name == "harmonie_arome_cy43_p1" {
        app_state.arome.update_model(message.data).await;
    } else if  message.data.dataset_name == "harmonie_arome_cy43_p3" {

    } else if  message.data.dataset_name == "10-minute-in-situ-meteorological-observations" {

    } else {
        tracing::warn!("Unkown dataset: {}", message.data.dataset_name);
    }
}