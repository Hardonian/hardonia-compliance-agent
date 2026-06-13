use crate::ComplianceError;
use async_trait::async_trait;
use compliance_domain::{RegulatorySource, AgentRun, Tenant, ComplianceTask, RegulatoryChange, TaskStatus};
use compliance_shared::ComplianceError as SharedComplianceError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait RegulatorySourceRepository: Send + Sync {
    async fn get_active(&self) -> Result<Vec<RegulatorySource>, ComplianceError>;
    async fn update_last_checked(&self, id: Uuid, checked_at: DateTime<Utc>) -> Result<(), ComplianceError>;
}

#[async_trait]
pub trait AgentRunRepository: Send + Sync {
    async fn create(&self, run: &AgentRun) -> Result<(), ComplianceError>;
    async fn update(&self, run: &AgentRun) -> Result<(), ComplianceError>;
    async fn get_latest(&self, limit: i64) -> Result<Vec<AgentRun>, ComplianceError>;
}

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn get_by_jurisdiction_and_category(&self, jurisdiction: &str, category: &str) -> Result<Vec<Tenant>, ComplianceError>;
}

#[async_trait]
pub trait ComplianceTaskRepository: Send + Sync {
    async fn create(&self, task: &ComplianceTask) -> Result<(), ComplianceError>;
    async fn get_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<ComplianceTask>, ComplianceError>;
    async fn update_status(&self, task_id: Uuid, status: TaskStatus) -> Result<(), ComplianceError>;
}

#[async_trait]
pub trait RegulatoryChangeRepository: Send + Sync {
    async fn upsert(&self, change: &RegulatoryChange) -> Result<(), ComplianceError>;
    async fn get_unprocessed(&self) -> Result<Vec<RegulatoryChange>, ComplianceError>;
}

#[derive(Clone)]
pub struct PostgresRepositories {
    pool: PgPool,
}

impl PostgresRepositories {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RegulatorySourceRepository for PostgresRepositories {
    async fn get_active(&self) -> Result<Vec<RegulatorySource>, ComplianceError> {
        Ok(Vec::new())
    }

    async fn update_last_checked(&self, id: Uuid, checked_at: DateTime<Utc>) -> Result<(), ComplianceError> {
        sqlx::query("UPDATE regulatory_sources SET last_checked = $1 WHERE id = $2")
            .bind(checked_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl AgentRunRepository for PostgresRepositories {
    async fn create(&self, run: &AgentRun) -> Result<(), ComplianceError> {
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
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, run: &AgentRun) -> Result<(), ComplianceError> {
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
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_latest(&self, limit: i64) -> Result<Vec<AgentRun>, ComplianceError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl TenantRepository for PostgresRepositories {
    async fn get_by_jurisdiction_and_category(&self, _jurisdiction: &str, _category: &str) -> Result<Vec<Tenant>, ComplianceError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl ComplianceTaskRepository for PostgresRepositories {
    async fn create(&self, task: &ComplianceTask) -> Result<(), ComplianceError> {
        sqlx::query(
            r#"INSERT INTO compliance_tasks (id, regulatory_change_id, tenant_id, title, description, priority, status, assigned_to, due_date, completed_at, evidence_required, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#
        )
        .bind(task.id)
        .bind(task.regulatory_change_id)
        .bind(task.tenant_id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.priority)
        .bind(task.status)
        .bind(task.assigned_to)
        .bind(task.due_date)
        .bind(task.completed_at)
        .bind(serde_json::to_value(&task.evidence_required)?)
        .bind(task.created_at)
        .bind(task.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_tenant(&self, _tenant_id: Uuid) -> Result<Vec<ComplianceTask>, ComplianceError> {
        Ok(Vec::new())
    }

    async fn update_status(&self, task_id: Uuid, status: TaskStatus) -> Result<(), ComplianceError> {
        sqlx::query("UPDATE compliance_tasks SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl RegulatoryChangeRepository for PostgresRepositories {
    async fn upsert(&self, _change: &RegulatoryChange) -> Result<(), ComplianceError> {
        Ok(())
    }

    async fn get_unprocessed(&self) -> Result<Vec<RegulatoryChange>, ComplianceError> {
        Ok(Vec::new())
    }
}