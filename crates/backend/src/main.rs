use hauski_backend::build_router;
use hauski_backend::config::AppConfig;
use hauski_backend::error::AppError;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    dotenvy::dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();

    let config = AppConfig::from_env()
        .map_err(|err| AppError::Startup(format!("failed to load configuration: {err}")))?;

    config.validate()?;

    let bind_addr = config.bind_addr;
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|err| AppError::Startup(format!("failed to bind to {bind_addr}: {err}")))?;

    info!("listening on {bind_addr}");

    axum::serve(listener, build_router(config))
        .await
        .map_err(|err| AppError::Startup(format!("server error: {err}")))?;

    Ok(())
}
