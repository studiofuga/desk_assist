use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub ollama: OllamaConfig,
    pub qdrant: QdrantConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub llm_model: String,
    pub embedding_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            ollama: OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                llm_model: "llama3.2".to_string(),
                embedding_model: "nomic-embed-text".to_string(),
            },
            qdrant: QdrantConfig {
                url: "http://localhost:6334".to_string(),
                collection_name: "documents".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("DESK_ASSIST"))
            .build()?;

        let config = settings.try_deserialize().unwrap_or_default();
        Ok(config)
    }
}