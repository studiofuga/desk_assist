use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::OllamaConfig;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
}

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn generate_text(&self, text: &str) -> Result<String> {
        let prompt = format!(
            "Please summarize and extract the key information from the following text. \
             Focus on the main topics, concepts, and important details:\n\n{}", 
            text
        );

        let request = GenerateRequest {
            model: self.config.llm_model.clone(),
            prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }

        let generate_response: GenerateResponse = response.json().await?;
        Ok(generate_response.response)
    }

    pub async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            model: self.config.embedding_model.clone(),
            prompt: text.to_string(),
        };

        let url = format!("{}/api/embeddings", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Ollama embedding API error: {}", error_text));
        }

        let embedding_response: EmbeddingResponse = response.json().await?;
        Ok(embedding_response.embedding)
    }
}