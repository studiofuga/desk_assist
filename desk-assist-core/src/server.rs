use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::{
    chunker::TextChunker,
    extractor::TextExtractor,
    ollama::OllamaClient,
    storage::QdrantStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub ollama: Arc<OllamaClient>,
    pub storage: Arc<QdrantStorage>,
}

#[derive(Deserialize)]
pub struct IngestQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub success: bool,
    pub message: String,
    pub chunks_processed: usize,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/ingest", post(ingest_handler))
        .with_state(state)
}

pub async fn ingest_handler(
    State(state): State<AppState>,
    Query(params): Query<IngestQuery>,
) -> Result<Json<IngestResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("Processing ingest request for path: {}", params.path);

    let text = match TextExtractor::extract_text(&params.path) {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("Failed to extract text from {}: {}", params.path, e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to extract text: {}", e),
                }),
            ));
        }
    };

    if text.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No text content found in file".to_string(),
            }),
        ));
    }

    let llm_summary = match state.ollama.generate_text(&text).await {
        Ok(summary) => summary,
        Err(e) => {
            tracing::error!("Failed to generate summary: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to generate summary: {}", e),
                }),
            ));
        }
    };

    let chunks = TextChunker::chunk_text(&text);
    tracing::info!("Generated {} chunks from text", chunks.len());

    let mut embeddings = Vec::new();
    for chunk in &chunks {
        match state.ollama.generate_embeddings(chunk).await {
            Ok(embedding) => embeddings.push(embedding),
            Err(e) => {
                tracing::error!("Failed to generate embedding: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to generate embeddings: {}", e),
                    }),
                ));
            }
        }
    }

    match state
        .storage
        .store_embeddings(&chunks, &embeddings, &params.path, &llm_summary)
        .await
    {
        Ok(_) => {
            tracing::info!(
                "Successfully processed {} chunks for {}",
                chunks.len(),
                params.path
            );
            Ok(Json(IngestResponse {
                success: true,
                message: format!("Successfully processed {} chunks", chunks.len()),
                chunks_processed: chunks.len(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to store embeddings: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to store embeddings: {}", e),
                }),
            ))
        }
    }
}