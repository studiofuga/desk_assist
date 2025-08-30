use anyhow::Result;
use qdrant_client::{
    Qdrant,
    qdrant::{
        vectors_config::Config, CreateCollection, Distance, PointStruct, UpsertPoints,
        VectorParams, VectorsConfig,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;
use crate::config::QdrantConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_hash: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub processed_at: String,
    pub llm_summary: String,
}

pub struct QdrantStorage {
    client: Qdrant,
    collection_name: String,
}

impl QdrantStorage {
    pub async fn new(config: &QdrantConfig) -> Result<Self> {
        let mut m = Qdrant::from_url(&config.url);
        m.check_compatibility = false;
        let client= m.build()?;

        let storage = Self {
            client,
            collection_name: config.collection_name.clone(),
        };
        
        storage.ensure_collection_exists().await?;
        Ok(storage)
    }

    async fn ensure_collection_exists(&self) -> Result<()> {
        let collections = self.client.list_collections().await?;
        
        let collection_exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection_name);

        if !collection_exists {
            let create_collection = CreateCollection {
                collection_name: self.collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 768, // Default for nomic-embed-text
                        distance: Distance::Cosine as i32,
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            };

            self.client.create_collection(create_collection).await?;
            tracing::info!("Created collection: {}", self.collection_name);
        }

        Ok(())
    }

    pub async fn store_embeddings(
        &self,
        chunks: &[String],
        embeddings: &[Vec<f32>],
        file_path: &str,
        llm_summary: &str,
    ) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(anyhow::anyhow!(
                "Chunks and embeddings length mismatch: {} vs {}",
                chunks.len(),
                embeddings.len()
            ));
        }

        let file_path_obj = std::path::Path::new(file_path);
        let file_name = file_path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_metadata = std::fs::metadata(file_path)?;
        let file_size = file_metadata.len();

        let file_content = std::fs::read(file_path)?;
        let file_hash = format!("{:x}", Sha256::digest(&file_content));

        let mut points = Vec::new();

        for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            let metadata = DocumentMetadata {
                file_path: file_path.to_string(),
                file_name: file_name.clone(),
                file_size,
                file_hash: file_hash.clone(),
                chunk_index: i,
                total_chunks: chunks.len(),
                processed_at: chrono::Utc::now().to_rfc3339(),
                llm_summary: llm_summary.to_string(),
            };

            let mut payload = HashMap::new();
            payload.insert("text".to_string(), chunk.clone().into());
            payload.insert(
                "metadata".to_string(),
                serde_json::to_value(&metadata)?.into(),
            );

            let point = PointStruct::new(
                Uuid::new_v4().to_string(),
                embedding.clone(),
                payload,
            );

            points.push(point);
        }

        let upsert_points = UpsertPoints {
            collection_name: self.collection_name.clone(),
            points,
            ..Default::default()
        };

        self.client.upsert_points(upsert_points).await?;
        tracing::info!(
            "Stored {} chunks with embeddings for file: {}",
            chunks.len(),
            file_path
        );

        Ok(())
    }

    pub async fn search_similar(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(String, f32, DocumentMetadata)>> {
        let search_points = qdrant_client::qdrant::SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: query_embedding,
            limit: limit as u64,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        let search_result = self.client.search_points(search_points).await?;

        let mut results = Vec::new();
        for point in search_result.result {
            if let (Some(text_value), Some(metadata_value)) = 
                (point.payload.get("text"), point.payload.get("metadata")) 
            {
                let text = text_value.to_string();
                let metadata: DocumentMetadata = 
                    serde_json::from_str(&metadata_value.to_string())?;
                results.push((text, point.score, metadata));
            }
        }

        Ok(results)
    }
}