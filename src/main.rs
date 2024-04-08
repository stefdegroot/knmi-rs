use tokio::signal;
use tokio::net::TcpListener;
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    body::{Body, Bytes},
    response::{IntoResponse, Response, Html},
    routing::get,
    Router,
    Json
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ndarray::{Array3};

mod knmi;

pub type NCMap = Arc<Mutex<HashMap<String, Array3<f64>>>>;

#[derive(Clone)]
pub struct AppState {
    nc_map: NCMap
}

#[tokio::main]
async fn main() {

    let state = AppState {
        nc_map: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(handler))
        .route("/list", get(knmi::files::pull_with_reqwest))
        .route("/download", get(knmi::download::download))
        .route("/weather/knmi-arome", get(knmi::arome::forecast))
        .with_state(state);

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        None => TcpListener::bind("127.0.0.1:3000").await.unwrap(),
    };

    // let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}