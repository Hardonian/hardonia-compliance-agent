// Application state
use compliance_agent::ComplianceAgent;
use compliance_db::PostgresRepositories;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub agent: Arc<ComplianceAgent>,
    pub db: PostgresRepositories,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let config = Arc::new(compliance_shared::Config::load().expect("Failed to load config"));
        let repos = PostgresRepositories::new(pool.clone());
        let agent = Arc::new(ComplianceAgent::new(config, pool));
        Self { agent, db: repos }
    }
}