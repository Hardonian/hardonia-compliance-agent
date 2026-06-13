use compliance_shared::{ComplianceError, Result};
use async_trait::async_trait;
use compliance_domain::{Jurisdiction, RegulatoryChange, RegulatorySource, RegulationCategory, Severity};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RecallIntelligenceClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl RecallIntelligenceClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn fetch_regulatory_changes(
        &self,
        jurisdiction: Jurisdiction,
        categories: &[RegulationCategory],
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RegulatoryChange>> {
        let mut params = HashMap::new();
        params.insert("jurisdiction", format!("{:?}", jurisdiction).to_lowercase());
        params.insert("categories", serde_json::to_string(categories)?);
        
        if let Some(since) = since {
            params.insert("since", since.to_rfc3339());
        }

        let response = self.client
            .get(&format!("{}/api/v1/regulatory-changes", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ComplianceError::Integration(format!(
                "Recall Intelligence API error: {}", response.status()
            )));
        }

        let changes: Vec<RegulatoryChange> = response.json().await?;
        Ok(changes)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, Clone)]
pub struct TradeComplianceClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl TradeComplianceClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn fetch_sanctions_updates(
        &self,
        jurisdiction: Jurisdiction,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RegulatoryChange>> {
        let mut params = HashMap::new();
        params.insert("jurisdiction", format!("{:?}", jurisdiction).to_lowercase());
        
        if let Some(since) = since {
            params.insert("since", since.to_rfc3339());
        }

        let response = self.client
            .get(&format!("{}/api/v1/sanctions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ComplianceError::Integration(format!(
                "Trade Compliance API error: {}", response.status()
            )));
        }

        let changes: Vec<RegulatoryChange> = response.json().await?;
        Ok(changes)
    }

    pub async fn fetch_trade_regulations(
        &self,
        jurisdiction: Jurisdiction,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RegulatoryChange>> {
        let mut params = HashMap::new();
        params.insert("jurisdiction", format!("{:?}", jurisdiction).to_lowercase());
        
        if let Some(since) = since {
            params.insert("since", since.to_rfc3339());
        }

        let response = self.client
            .get(&format!("{}/api/v1/trade-regulations", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ComplianceError::Integration(format!(
                "Trade Compliance API error: {}", response.status()
            )));
        }

        let changes: Vec<RegulatoryChange> = response.json().await?;
        Ok(changes)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, Clone)]
pub struct SettlerClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl SettlerClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn create_compliance_task(&self, task: &ComplianceTaskRequest) -> Result<Uuid> {
        let response = self.client
            .post(&format!("{}/api/v1/compliance/tasks", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(task)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ComplianceError::Integration(format!(
                "Settler API error: {}", response.status()
            )));
        }

        let result: TaskResponse = response.json().await?;
        Ok(result.id)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, Serialize)]
pub struct ComplianceTaskRequest {
    pub regulatory_change_id: Uuid,
    pub tenant_id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: Severity,
    pub due_date: Option<DateTime<Utc>>,
    pub evidence_required: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
}

#[async_trait]
pub trait RegulatorySourceClient: Send + Sync {
    async fn fetch_changes(&self, source: &RegulatorySource) -> Result<Vec<RegulatoryChange>>;
    async fn health_check(&self) -> Result<bool>;
}

pub fn create_client_for_source(
    source: &RegulatorySource,
    recall_client: Option<&RecallIntelligenceClient>,
    trade_client: Option<&TradeComplianceClient>,
) -> Option<Box<dyn RegulatorySourceClient>> {
    match source.source_type.as_str() {
        "recall_intelligence" => recall_client.map(|c| Box::new(c.clone()) as Box<dyn RegulatorySourceClient>),
        "trade_compliance" => trade_client.map(|c| Box::new(c.clone()) as Box<dyn RegulatorySourceClient>),
        _ => None,
    }
}

#[async_trait]
impl RegulatorySourceClient for RecallIntelligenceClient {
    async fn fetch_changes(&self, source: &RegulatorySource) -> Result<Vec<RegulatoryChange>> {
        self.fetch_regulatory_changes(source.jurisdiction, &source.categories, source.last_checked).await
    }

    async fn health_check(&self) -> Result<bool> {
        self.health_check().await
    }
}

#[async_trait]
impl RegulatorySourceClient for TradeComplianceClient {
    async fn fetch_changes(&self, source: &RegulatorySource) -> Result<Vec<RegulatoryChange>> {
        // For trade compliance, fetch both sanctions and trade regulations
        let mut all_changes = Vec::new();
        
        let sanctions = self.fetch_sanctions_updates(source.jurisdiction, source.last_checked).await?;
        all_changes.extend(sanctions);
        
        let trade_regs = self.fetch_trade_regulations(source.jurisdiction, source.last_checked).await?;
        all_changes.extend(trade_regs);
        
        Ok(all_changes)
    }

    async fn health_check(&self) -> Result<bool> {
        self.health_check().await
    }
}