use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, SqlitePool};
use uuid::Uuid;

use super::DbPool;
use crate::domain::{AttackEvent, Mitigation, MitigationRow, MitigationStatus};
use crate::error::Result;
use crate::observability::{ActorType, AuditEntry};

#[derive(Debug, FromRow)]
struct AuditRow {
    audit_id: Uuid,
    timestamp: DateTime<Utc>,
    schema_version: i32,
    actor_type: String,
    actor_id: Option<String>,
    action: String,
    target_type: Option<String>,
    target_id: Option<String>,
    details_json: String,
}

impl AuditEntry {
    fn from_row(row: AuditRow) -> Self {
        let actor_type = match row.actor_type.as_str() {
            "operator" => ActorType::Operator,
            "detector" => ActorType::Detector,
            _ => ActorType::System,
        };
        Self {
            audit_id: row.audit_id,
            timestamp: row.timestamp,
            schema_version: row.schema_version as u32,
            actor_type,
            actor_id: row.actor_id,
            action: row.action,
            target_type: row.target_type,
            target_id: row.target_id,
            details: serde_json::from_str(&row.details_json).unwrap_or(serde_json::json!({})),
        }
    }
}

#[derive(Clone)]
pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn from_sqlite(pool: SqlitePool) -> Self {
        Self { pool: DbPool::Sqlite(pool) }
    }

    pub fn from_postgres(pool: PgPool) -> Self {
        Self { pool: DbPool::Postgres(pool) }
    }

    // Events

    pub async fn insert_event(&self, event: &AttackEvent) -> Result<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => insert_event_sqlite(pool, event).await,
            DbPool::Postgres(pool) => insert_event_postgres(pool, event).await,
        }
    }

    pub async fn find_event_by_external_id(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<AttackEvent>> {
        match &self.pool {
            DbPool::Sqlite(pool) => find_event_by_external_id_sqlite(pool, source, external_id).await,
            DbPool::Postgres(pool) => find_event_by_external_id_postgres(pool, source, external_id).await,
        }
    }

    pub async fn list_events(&self, limit: u32, offset: u32) -> Result<Vec<AttackEvent>> {
        match &self.pool {
            DbPool::Sqlite(pool) => list_events_sqlite(pool, limit, offset).await,
            DbPool::Postgres(pool) => list_events_postgres(pool, limit, offset).await,
        }
    }

    // Audit Log

    pub async fn insert_audit(&self, entry: &AuditEntry) -> Result<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => insert_audit_sqlite(pool, entry).await,
            DbPool::Postgres(pool) => insert_audit_postgres(pool, entry).await,
        }
    }

    pub async fn list_audit(&self, limit: u32, offset: u32) -> Result<Vec<AuditEntry>> {
        match &self.pool {
            DbPool::Sqlite(pool) => list_audit_sqlite(pool, limit, offset).await,
            DbPool::Postgres(pool) => list_audit_postgres(pool, limit, offset).await,
        }
    }

    // Mitigations

    pub async fn insert_mitigation(&self, m: &Mitigation) -> Result<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => insert_mitigation_sqlite(pool, m).await,
            DbPool::Postgres(pool) => insert_mitigation_postgres(pool, m).await,
        }
    }

    pub async fn update_mitigation(&self, m: &Mitigation) -> Result<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => update_mitigation_sqlite(pool, m).await,
            DbPool::Postgres(pool) => update_mitigation_postgres(pool, m).await,
        }
    }

    pub async fn get_mitigation(&self, id: Uuid) -> Result<Option<Mitigation>> {
        match &self.pool {
            DbPool::Sqlite(pool) => get_mitigation_sqlite(pool, id).await,
            DbPool::Postgres(pool) => get_mitigation_postgres(pool, id).await,
        }
    }

    pub async fn find_active_by_scope(&self, scope_hash: &str, pop: &str) -> Result<Option<Mitigation>> {
        match &self.pool {
            DbPool::Sqlite(pool) => find_active_by_scope_sqlite(pool, scope_hash, pop).await,
            DbPool::Postgres(pool) => find_active_by_scope_postgres(pool, scope_hash, pop).await,
        }
    }

    pub async fn find_active_by_victim(&self, victim_ip: &str) -> Result<Vec<Mitigation>> {
        match &self.pool {
            DbPool::Sqlite(pool) => find_active_by_victim_sqlite(pool, victim_ip).await,
            DbPool::Postgres(pool) => find_active_by_victim_postgres(pool, victim_ip).await,
        }
    }

    pub async fn list_mitigations(
        &self,
        status_filter: Option<&[MitigationStatus]>,
        customer_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Mitigation>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                list_mitigations_sqlite(pool, status_filter, customer_id, limit, offset).await
            }
            DbPool::Postgres(pool) => {
                list_mitigations_postgres(pool, status_filter, customer_id, limit, offset).await
            }
        }
    }

    pub async fn count_active_by_customer(&self, customer_id: &str) -> Result<u32> {
        match &self.pool {
            DbPool::Sqlite(pool) => count_active_by_customer_sqlite(pool, customer_id).await,
            DbPool::Postgres(pool) => count_active_by_customer_postgres(pool, customer_id).await,
        }
    }

    pub async fn count_active_by_pop(&self, pop: &str) -> Result<u32> {
        match &self.pool {
            DbPool::Sqlite(pool) => count_active_by_pop_sqlite(pool, pop).await,
            DbPool::Postgres(pool) => count_active_by_pop_postgres(pool, pop).await,
        }
    }

    pub async fn count_active_global(&self) -> Result<u32> {
        match &self.pool {
            DbPool::Sqlite(pool) => count_active_global_sqlite(pool).await,
            DbPool::Postgres(pool) => count_active_global_postgres(pool).await,
        }
    }

    pub async fn find_expired_mitigations(&self) -> Result<Vec<Mitigation>> {
        match &self.pool {
            DbPool::Sqlite(pool) => find_expired_mitigations_sqlite(pool).await,
            DbPool::Postgres(pool) => find_expired_mitigations_postgres(pool).await,
        }
    }

    // Safelist

    pub async fn insert_safelist(&self, prefix: &str, added_by: &str, reason: Option<&str>) -> Result<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => insert_safelist_sqlite(pool, prefix, added_by, reason).await,
            DbPool::Postgres(pool) => insert_safelist_postgres(pool, prefix, added_by, reason).await,
        }
    }

    pub async fn remove_safelist(&self, prefix: &str) -> Result<bool> {
        match &self.pool {
            DbPool::Sqlite(pool) => remove_safelist_sqlite(pool, prefix).await,
            DbPool::Postgres(pool) => remove_safelist_postgres(pool, prefix).await,
        }
    }

    pub async fn list_safelist(&self) -> Result<Vec<SafelistEntry>> {
        match &self.pool {
            DbPool::Sqlite(pool) => list_safelist_sqlite(pool).await,
            DbPool::Postgres(pool) => list_safelist_postgres(pool).await,
        }
    }

    pub async fn is_safelisted(&self, ip: &str) -> Result<bool> {
        use ipnet::Ipv4Net;
        use std::net::Ipv4Addr;
        use std::str::FromStr;

        let entries = self.list_safelist().await?;
        let ip_addr = match Ipv4Addr::from_str(ip) {
            Ok(addr) => addr,
            Err(_) => return Ok(false),
        };

        for entry in entries {
            if let Ok(prefix) = Ipv4Net::from_str(&entry.prefix) {
                if prefix.contains(&ip_addr) {
                    return Ok(true);
                }
            } else if entry.prefix == ip {
                return Ok(true);
            }
        }

        Ok(false)
    }

    // Multi-POP coordination

    /// List all distinct POPs that have mitigations
    pub async fn list_pops(&self) -> Result<Vec<PopInfo>> {
        match &self.pool {
            DbPool::Sqlite(pool) => list_pops_sqlite(pool).await,
            DbPool::Postgres(pool) => list_pops_postgres(pool).await,
        }
    }

    /// Get aggregate stats across all POPs
    pub async fn get_stats(&self) -> Result<GlobalStats> {
        match &self.pool {
            DbPool::Sqlite(pool) => get_stats_sqlite(pool).await,
            DbPool::Postgres(pool) => get_stats_postgres(pool).await,
        }
    }

    /// List mitigations across all POPs (no POP filter)
    pub async fn list_mitigations_all_pops(
        &self,
        status_filter: Option<&[MitigationStatus]>,
        customer_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Mitigation>> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                list_mitigations_all_pops_sqlite(pool, status_filter, customer_id, limit, offset).await
            }
            DbPool::Postgres(pool) => {
                list_mitigations_all_pops_postgres(pool, status_filter, customer_id, limit, offset).await
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct PopInfo {
    /// POP identifier
    pub pop: String,
    /// Number of active mitigations in this POP
    pub active_mitigations: u32,
    /// Total mitigations (all statuses) in this POP
    pub total_mitigations: u32,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct GlobalStats {
    /// Total active mitigations across all POPs
    pub total_active: u32,
    /// Total mitigations across all POPs
    pub total_mitigations: u32,
    /// Total events ingested
    pub total_events: u32,
    /// Per-POP breakdown
    pub pops: Vec<PopStats>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct PopStats {
    /// POP identifier
    pub pop: String,
    /// Active mitigations
    pub active: u32,
    /// Total mitigations
    pub total: u32,
}

// ============================================================================
// SQLite implementations
// ============================================================================

async fn insert_event_sqlite(pool: &SqlitePool, event: &AttackEvent) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO events (
            event_id, external_event_id, source, event_timestamp, ingested_at,
            victim_ip, vector, protocol, bps, pps, top_dst_ports_json, confidence
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(event.event_id)
    .bind(&event.external_event_id)
    .bind(&event.source)
    .bind(event.event_timestamp)
    .bind(event.ingested_at)
    .bind(&event.victim_ip)
    .bind(&event.vector)
    .bind(event.protocol)
    .bind(event.bps)
    .bind(event.pps)
    .bind(&event.top_dst_ports_json)
    .bind(event.confidence)
    .execute(pool)
    .await?;
    Ok(())
}

async fn find_event_by_external_id_sqlite(
    pool: &SqlitePool,
    source: &str,
    external_id: &str,
) -> Result<Option<AttackEvent>> {
    let event = sqlx::query_as::<_, AttackEvent>(
        r#"
        SELECT event_id, external_event_id, source, event_timestamp, ingested_at,
               victim_ip, vector, protocol, bps, pps, top_dst_ports_json, confidence
        FROM events WHERE source = $1 AND external_event_id = $2
        "#,
    )
    .bind(source)
    .bind(external_id)
    .fetch_optional(pool)
    .await?;
    Ok(event)
}

async fn list_events_sqlite(pool: &SqlitePool, limit: u32, offset: u32) -> Result<Vec<AttackEvent>> {
    let events = sqlx::query_as::<_, AttackEvent>(
        r#"
        SELECT event_id, external_event_id, source, event_timestamp, ingested_at,
               victim_ip, vector, protocol, bps, pps, top_dst_ports_json, confidence
        FROM events ORDER BY ingested_at DESC LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(events)
}

async fn insert_audit_sqlite(pool: &SqlitePool, entry: &AuditEntry) -> Result<()> {
    let details_json = serde_json::to_string(&entry.details)?;
    sqlx::query(
        r#"
        INSERT INTO audit_log (audit_id, timestamp, schema_version, actor_type, actor_id, action, target_type, target_id, details_json)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(entry.audit_id)
    .bind(entry.timestamp)
    .bind(entry.schema_version as i32)
    .bind(format!("{:?}", entry.actor_type).to_lowercase())
    .bind(&entry.actor_id)
    .bind(&entry.action)
    .bind(&entry.target_type)
    .bind(&entry.target_id)
    .bind(&details_json)
    .execute(pool)
    .await?;
    Ok(())
}

async fn list_audit_sqlite(pool: &SqlitePool, limit: u32, offset: u32) -> Result<Vec<AuditEntry>> {
    let rows = sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT audit_id, timestamp, schema_version, actor_type, actor_id, action, target_type, target_id, details_json
        FROM audit_log ORDER BY timestamp DESC LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(AuditEntry::from_row).collect())
}

async fn insert_mitigation_sqlite(pool: &SqlitePool, m: &Mitigation) -> Result<()> {
    let match_json = serde_json::to_string(&m.match_criteria)?;
    let action_params_json = serde_json::to_string(&m.action_params)?;

    sqlx::query(
        r#"
        INSERT INTO mitigations (
            mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
            match_json, action_type, action_params_json, status,
            created_at, updated_at, expires_at, withdrawn_at,
            triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        "#,
    )
    .bind(m.mitigation_id)
    .bind(&m.scope_hash)
    .bind(&m.pop)
    .bind(&m.customer_id)
    .bind(&m.service_id)
    .bind(&m.victim_ip)
    .bind(m.vector.as_str())
    .bind(&match_json)
    .bind(m.action_type.as_str())
    .bind(&action_params_json)
    .bind(m.status.as_str())
    .bind(m.created_at)
    .bind(m.updated_at)
    .bind(m.expires_at)
    .bind(m.withdrawn_at)
    .bind(m.triggering_event_id)
    .bind(m.last_event_id)
    .bind(m.escalated_from_id)
    .bind(&m.reason)
    .bind(&m.rejection_reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_mitigation_sqlite(pool: &SqlitePool, m: &Mitigation) -> Result<()> {
    let match_json = serde_json::to_string(&m.match_criteria)?;
    let action_params_json = serde_json::to_string(&m.action_params)?;

    sqlx::query(
        r#"
        UPDATE mitigations SET
            scope_hash = $2, status = $3, updated_at = $4, expires_at = $5,
            withdrawn_at = $6, last_event_id = $7, match_json = $8,
            action_type = $9, action_params_json = $10, reason = $11, rejection_reason = $12
        WHERE mitigation_id = $1
        "#,
    )
    .bind(m.mitigation_id)
    .bind(&m.scope_hash)
    .bind(m.status.as_str())
    .bind(m.updated_at)
    .bind(m.expires_at)
    .bind(m.withdrawn_at)
    .bind(m.last_event_id)
    .bind(&match_json)
    .bind(m.action_type.as_str())
    .bind(&action_params_json)
    .bind(&m.reason)
    .bind(&m.rejection_reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_mitigation_sqlite(pool: &SqlitePool, id: Uuid) -> Result<Option<Mitigation>> {
    let row = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations WHERE mitigation_id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Mitigation::from_row))
}

async fn find_active_by_scope_sqlite(
    pool: &SqlitePool,
    scope_hash: &str,
    pop: &str,
) -> Result<Option<Mitigation>> {
    let row = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations
        WHERE scope_hash = $1 AND pop = $2 AND status IN ('pending', 'active', 'escalated')
        "#,
    )
    .bind(scope_hash)
    .bind(pop)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Mitigation::from_row))
}

async fn find_active_by_victim_sqlite(pool: &SqlitePool, victim_ip: &str) -> Result<Vec<Mitigation>> {
    let rows = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations
        WHERE victim_ip = $1 AND status IN ('pending', 'active', 'escalated')
        "#,
    )
    .bind(victim_ip)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

async fn list_mitigations_sqlite(
    pool: &SqlitePool,
    status_filter: Option<&[MitigationStatus]>,
    customer_id: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Mitigation>> {
    let mut query = String::from(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations WHERE 1=1
        "#,
    );

    if let Some(statuses) = status_filter {
        let placeholders: Vec<_> = statuses.iter().map(|s| format!("'{}'", s.as_str())).collect();
        query.push_str(&format!(" AND status IN ({})", placeholders.join(",")));
    }

    if let Some(cid) = customer_id {
        query.push_str(&format!(" AND customer_id = '{}'", cid));
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

    let rows = sqlx::query_as::<_, MitigationRow>(&query)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

async fn count_active_by_customer_sqlite(pool: &SqlitePool, customer_id: &str) -> Result<u32> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mitigations WHERE customer_id = $1 AND status IN ('pending', 'active', 'escalated')",
    )
    .bind(customer_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0 as u32)
}

async fn count_active_by_pop_sqlite(pool: &SqlitePool, pop: &str) -> Result<u32> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mitigations WHERE pop = $1 AND status IN ('pending', 'active', 'escalated')",
    )
    .bind(pop)
    .fetch_one(pool)
    .await?;
    Ok(row.0 as u32)
}

async fn count_active_global_sqlite(pool: &SqlitePool) -> Result<u32> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mitigations WHERE status IN ('pending', 'active', 'escalated')",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0 as u32)
}

async fn find_expired_mitigations_sqlite(pool: &SqlitePool) -> Result<Vec<Mitigation>> {
    let now = Utc::now();
    let rows = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations
        WHERE status IN ('active', 'escalated') AND expires_at < $1
        "#,
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

async fn insert_safelist_sqlite(
    pool: &SqlitePool,
    prefix: &str,
    added_by: &str,
    reason: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO safelist (prefix, added_at, added_by, reason) VALUES ($1, $2, $3, $4)",
    )
    .bind(prefix)
    .bind(Utc::now())
    .bind(added_by)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn remove_safelist_sqlite(pool: &SqlitePool, prefix: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM safelist WHERE prefix = $1")
        .bind(prefix)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

async fn list_safelist_sqlite(pool: &SqlitePool) -> Result<Vec<SafelistEntry>> {
    let rows = sqlx::query_as::<_, SafelistEntry>(
        "SELECT prefix, added_at, added_by, reason, expires_at FROM safelist",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn list_pops_sqlite(pool: &SqlitePool) -> Result<Vec<PopInfo>> {
    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT pop,
               SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active,
               COUNT(*) as total
        FROM mitigations
        GROUP BY pop
        ORDER BY pop
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(pop, active, total)| PopInfo {
            pop,
            active_mitigations: active as u32,
            total_mitigations: total as u32,
        })
        .collect())
}

async fn get_stats_sqlite(pool: &SqlitePool) -> Result<GlobalStats> {
    let (total_active, total_mitigations): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END),
            COUNT(*)
        FROM mitigations
        "#,
    )
    .fetch_one(pool)
    .await?;

    let total_events: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(pool)
        .await?;

    let pop_rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT pop,
               SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active,
               COUNT(*) as total
        FROM mitigations
        GROUP BY pop
        "#,
    )
    .fetch_all(pool)
    .await?;

    let pops = pop_rows
        .into_iter()
        .map(|(pop, active, total)| PopStats {
            pop,
            active: active as u32,
            total: total as u32,
        })
        .collect();

    Ok(GlobalStats {
        total_active: total_active as u32,
        total_mitigations: total_mitigations as u32,
        total_events: total_events.0 as u32,
        pops,
    })
}

async fn list_mitigations_all_pops_sqlite(
    pool: &SqlitePool,
    status_filter: Option<&[MitigationStatus]>,
    customer_id: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Mitigation>> {
    let mut query = String::from(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations WHERE 1=1
        "#,
    );

    if let Some(statuses) = status_filter {
        let placeholders: Vec<_> = statuses.iter().map(|s| format!("'{}'", s.as_str())).collect();
        query.push_str(&format!(" AND status IN ({})", placeholders.join(",")));
    }

    if let Some(cid) = customer_id {
        query.push_str(&format!(" AND customer_id = '{}'", cid));
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

    let rows = sqlx::query_as::<_, MitigationRow>(&query)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

// ============================================================================
// PostgreSQL implementations
// ============================================================================

async fn insert_event_postgres(pool: &PgPool, event: &AttackEvent) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO events (
            event_id, external_event_id, source, event_timestamp, ingested_at,
            victim_ip, vector, protocol, bps, pps, top_dst_ports_json, confidence
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(event.event_id)
    .bind(&event.external_event_id)
    .bind(&event.source)
    .bind(event.event_timestamp)
    .bind(event.ingested_at)
    .bind(&event.victim_ip)
    .bind(&event.vector)
    .bind(event.protocol.map(|p| p as i32))
    .bind(event.bps.map(|b| b as i64))
    .bind(event.pps.map(|p| p as i64))
    .bind(&event.top_dst_ports_json)
    .bind(event.confidence)
    .execute(pool)
    .await?;
    Ok(())
}

async fn find_event_by_external_id_postgres(
    pool: &PgPool,
    source: &str,
    external_id: &str,
) -> Result<Option<AttackEvent>> {
    let event = sqlx::query_as::<_, AttackEvent>(
        r#"
        SELECT event_id, external_event_id, source, event_timestamp, ingested_at,
               victim_ip, vector, protocol, bps, pps, top_dst_ports_json, confidence
        FROM events WHERE source = $1 AND external_event_id = $2
        "#,
    )
    .bind(source)
    .bind(external_id)
    .fetch_optional(pool)
    .await?;
    Ok(event)
}

async fn list_events_postgres(pool: &PgPool, limit: u32, offset: u32) -> Result<Vec<AttackEvent>> {
    let events = sqlx::query_as::<_, AttackEvent>(
        r#"
        SELECT event_id, external_event_id, source, event_timestamp, ingested_at,
               victim_ip, vector, protocol, bps, pps, top_dst_ports_json, confidence
        FROM events ORDER BY ingested_at DESC LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;
    Ok(events)
}

async fn insert_audit_postgres(pool: &PgPool, entry: &AuditEntry) -> Result<()> {
    let details_json = serde_json::to_string(&entry.details)?;
    sqlx::query(
        r#"
        INSERT INTO audit_log (audit_id, timestamp, schema_version, actor_type, actor_id, action, target_type, target_id, details_json)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(entry.audit_id)
    .bind(entry.timestamp)
    .bind(entry.schema_version as i32)
    .bind(format!("{:?}", entry.actor_type).to_lowercase())
    .bind(&entry.actor_id)
    .bind(&entry.action)
    .bind(&entry.target_type)
    .bind(&entry.target_id)
    .bind(&details_json)
    .execute(pool)
    .await?;
    Ok(())
}

async fn list_audit_postgres(pool: &PgPool, limit: u32, offset: u32) -> Result<Vec<AuditEntry>> {
    let rows = sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT audit_id, timestamp, schema_version, actor_type, actor_id, action, target_type, target_id, details_json
        FROM audit_log ORDER BY timestamp DESC LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(AuditEntry::from_row).collect())
}

async fn insert_mitigation_postgres(pool: &PgPool, m: &Mitigation) -> Result<()> {
    let match_json = serde_json::to_string(&m.match_criteria)?;
    let action_params_json = serde_json::to_string(&m.action_params)?;

    sqlx::query(
        r#"
        INSERT INTO mitigations (
            mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
            match_json, action_type, action_params_json, status,
            created_at, updated_at, expires_at, withdrawn_at,
            triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        "#,
    )
    .bind(m.mitigation_id)
    .bind(&m.scope_hash)
    .bind(&m.pop)
    .bind(&m.customer_id)
    .bind(&m.service_id)
    .bind(&m.victim_ip)
    .bind(m.vector.as_str())
    .bind(&match_json)
    .bind(m.action_type.as_str())
    .bind(&action_params_json)
    .bind(m.status.as_str())
    .bind(m.created_at)
    .bind(m.updated_at)
    .bind(m.expires_at)
    .bind(m.withdrawn_at)
    .bind(m.triggering_event_id)
    .bind(m.last_event_id)
    .bind(m.escalated_from_id)
    .bind(&m.reason)
    .bind(&m.rejection_reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_mitigation_postgres(pool: &PgPool, m: &Mitigation) -> Result<()> {
    let match_json = serde_json::to_string(&m.match_criteria)?;
    let action_params_json = serde_json::to_string(&m.action_params)?;

    sqlx::query(
        r#"
        UPDATE mitigations SET
            scope_hash = $2, status = $3, updated_at = $4, expires_at = $5,
            withdrawn_at = $6, last_event_id = $7, match_json = $8,
            action_type = $9, action_params_json = $10, reason = $11, rejection_reason = $12
        WHERE mitigation_id = $1
        "#,
    )
    .bind(m.mitigation_id)
    .bind(&m.scope_hash)
    .bind(m.status.as_str())
    .bind(m.updated_at)
    .bind(m.expires_at)
    .bind(m.withdrawn_at)
    .bind(m.last_event_id)
    .bind(&match_json)
    .bind(m.action_type.as_str())
    .bind(&action_params_json)
    .bind(&m.reason)
    .bind(&m.rejection_reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_mitigation_postgres(pool: &PgPool, id: Uuid) -> Result<Option<Mitigation>> {
    let row = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations WHERE mitigation_id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Mitigation::from_row))
}

async fn find_active_by_scope_postgres(
    pool: &PgPool,
    scope_hash: &str,
    pop: &str,
) -> Result<Option<Mitigation>> {
    let row = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations
        WHERE scope_hash = $1 AND pop = $2 AND status IN ('pending', 'active', 'escalated')
        "#,
    )
    .bind(scope_hash)
    .bind(pop)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Mitigation::from_row))
}

async fn find_active_by_victim_postgres(pool: &PgPool, victim_ip: &str) -> Result<Vec<Mitigation>> {
    let rows = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations
        WHERE victim_ip = $1 AND status IN ('pending', 'active', 'escalated')
        "#,
    )
    .bind(victim_ip)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

async fn list_mitigations_postgres(
    pool: &PgPool,
    status_filter: Option<&[MitigationStatus]>,
    customer_id: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Mitigation>> {
    let mut query = String::from(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations WHERE 1=1
        "#,
    );

    if let Some(statuses) = status_filter {
        let placeholders: Vec<_> = statuses.iter().map(|s| format!("'{}'", s.as_str())).collect();
        query.push_str(&format!(" AND status IN ({})", placeholders.join(",")));
    }

    if let Some(cid) = customer_id {
        query.push_str(&format!(" AND customer_id = '{}'", cid));
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

    let rows = sqlx::query_as::<_, MitigationRow>(&query)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

async fn count_active_by_customer_postgres(pool: &PgPool, customer_id: &str) -> Result<u32> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mitigations WHERE customer_id = $1 AND status IN ('pending', 'active', 'escalated')",
    )
    .bind(customer_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0 as u32)
}

async fn count_active_by_pop_postgres(pool: &PgPool, pop: &str) -> Result<u32> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mitigations WHERE pop = $1 AND status IN ('pending', 'active', 'escalated')",
    )
    .bind(pop)
    .fetch_one(pool)
    .await?;
    Ok(row.0 as u32)
}

async fn count_active_global_postgres(pool: &PgPool) -> Result<u32> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mitigations WHERE status IN ('pending', 'active', 'escalated')",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0 as u32)
}

async fn find_expired_mitigations_postgres(pool: &PgPool) -> Result<Vec<Mitigation>> {
    let now = Utc::now();
    let rows = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, pop, customer_id, service_id, victim_ip, vector,
               match_json, action_type, action_params_json, status,
               created_at, updated_at, expires_at, withdrawn_at,
               triggering_event_id, last_event_id, escalated_from_id, reason, rejection_reason
        FROM mitigations
        WHERE status IN ('active', 'escalated') AND expires_at < $1
        "#,
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

async fn insert_safelist_postgres(
    pool: &PgPool,
    prefix: &str,
    added_by: &str,
    reason: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO safelist (prefix, added_at, added_by, reason)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (prefix) DO UPDATE SET added_at = $2, added_by = $3, reason = $4
        "#,
    )
    .bind(prefix)
    .bind(Utc::now())
    .bind(added_by)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn remove_safelist_postgres(pool: &PgPool, prefix: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM safelist WHERE prefix = $1")
        .bind(prefix)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

async fn list_safelist_postgres(pool: &PgPool) -> Result<Vec<SafelistEntry>> {
    let rows = sqlx::query_as::<_, SafelistEntry>(
        "SELECT prefix, added_at, added_by, reason, expires_at FROM safelist",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn list_pops_postgres(pool: &PgPool) -> Result<Vec<PopInfo>> {
    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT pop,
               SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END)::bigint as active,
               COUNT(*)::bigint as total
        FROM mitigations
        GROUP BY pop
        ORDER BY pop
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(pop, active, total)| PopInfo {
            pop,
            active_mitigations: active as u32,
            total_mitigations: total as u32,
        })
        .collect())
}

async fn get_stats_postgres(pool: &PgPool) -> Result<GlobalStats> {
    let (total_active, total_mitigations): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END), 0)::bigint,
            COUNT(*)::bigint
        FROM mitigations
        "#,
    )
    .fetch_one(pool)
    .await?;

    let total_events: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM events")
        .fetch_one(pool)
        .await?;

    let pop_rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT pop,
               SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END)::bigint as active,
               COUNT(*)::bigint as total
        FROM mitigations
        GROUP BY pop
        "#,
    )
    .fetch_all(pool)
    .await?;

    let pops = pop_rows
        .into_iter()
        .map(|(pop, active, total)| PopStats {
            pop,
            active: active as u32,
            total: total as u32,
        })
        .collect();

    Ok(GlobalStats {
        total_active: total_active as u32,
        total_mitigations: total_mitigations as u32,
        total_events: total_events.0 as u32,
        pops,
    })
}

async fn list_mitigations_all_pops_postgres(
    pool: &PgPool,
    status_filter: Option<&[MitigationStatus]>,
    customer_id: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Mitigation>> {
    let status_strings: Option<Vec<String>> =
        status_filter.map(|s| s.iter().map(|st| st.as_str().to_string()).collect());

    let rows = sqlx::query_as::<_, MitigationRow>(
        r#"
        SELECT mitigation_id, scope_hash, customer_id, service_id, victim_ip,
               status, action, dst_prefix, protocol, dst_ports_json,
               announced_at, expires_at, withdrawn_at, withdraw_reason, pop, escalation_level
        FROM mitigations
        WHERE ($1::text[] IS NULL OR status = ANY($1))
          AND ($2::text IS NULL OR customer_id = $2)
        ORDER BY announced_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&status_strings)
    .bind(customer_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Mitigation::from_row).collect())
}

// ============================================================================
// Types
// ============================================================================

use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct SafelistEntry {
    /// IP prefix in CIDR notation
    pub prefix: String,
    /// When the entry was added
    pub added_at: chrono::DateTime<Utc>,
    /// Who added the entry
    pub added_by: String,
    /// Reason for safelisting
    pub reason: Option<String>,
    /// Optional expiration time
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_memory_pool;
    use crate::domain::{ActionParams, ActionType, AttackVector, MatchCriteria};

    async fn setup_repo() -> Repository {
        let pool = init_memory_pool().await.unwrap();
        Repository::from_sqlite(pool)
    }

    fn make_mitigation(victim_ip: &str, pop: &str, customer_id: Option<&str>) -> Mitigation {
        let event_id = Uuid::new_v4();
        Mitigation {
            mitigation_id: Uuid::new_v4(),
            scope_hash: format!("hash_{}", victim_ip),
            pop: pop.to_string(),
            customer_id: customer_id.map(String::from),
            service_id: None,
            victim_ip: victim_ip.to_string(),
            vector: AttackVector::UdpFlood,
            match_criteria: MatchCriteria {
                dst_prefix: format!("{}/32", victim_ip),
                protocol: Some(17),
                dst_ports: vec![53],
            },
            action_type: ActionType::Discard,
            action_params: ActionParams { rate_bps: None },
            status: MitigationStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            withdrawn_at: None,
            triggering_event_id: event_id,
            last_event_id: event_id,
            escalated_from_id: None,
            reason: "test mitigation".to_string(),
            rejection_reason: None,
        }
    }

    fn make_event(victim_ip: &str) -> AttackEvent {
        AttackEvent {
            event_id: Uuid::new_v4(),
            external_event_id: Some(format!("ext_{}", victim_ip)),
            source: "test_detector".to_string(),
            event_timestamp: Utc::now(),
            ingested_at: Utc::now(),
            victim_ip: victim_ip.to_string(),
            vector: "udp_flood".to_string(),
            protocol: Some(17),
            bps: Some(1_000_000_000),
            pps: Some(100_000),
            top_dst_ports_json: "[53]".to_string(),
            confidence: Some(0.95),
        }
    }

    // ==========================================================================
    // Event Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_insert_and_find_event() {
        let repo = setup_repo().await;
        let event = make_event("192.168.1.1");

        repo.insert_event(&event).await.unwrap();

        let found = repo
            .find_event_by_external_id(&event.source, event.external_event_id.as_ref().unwrap())
            .await
            .unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.event_id, event.event_id);
        assert_eq!(found.victim_ip, "192.168.1.1");
    }

    #[tokio::test]
    async fn test_find_event_not_found() {
        let repo = setup_repo().await;

        let found = repo
            .find_event_by_external_id("unknown", "unknown_id")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    // ==========================================================================
    // Mitigation CRUD Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_insert_and_get_mitigation() {
        let repo = setup_repo().await;
        let mitigation = make_mitigation("10.0.0.1", "pop1", Some("cust_1"));

        repo.insert_mitigation(&mitigation).await.unwrap();

        let found = repo.get_mitigation(mitigation.mitigation_id).await.unwrap();
        assert!(found.is_some());

        let found = found.unwrap();
        assert_eq!(found.victim_ip, "10.0.0.1");
        assert_eq!(found.pop, "pop1");
        assert_eq!(found.customer_id, Some("cust_1".to_string()));
    }

    #[tokio::test]
    async fn test_get_mitigation_not_found() {
        let repo = setup_repo().await;

        let found = repo.get_mitigation(Uuid::new_v4()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_mitigation() {
        let repo = setup_repo().await;
        let mut mitigation = make_mitigation("10.0.0.2", "pop1", None);

        repo.insert_mitigation(&mitigation).await.unwrap();

        // Update status
        mitigation.status = MitigationStatus::Withdrawn;
        mitigation.withdrawn_at = Some(Utc::now());
        mitigation.reason = "test withdrawal".to_string();

        repo.update_mitigation(&mitigation).await.unwrap();

        let found = repo.get_mitigation(mitigation.mitigation_id).await.unwrap().unwrap();
        assert_eq!(found.status, MitigationStatus::Withdrawn);
        assert!(found.withdrawn_at.is_some());
    }

    // ==========================================================================
    // Mitigation Query Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_find_active_by_scope() {
        let repo = setup_repo().await;
        let mitigation = make_mitigation("10.0.0.3", "pop1", None);

        repo.insert_mitigation(&mitigation).await.unwrap();

        let found = repo
            .find_active_by_scope(&mitigation.scope_hash, "pop1")
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().mitigation_id, mitigation.mitigation_id);
    }

    #[tokio::test]
    async fn test_find_active_by_scope_wrong_pop() {
        let repo = setup_repo().await;
        let mitigation = make_mitigation("10.0.0.4", "pop1", None);

        repo.insert_mitigation(&mitigation).await.unwrap();

        // Different POP should not find it
        let found = repo
            .find_active_by_scope(&mitigation.scope_hash, "pop2")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_active_by_victim() {
        let repo = setup_repo().await;

        // Insert multiple mitigations for same victim
        let m1 = make_mitigation("10.0.0.5", "pop1", None);
        let m2 = make_mitigation("10.0.0.5", "pop2", None);

        repo.insert_mitigation(&m1).await.unwrap();
        repo.insert_mitigation(&m2).await.unwrap();

        let found = repo.find_active_by_victim("10.0.0.5").await.unwrap();
        assert_eq!(found.len(), 2);
    }

    #[tokio::test]
    async fn test_list_mitigations() {
        let repo = setup_repo().await;

        repo.insert_mitigation(&make_mitigation("10.0.1.1", "pop1", Some("cust_a"))).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.1.2", "pop1", Some("cust_b"))).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.1.3", "pop1", Some("cust_a"))).await.unwrap();

        // List all
        let all = repo.list_mitigations(None, None, 100, 0).await.unwrap();
        assert_eq!(all.len(), 3);

        // Filter by customer
        let cust_a = repo.list_mitigations(None, Some("cust_a"), 100, 0).await.unwrap();
        assert_eq!(cust_a.len(), 2);

        // Filter by status
        let active = repo
            .list_mitigations(Some(&[MitigationStatus::Active]), None, 100, 0)
            .await
            .unwrap();
        assert_eq!(active.len(), 3);
    }

    #[tokio::test]
    async fn test_list_mitigations_pagination() {
        let repo = setup_repo().await;

        for i in 0..10 {
            repo.insert_mitigation(&make_mitigation(&format!("10.0.2.{}", i), "pop1", None))
                .await
                .unwrap();
        }

        let page1 = repo.list_mitigations(None, None, 3, 0).await.unwrap();
        assert_eq!(page1.len(), 3);

        let page2 = repo.list_mitigations(None, None, 3, 3).await.unwrap();
        assert_eq!(page2.len(), 3);

        let page4 = repo.list_mitigations(None, None, 3, 9).await.unwrap();
        assert_eq!(page4.len(), 1);
    }

    // ==========================================================================
    // Counting Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_count_active_by_customer() {
        let repo = setup_repo().await;

        repo.insert_mitigation(&make_mitigation("10.0.3.1", "pop1", Some("cust_x"))).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.3.2", "pop1", Some("cust_x"))).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.3.3", "pop1", Some("cust_y"))).await.unwrap();

        let count_x = repo.count_active_by_customer("cust_x").await.unwrap();
        assert_eq!(count_x, 2);

        let count_y = repo.count_active_by_customer("cust_y").await.unwrap();
        assert_eq!(count_y, 1);

        let count_z = repo.count_active_by_customer("cust_z").await.unwrap();
        assert_eq!(count_z, 0);
    }

    #[tokio::test]
    async fn test_count_active_by_pop() {
        let repo = setup_repo().await;

        repo.insert_mitigation(&make_mitigation("10.0.4.1", "nyc1", None)).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.4.2", "nyc1", None)).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.4.3", "lax1", None)).await.unwrap();

        let nyc = repo.count_active_by_pop("nyc1").await.unwrap();
        assert_eq!(nyc, 2);

        let lax = repo.count_active_by_pop("lax1").await.unwrap();
        assert_eq!(lax, 1);
    }

    #[tokio::test]
    async fn test_count_active_global() {
        let repo = setup_repo().await;

        assert_eq!(repo.count_active_global().await.unwrap(), 0);

        repo.insert_mitigation(&make_mitigation("10.0.5.1", "pop1", None)).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.5.2", "pop2", None)).await.unwrap();

        assert_eq!(repo.count_active_global().await.unwrap(), 2);
    }

    // ==========================================================================
    // Safelist Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_safelist_crud() {
        let repo = setup_repo().await;

        // Add
        repo.insert_safelist("10.0.0.0/8", "admin", Some("internal network"))
            .await
            .unwrap();

        // List
        let list = repo.list_safelist().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].prefix, "10.0.0.0/8");
        assert_eq!(list[0].added_by, "admin");

        // Remove
        let removed = repo.remove_safelist("10.0.0.0/8").await.unwrap();
        assert!(removed);

        let list = repo.list_safelist().await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_is_safelisted() {
        let repo = setup_repo().await;

        repo.insert_safelist("10.0.0.0/8", "admin", None).await.unwrap();
        repo.insert_safelist("192.168.1.100/32", "admin", None).await.unwrap();

        // In safelist range
        assert!(repo.is_safelisted("10.1.2.3").await.unwrap());
        assert!(repo.is_safelisted("10.255.255.255").await.unwrap());

        // Exact match
        assert!(repo.is_safelisted("192.168.1.100").await.unwrap());

        // Not safelisted
        assert!(!repo.is_safelisted("192.168.1.101").await.unwrap());
        assert!(!repo.is_safelisted("8.8.8.8").await.unwrap());
    }

    // ==========================================================================
    // Multi-POP Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_list_pops() {
        let repo = setup_repo().await;

        repo.insert_mitigation(&make_mitigation("10.0.6.1", "nyc1", None)).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.6.2", "nyc1", None)).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.6.3", "lax1", None)).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.6.4", "ams1", None)).await.unwrap();

        let pops = repo.list_pops().await.unwrap();
        assert_eq!(pops.len(), 3);

        // Check NYC has 2 active
        let nyc = pops.iter().find(|p| p.pop == "nyc1").unwrap();
        assert_eq!(nyc.active_mitigations, 2);
        assert_eq!(nyc.total_mitigations, 2);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let repo = setup_repo().await;

        // Insert some data
        repo.insert_event(&make_event("10.0.7.1")).await.unwrap();
        repo.insert_event(&make_event("10.0.7.2")).await.unwrap();

        repo.insert_mitigation(&make_mitigation("10.0.7.1", "pop1", None)).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.7.2", "pop2", None)).await.unwrap();

        let stats = repo.get_stats().await.unwrap();

        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.total_active, 2);
        assert_eq!(stats.total_mitigations, 2);
        assert_eq!(stats.pops.len(), 2);
    }

    #[tokio::test]
    async fn test_list_mitigations_all_pops() {
        let repo = setup_repo().await;

        repo.insert_mitigation(&make_mitigation("10.0.8.1", "pop1", Some("cust_1"))).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.8.2", "pop2", Some("cust_1"))).await.unwrap();
        repo.insert_mitigation(&make_mitigation("10.0.8.3", "pop3", Some("cust_2"))).await.unwrap();

        // All POPs, no filter
        let all = repo.list_mitigations_all_pops(None, None, 100, 0).await.unwrap();
        assert_eq!(all.len(), 3);

        // All POPs, filter by customer
        let cust1 = repo
            .list_mitigations_all_pops(None, Some("cust_1"), 100, 0)
            .await
            .unwrap();
        assert_eq!(cust1.len(), 2);
    }
}
