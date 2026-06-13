use async_trait::async_trait;
use compliance_domain::{AgentRun, AgentRunStatus, ComplianceTask, RegulatoryChange, RegulatorySource, Tenant};
use compliance_shared::{ComplianceError, Config, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

pub struct ComplianceAgent {
    config: Arc<Config>,
    db_pool: PgPool,
    http_client: Client,
    openai_api_key: String,
    openai_model: String,
}

impl ComplianceAgent {
    pub fn new(config: Arc<Config>, db_pool: PgPool) -> Self {
        let http_client = Client::new();
        let openai_api_key = config.llm.api_key.clone();
        let openai_model = config.llm.model.clone();
        
        Self {
            config,
            db_pool,
            http_client,
            openai_api_key,
            openai_model,
        }
    }

    pub async fn run_scheduled_check(&self) -> Result<AgentRun> {
        let run_id = Uuid::new_v4();
        let started_at = chrono::Utc::now();
        
        let run = AgentRun {
            id: run_id,
            started_at,
            completed_at: None,
            status: AgentRunStatus::Running,
            sources_checked: 0,
            changes_found: 0,
            tasks_created: 0,
            errors: Vec::new(),
        };

        // Save initial run state
        self.save_agent_run(&run).await?;

        let mut total_changes = 0;
        let mut total_tasks = 0;
        let mut errors = Vec::new();

        // Fetch all active regulatory sources
        let sources = self.get_active_sources().await?;
        let sources_count = sources.len();
        
        for source in sources {
            // Check each source for changes
            match self.check_source(&source).await {
                Ok(changes) => {
                    total_changes += changes.len();
                    
                    // Process each change: analyze relevance and create tasks
                    for change in changes {
                        let tasks = self.analyze_and_create_tasks(&change).await?;
                        total_tasks += tasks.len();
                    }
                }
                Err(e) => {
                    errors.push(format!("Source {}: {}", source.name, e));
                }
            }
        }

        let completed_at = chrono::Utc::now();
        let final_run = AgentRun {
            id: run_id,
            started_at,
            completed_at: Some(completed_at),
            status: if errors.is_empty() { AgentRunStatus::Completed } else { AgentRunStatus::Partial },
            sources_checked: sources_count,
            changes_found: total_changes,
            tasks_created: total_tasks,
            errors,
        };

        self.update_agent_run(&final_run).await?;
        Ok(final_run)
    }

    async fn check_source(&self, _source: &RegulatorySource) -> Result<Vec<RegulatoryChange>> {
        // This would use the integration clients to fetch changes
        // For now, return empty - implementation uses compliance_integrations crate
        Ok(Vec::new())
    }

    async fn analyze_and_create_tasks(&self, change: &RegulatoryChange) -> Result<Vec<ComplianceTask>> {
        // Use LLM to analyze the change for relevance to each tenant
        // and generate specific compliance tasks
        let prompt = self.build_analysis_prompt(change);
        
        let response = self.call_openai(&prompt).await?;
        
        // Parse response and create tasks
        // For now, return empty
        Ok(Vec::new())
    }

    fn build_analysis_prompt(&self, change: &RegulatoryChange) -> String {
        format!(
            r#"Analyze this regulatory change for compliance relevance:

Title: {}
Summary: {}
Jurisdiction: {:?}
Category: {:?}
Change Type: {:?}
Severity: {:?}
Effective Date: {:?}
Affected Entities: {:?}

For each of our tenants (fintech, healthtech, insurtech companies), determine:
1. Is this change relevant to their industry and jurisdictions?
2. What specific compliance tasks are required?
3. What is the priority and deadline?
4. What evidence is needed to demonstrate compliance?

Return a JSON array of tasks with: tenant_id, title, description, priority, due_date, evidence_required.
"#,
            change.title,
            change.summary,
            change.jurisdiction,
            change.category,
            change.change_type,
            change.severity,
            change.effective_date,
            change.affected_entities
        )
    }

    async fn call_openai(&self, prompt: &str) -> Result<String> {
        let request = OpenAIRequest {
            model: self.openai_model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a regulatory compliance expert. Analyze regulatory changes and generate specific compliance tasks.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: self.config.llm.temperature,
            max_tokens: self.config.llm.max_tokens,
        };

        let response = self.http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| ComplianceError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ComplianceError::Llm(format!("OpenAI API error: {}", error_text)));
        }

        let openai_response: OpenAIResponse = response.json().await
            .map_err(|e| ComplianceError::Llm(e.to_string()))?;

        openai_response.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| ComplianceError::Llm("No response from OpenAI".to_string()))
    }

    async fn get_active_sources(&self) -> Result<Vec<RegulatorySource>> {
        let rows = sqlx::query("SELECT * FROM regulatory_sources WHERE is_active = true")
            .fetch_all(&self.db_pool)
            .await?;

        let mut sources = Vec::new();
        for _row in rows {
            // Parse row into RegulatorySource
        }

        Ok(sources)
    }

    async fn get_tenants_for_change(&self, _change: &RegulatoryChange) -> Result<Vec<Tenant>> {
        Ok(Vec::new())
    }

    async fn save_agent_run(&self, run: &AgentRun) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO agent_runs (id, started_at, completed_at, status, sources_checked, changes_found, tasks_created, errors)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#
        )
        .bind(run.id)
        .bind(run.started_at)
        .bind(run.completed_at)
        .bind(run.status)
        .bind(run.sources_checked as i32)
        .bind(run.changes_found as i32)
        .bind(run.tasks_created as i32)
        .bind(serde_json::to_value(&run.errors)?)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_agent_run(&self, run: &AgentRun) -> Result<()> {
        sqlx::query(
            r#"UPDATE agent_runs SET completed_at = $1, status = $2, sources_checked = $3, changes_found = $4, tasks_created = $5, errors = $6
               WHERE id = $7"#
        )
        .bind(run.completed_at)
        .bind(run.status)
        .bind(run.sources_checked as i32)
        .bind(run.changes_found as i32)
        .bind(run.tasks_created as i32)
        .bind(serde_json::to_value(&run.errors)?)
        .bind(run.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
pub trait ComplianceAgentTrait: Send + Sync {
    async fn run_scheduled_check(&self) -> Result<AgentRun>;
}

#[async_trait]
impl ComplianceAgentTrait for ComplianceAgent {
    async fn run_scheduled_check(&self) -> Result<AgentRun> {
        self.run_scheduled_check().await
    }
}