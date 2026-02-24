use tokio::{task, signal};
use tokio::net::TcpListener;
use listenfd::ListenFd;
use axum::{
    routing::get,
    Router,
};

mod util;
mod knmi;
mod config;

#[derive(Clone)]
pub struct AppState {
    // config: config::Config,
    arome: knmi::models::arome::Arome,
}

#[tokio::main]
async fn main() {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap(); 

    let arome = knmi::models::arome::Arome::new().await;

    let state = AppState {
        arome,
    };

    let port = config::CONFIG.server.port;

    task::spawn(knmi::notifications::sub_knmi_notifications(state.clone()));

    let app = Router::new()
        .route("/download", get(knmi::download::download))
        .route("/weather/knmi-arome", get(knmi::arome::forecast))
        .with_state(state);

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