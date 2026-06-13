use serde::{Deserialize, Serialize};
use figment::providers::Format;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub llm: LlmConfig,
    pub integrations: IntegrationsConfig,
    pub agent: AgentConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntegrationsConfig {
    pub recall_intelligence_api: Option<ApiIntegrationConfig>,
    pub trade_compliance_api: Option<ApiIntegrationConfig>,
    pub settler_api: Option<ApiIntegrationConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiIntegrationConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub check_interval_hours: u64,
    pub max_concurrent_tasks: usize,
    pub jurisdictions: Vec<String>,
    pub regulatory_sources: Vec<RegulatorySource>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegulatorySource {
    pub name: String,
    pub url: String,
    pub source_type: String,
    pub categories: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
            },
            database: DatabaseConfig {
                url: "postgres://localhost/compliance_agent".to_string(),
                max_connections: 20,
                min_connections: 5,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 10,
            },
            llm: LlmConfig {
                provider: "openai".to_string(),
                model: "gpt-4o".to_string(),
                api_key: "".to_string(),
                base_url: None,
                temperature: 0.1,
                max_tokens: 4096,
            },
            integrations: IntegrationsConfig {
                recall_intelligence_api: None,
                trade_compliance_api: None,
                settler_api: None,
            },
            agent: AgentConfig {
                check_interval_hours: 24,
                max_concurrent_tasks: 10,
                jurisdictions: vec!["US".to_string(), "EU".to_string(), "CA".to_string()],
                regulatory_sources: vec![],
            },
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = Config::default();
        
        if let Ok(figment) = figment::Figment::new()
            .merge(figment::providers::Toml::file("config.toml"))
            .merge(figment::providers::Env::prefixed("COMPLIANCE_"))
            .extract()
        {
            config = figment;
        }
        
        Ok(config)
    }
}