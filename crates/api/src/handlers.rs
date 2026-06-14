use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/agent/run", post(run_agent_check))
        .route("/api/v1/agent/runs", get(list_agent_runs))
        .route("/api/v1/agent/runs/{run_id}", get(get_agent_run))
        .route("/api/v1/sources", get(list_regulatory_sources))
        .route("/api/v1/sources", post(create_regulatory_source))
        .route("/api/v1/changes", get(list_regulatory_changes))
        .route("/api/v1/tasks", get(list_compliance_tasks))
        .route("/api/v1/tasks/{task_id}", get(get_compliance_task))
        .route("/api/v1/tasks/{task_id}/status", post(update_task_status))
        .route("/api/v1/dashboard/summary", get(dashboard_summary))
        .route("/api/v1/dashboard/compliance-score", get(compliance_score))
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "compliance-agent" }))
}

async fn run_agent_check(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let agent = state.agent.lock().await;
    match agent.run_scheduled_check().await {
        Ok(run) => Ok(Json(json!({
            "status": "ok",
            "run_id": run.id,
            "sources_checked": run.sources_checked,
            "changes_found": run.changes_found,
            "tasks_created": run.tasks_created,
            "errors": run.errors,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

async fn list_agent_runs(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let rows = sqlx::query(
        "SELECT id, started_at, completed_at, status, sources_checked, changes_found, tasks_created, errors
         FROM agent_runs ORDER BY started_at DESC LIMIT 50"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    let runs: Vec<Value> = rows.iter().map(|row| {
        json!({
            "id": row.get::<Uuid, _>("id"),
            "started_at": row.get::<chrono::DateTime<chrono::Utc>, _>("started_at"),
            "completed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at"),
            "status": row.get::<String, _>("status"),
            "sources_checked": row.get::<i32, _>("sources_checked"),
            "changes_found": row.get::<i32, _>("changes_found"),
            "tasks_created": row.get::<i32, _>("tasks_created"),
            "errors": row.get::<Value, _>("errors"),
        })
    }).collect();

    Ok(Json(json!({ "runs": runs })))
}

async fn get_agent_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let row = sqlx::query(
        "SELECT id, started_at, completed_at, status, sources_checked, changes_found, tasks_created, errors
         FROM agent_runs WHERE id = $1"
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    match row {
        Some(row) => Ok(Json(json!({
            "id": row.get::<Uuid, _>("id"),
            "started_at": row.get::<chrono::DateTime<chrono::Utc>, _>("started_at"),
            "completed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at"),
            "status": row.get::<String, _>("status"),
            "sources_checked": row.get::<i32, _>("sources_checked"),
            "changes_found": row.get::<i32, _>("changes_found"),
            "tasks_created": row.get::<i32, _>("tasks_created"),
            "errors": row.get::<Value, _>("errors"),
        }))),
        None => Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Run not found" })))),
    }
}

async fn list_regulatory_sources(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let rows = sqlx::query(
        "SELECT id, name, source_type, jurisdiction, categories, is_active, last_checked, last_error
         FROM regulatory_sources ORDER BY name"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    let sources: Vec<Value> = rows.iter().map(|row| {
        json!({
            "id": row.get::<Uuid, _>("id"),
            "name": row.get::<String, _>("name"),
            "source_type": row.get::<String, _>("source_type"),
            "jurisdiction": row.get::<String, _>("jurisdiction"),
            "categories": row.get::<Value, _>("categories"),
            "is_active": row.get::<bool, _>("is_active"),
            "last_checked": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_checked"),
            "last_error": row.get::<Option<String>, _>("last_error"),
        })
    }).collect();

    Ok(Json(json!({ "sources": sources })))
}

async fn create_regulatory_source(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let id = Uuid::new_v4();
    let name = payload["name"].as_str().unwrap_or("");
    let source_type = payload["source_type"].as_str().unwrap_or("");
    let jurisdiction = payload["jurisdiction"].as_str().unwrap_or("us");
    let categories = payload["categories"].as_array().map(|a| json!(a)).unwrap_or(json!([]));

    sqlx::query(
        "INSERT INTO regulatory_sources (id, name, source_type, jurisdiction, categories, is_active)
         VALUES ($1, $2, $3, $4, $5, true)"
    )
    .bind(id)
    .bind(name)
    .bind(source_type)
    .bind(jurisdiction)
    .bind(categories)
    .execute(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(json!({ "id": id, "status": "created" })))
}

async fn list_regulatory_changes(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let rows = sqlx::query(
        "SELECT id, title, summary, jurisdiction, category, change_type, severity, effective_date, affected_entities, source_id, fetched_at
         FROM regulatory_changes ORDER BY fetched_at DESC LIMIT 100"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    let changes: Vec<Value> = rows.iter().map(|row| {
        json!({
            "id": row.get::<Uuid, _>("id"),
            "title": row.get::<String, _>("title"),
            "summary": row.get::<String, _>("summary"),
            "jurisdiction": row.get::<String, _>("jurisdiction"),
            "category": row.get::<String, _>("category"),
            "change_type": row.get::<String, _>("change_type"),
            "severity": row.get::<String, _>("severity"),
            "effective_date": row.get::<Option<chrono::NaiveDate>, _>("effective_date"),
            "affected_entities": row.get::<Value, _>("affected_entities"),
            "source_id": row.get::<Uuid, _>("source_id"),
            "fetched_at": row.get::<chrono::DateTime<chrono::Utc>, _>("fetched_at"),
        })
    }).collect();

    Ok(Json(json!({ "changes": changes })))
}

async fn list_compliance_tasks(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let rows = sqlx::query(
        "SELECT id, tenant_id, regulatory_change_id, title, description, priority, status, due_date, evidence_required, assigned_to, completed_at
         FROM compliance_tasks ORDER BY due_date ASC LIMIT 100"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    let tasks: Vec<Value> = rows.iter().map(|row| {
        json!({
            "id": row.get::<Uuid, _>("id"),
            "tenant_id": row.get::<Uuid, _>("tenant_id"),
            "regulatory_change_id": row.get::<Uuid, _>("regulatory_change_id"),
            "title": row.get::<String, _>("title"),
            "description": row.get::<String, _>("description"),
            "priority": row.get::<String, _>("priority"),
            "status": row.get::<String, _>("status"),
            "due_date": row.get::<Option<chrono::NaiveDate>, _>("due_date"),
            "evidence_required": row.get::<Value, _>("evidence_required"),
            "assigned_to": row.get::<Option<String>, _>("assigned_to"),
            "completed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at"),
        })
    }).collect();

    Ok(Json(json!({ "tasks": tasks })))
}

async fn get_compliance_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let row = sqlx::query(
        "SELECT id, tenant_id, regulatory_change_id, title, description, priority, status, due_date, evidence_required, assigned_to, completed_at
         FROM compliance_tasks WHERE id = $1"
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    match row {
        Some(row) => Ok(Json(json!({
            "id": row.get::<Uuid, _>("id"),
            "tenant_id": row.get::<Uuid, _>("tenant_id"),
            "regulatory_change_id": row.get::<Uuid, _>("regulatory_change_id"),
            "title": row.get::<String, _>("title"),
            "description": row.get::<String, _>("description"),
            "priority": row.get::<String, _>("priority"),
            "status": row.get::<String, _>("status"),
            "due_date": row.get::<Option<chrono::NaiveDate>, _>("due_date"),
            "evidence_required": row.get::<Value, _>("evidence_required"),
            "assigned_to": row.get::<Option<String>, _>("assigned_to"),
            "completed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at"),
        }))),
        None => Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Task not found" })))),
    }
}

async fn update_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;
    let status = payload["status"].as_str().unwrap_or("pending");
    let completed_at = if status == "completed" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    sqlx::query(
        "UPDATE compliance_tasks SET status = $1, completed_at = $2 WHERE id = $3"
    )
    .bind(status)
    .bind(completed_at)
    .bind(task_id)
    .execute(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(json!({ "id": task_id, "status": status })))
}

async fn dashboard_summary(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;

    let total_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_tasks")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let pending_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_tasks WHERE status = 'pending'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let overdue_tasks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_tasks WHERE status = 'pending' AND due_date < CURRENT_DATE"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let completed_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_tasks WHERE status = 'completed'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let total_changes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM regulatory_changes")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let total_sources: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM regulatory_sources WHERE is_active = true")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let last_run: Option<Value> = sqlx::query(
        "SELECT id, started_at, completed_at, status, sources_checked, changes_found, tasks_created
         FROM agent_runs ORDER BY started_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(|row| {
        json!({
            "id": row.get::<Uuid, _>("id"),
            "started_at": row.get::<chrono::DateTime<chrono::Utc>, _>("started_at"),
            "completed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at"),
            "status": row.get::<String, _>("status"),
            "sources_checked": row.get::<i32, _>("sources_checked"),
            "changes_found": row.get::<i32, _>("changes_found"),
            "tasks_created": row.get::<i32, _>("tasks_created"),
        })
    });

    Ok(Json(json!({
        "tasks": {
            "total": total_tasks,
            "pending": pending_tasks,
            "overdue": overdue_tasks,
            "completed": completed_tasks,
        },
        "changes": {
            "total": total_changes,
        },
        "sources": {
            "active": total_sources,
        },
        "last_run": last_run,
    })))
}

async fn compliance_score(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = &state.db_pool;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_tasks")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let completed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_tasks WHERE status = 'completed'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let overdue: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compliance_tasks WHERE status = 'pending' AND due_date < CURRENT_DATE"
    )
    .fetch_one(pool)
    .await
        .unwrap_or(0);

    let score = if total > 0 {
        ((completed as f64 / total as f64) * 100.0).round() as i64
    } else {
        100 // No tasks = fully compliant
    };

    Ok(Json(json!({
        "score": score,
        "total_tasks": total,
        "completed": completed,
        "overdue": overdue,
    })))
}