use hauski_backend::build_router;
use hauski_backend::config::AppConfig;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();

    let config = match AppConfig::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("failed to load configuration: {err}");
            std::process::exit(1);
        }
    };

    let bind_addr = config.bind_addr;
    let listener = match TcpListener::bind(bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!("failed to bind to {bind_addr}: {err}");
            std::process::exit(1);
        }
    };

    info!("listening on {bind_addr}");

    if let Err(err) = axum::serve(listener, build_router(config)).await {
        error!("server error: {err}");
    }
}
