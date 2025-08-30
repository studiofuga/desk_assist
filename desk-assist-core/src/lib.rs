pub mod config;
pub mod chunker;
pub mod extractor;
pub mod ollama;
pub mod server;
pub mod storage;

pub use config::Config;
pub use server::{create_router, AppState};

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = Config::load()?;
    tracing::info!("Loaded configuration: {:?}", config);

    let ollama_client = Arc::new(ollama::OllamaClient::new(config.ollama.clone()));
    let qdrant_storage = Arc::new(storage::QdrantStorage::new(&config.qdrant).await?);

    let app_state = AppState {
        ollama: ollama_client,
        storage: qdrant_storage,
    };

    let app = create_router(app_state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    
    tracing::info!("DeskAssist Core server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}