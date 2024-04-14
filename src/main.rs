use tokio::{task, signal};
use tokio::net::TcpListener;
use listenfd::ListenFd;
use axum::{
    response::Html,
    routing::get,
    Router,
};
use tracing_subscriber::prelude::*;
use tracing::{error};

mod knmi;
mod config;

#[derive(Clone)]
pub struct AppState {
    // config: config::Config,
    arome: knmi::models::arome::Arome,
}

#[tokio::main]
async fn main() {

    // construct a subscriber that prints formatted traces to stdout
    // let console_layer = console_subscriber::spawn();
    // tracing_subscriber::registry()
    //     .with(console_layer)
    //     .with(tracing_subscriber::fmt::layer().with_filter(tracing_subscriber::filter::LevelFilter::INFO))
    //     .init();
    // use that subscriber to process traces emitted after this point
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap(); 

    let state = AppState {
        // config: config::load_config(),
        arome: knmi::models::arome::Arome::new(),
    };

    let port = config::CONFIG.server.port;

    match knmi::files::list_latest_files().await {
        Ok(f) => {
            println!("{:?}", f);
        },
        Err(_) => {
            error!("Failed to fetch latest files.");
        }
    };

    task::spawn(knmi::notifications::sub_knmi_notifications(state.clone()));

    let app = Router::new()
        .route("/", get(handler))
        // .route("/list", get(knmi::files::pull_with_reqwest))
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