use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{AttackEvent, Mitigation, MitigationStatus};
use crate::error::Result;
use crate::observability::AuditEntry;

use super::{GlobalStats, PopInfo, SafelistEntry};

#[async_trait]
pub trait RepositoryTrait: Send + Sync {
    // Events
    async fn insert_event(&self, event: &AttackEvent) -> Result<()>;
    async fn find_event_by_external_id(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<AttackEvent>>;
    async fn list_events(&self, limit: u32, offset: u32) -> Result<Vec<AttackEvent>>;

    // Audit Log
    async fn insert_audit(&self, entry: &AuditEntry) -> Result<()>;
    async fn list_audit(&self, limit: u32, offset: u32) -> Result<Vec<AuditEntry>>;

    // Mitigations
    async fn insert_mitigation(&self, m: &Mitigation) -> Result<()>;
    async fn update_mitigation(&self, m: &Mitigation) -> Result<()>;
    async fn get_mitigation(&self, id: Uuid) -> Result<Option<Mitigation>>;
    async fn find_active_by_scope(&self, scope_hash: &str, pop: &str) -> Result<Option<Mitigation>>;
    async fn find_active_by_victim(&self, victim_ip: &str) -> Result<Vec<Mitigation>>;
    async fn list_mitigations(
        &self,
        status_filter: Option<&[MitigationStatus]>,
        customer_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Mitigation>>;
    async fn count_active_by_customer(&self, customer_id: &str) -> Result<u32>;
    async fn count_active_by_pop(&self, pop: &str) -> Result<u32>;
    async fn count_active_global(&self) -> Result<u32>;
    async fn find_expired_mitigations(&self) -> Result<Vec<Mitigation>>;

    // Safelist
    async fn insert_safelist(&self, prefix: &str, added_by: &str, reason: Option<&str>) -> Result<()>;
    async fn remove_safelist(&self, prefix: &str) -> Result<bool>;
    async fn list_safelist(&self) -> Result<Vec<SafelistEntry>>;
    async fn is_safelisted(&self, ip: &str) -> Result<bool>;

    // Multi-POP coordination
    async fn list_pops(&self) -> Result<Vec<PopInfo>>;
    async fn get_stats(&self) -> Result<GlobalStats>;
    async fn list_mitigations_all_pops(
        &self,
        status_filter: Option<&[MitigationStatus]>,
        customer_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Mitigation>>;
}
