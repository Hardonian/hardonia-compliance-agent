// Application state
use compliance_agent::ComplianceAgent;
use compliance_db::PostgresRepositories;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub agent: Arc<Mutex<ComplianceAgent>>,
    pub db: PostgresRepositories,
    pub db_pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let config = Arc::new(compliance_shared::Config::load().expect("Failed to load config"));
        let repos = PostgresRepositories::new(pool.clone());
        let agent = Arc::new(Mutex::new(ComplianceAgent::new(config, pool.clone())));
        Self { agent, db: repos, db_pool: pool }
    }
}