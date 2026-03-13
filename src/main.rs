use tokio::{task, signal};
use tokio::net::TcpListener;
use listenfd::ListenFd;
use axum::Router;

use crate::knmi::api::Api;
use crate::knmi::sources::{KnmiSource, load_sources_from_config};

mod util;
mod knmi;
mod config;

#[derive(Clone)]
pub struct AppState {
    sources: Vec<KnmiSource>
}

#[tokio::main]
async fn main() {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap(); 

    let sources = load_sources_from_config();

    let state = AppState {
        sources,
    };

    let port = config::CONFIG.server.port;

    task::spawn(knmi::notifications::sub_knmi_notifications(state.clone()));

    let mut app = Router::new();

    for source in state.sources.iter() {
        app = source.set_route(app, state.clone());
    }

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        None => TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap(),
    };

    tracing::info!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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