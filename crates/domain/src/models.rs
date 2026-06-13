use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "jurisdiction", rename_all = "lowercase")]
pub enum Jurisdiction {
    US,
    EU,
    CA,
    UK,
    AU,
    SG,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "regulation_category", rename_all = "snake_case")]
pub enum RegulationCategory {
    FinancialServices,
    DataProtection,
    AntiMoneyLaundering,
    Sanctions,
    ConsumerProtection,
    Securities,
    Banking,
    Insurance,
    Crypto,
    ESG,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "change_type", rename_all = "snake_case")]
pub enum ChangeType {
    NewRegulation,
    Amendment,
    Repeal,
    Guidance,
    EnforcementAction,
    ProposedRule,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "severity", rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryChange {
    pub id: Uuid,
    pub source_id: Uuid,
    pub jurisdiction: Jurisdiction,
    pub category: RegulationCategory,
    pub change_type: ChangeType,
    pub severity: Severity,
    pub title: String,
    pub summary: String,
    pub full_text_url: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub comment_deadline: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub affected_entities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatorySource {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub source_type: String,
    pub jurisdiction: Jurisdiction,
    pub categories: Vec<RegulationCategory>,
    pub last_checked: Option<DateTime<Utc>>,
    pub check_frequency_hours: u64,
    pub is_active: bool,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTask {
    pub id: Uuid,
    pub regulatory_change_id: Uuid,
    pub tenant_id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: Severity,
    pub status: TaskStatus,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub evidence_required: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    InReview,
    Completed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub industry: String,
    pub jurisdictions: Vec<Jurisdiction>,
    pub categories_of_interest: Vec<RegulationCategory>,
    pub notification_preferences: NotificationPreferences,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email: bool,
    pub slack: bool,
    pub webhook_url: Option<String>,
    pub severity_threshold: Severity,
    pub digest_frequency: DigestFrequency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "digest_frequency", rename_all = "snake_case")]
pub enum DigestFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: AgentRunStatus,
    pub sources_checked: usize,
    pub changes_found: usize,
    pub tasks_created: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "agent_run_status", rename_all = "snake_case")]
pub enum AgentRunStatus {
    Running,
    Completed,
    Failed,
    Partial,
}