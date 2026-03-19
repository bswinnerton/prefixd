use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the multi-signal correlation engine.
///
/// When `enabled` is false (the default), the correlation engine is bypassed
/// and events follow the direct path to policy evaluation — identical to
/// pre-correlation behavior.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorrelationConfig {
    /// Whether the correlation engine is active.
    #[serde(default)]
    pub enabled: bool,

    /// Time window (in seconds) for grouping signals by (victim_ip, vector).
    /// Events arriving within this window are added to the same signal group.
    #[serde(default = "default_window_seconds")]
    pub window_seconds: u32,

    /// Global minimum number of distinct sources required before a signal group
    /// can trigger a mitigation. Set to 1 for backward-compatible single-source
    /// behavior.
    #[serde(default = "default_min_sources")]
    pub min_sources: u32,

    /// Global minimum derived confidence threshold. A signal group must reach
    /// this threshold (in addition to `min_sources`) before triggering.
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,

    /// Per-source configuration: weight and type for known detection sources.
    #[serde(default)]
    pub sources: HashMap<String, SourceConfig>,

    /// Default weight assigned to events from sources not listed in `sources`.
    #[serde(default = "default_weight")]
    pub default_weight: f32,
}

/// Configuration for a single detection/signal source.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SourceConfig {
    /// Weight applied to events from this source when computing derived
    /// confidence. Higher weight = more influence on the weighted average.
    #[serde(default = "default_weight")]
    pub weight: f32,

    /// Descriptive type of the source (e.g., "detector", "telemetry", "manual").
    #[serde(default)]
    pub r#type: String,

    /// Optional per-action confidence mapping. Keys are action types (e.g.,
    /// "ban", "partial_block", "alert") and values are confidence scores (0.0–1.0).
    /// Used by signal adapters (e.g., FastNetMon) to map action types to confidence.
    #[serde(default)]
    pub confidence_mapping: HashMap<String, f32>,
}

/// Per-playbook correlation override. When present on a playbook, these values
/// override the global `min_sources` and `confidence_threshold` for events
/// matching that playbook.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PlaybookCorrelationOverride {
    /// Override for the minimum number of distinct sources.
    #[serde(default)]
    pub min_sources: Option<u32>,

    /// Override for the minimum derived confidence threshold.
    #[serde(default)]
    pub confidence_threshold: Option<f32>,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            window_seconds: default_window_seconds(),
            min_sources: default_min_sources(),
            confidence_threshold: default_confidence_threshold(),
            sources: HashMap::new(),
            default_weight: default_weight(),
        }
    }
}

fn default_window_seconds() -> u32 {
    300
}

fn default_min_sources() -> u32 {
    1
}

fn default_confidence_threshold() -> f32 {
    0.5
}

fn default_weight() -> f32 {
    1.0
}

impl CorrelationConfig {
    /// Resolve the effective weight for a given source name.
    /// Returns the configured weight if the source is known, or `default_weight` otherwise.
    pub fn source_weight(&self, source: &str) -> f32 {
        self.sources
            .get(source)
            .map(|s| s.weight)
            .unwrap_or(self.default_weight)
    }

    /// Resolve effective min_sources, using a per-playbook override if provided.
    pub fn effective_min_sources(
        &self,
        playbook_override: Option<&PlaybookCorrelationOverride>,
    ) -> u32 {
        playbook_override
            .and_then(|o| o.min_sources)
            .unwrap_or(self.min_sources)
    }

    /// Resolve effective confidence_threshold, using a per-playbook override if provided.
    pub fn effective_confidence_threshold(
        &self,
        playbook_override: Option<&PlaybookCorrelationOverride>,
    ) -> f32 {
        playbook_override
            .and_then(|o| o.confidence_threshold)
            .unwrap_or(self.confidence_threshold)
    }

    /// Resolve confidence for a given source and action type using the per-source
    /// `confidence_mapping`. Falls back to `default_confidence_mapping` if no
    /// source-specific mapping is configured.
    pub fn source_action_confidence(&self, source: &str, action: &str) -> f32 {
        if let Some(source_config) = self.sources.get(source) {
            if let Some(&confidence) = source_config.confidence_mapping.get(action) {
                return confidence;
            }
        }
        // Default mapping: ban=0.9, partial_block=0.7, alert=0.5
        default_confidence_mapping(action)
    }
}

/// Default confidence mapping for FastNetMon action types.
fn default_confidence_mapping(action: &str) -> f32 {
    match action {
        "ban" => 0.9,
        "partial_block" => 0.7,
        "alert" => 0.5,
        _ => 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CorrelationConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.window_seconds, 300);
        assert_eq!(config.min_sources, 1);
        assert_eq!(config.confidence_threshold, 0.5);
        assert!(config.sources.is_empty());
        assert_eq!(config.default_weight, 1.0);
    }

    #[test]
    fn test_deserialize_empty_yaml() {
        // Missing correlation section should result in defaults
        let yaml = "";
        let config: CorrelationConfig = serde_yaml::from_str(yaml).unwrap_or_default();
        assert!(!config.enabled);
        assert_eq!(config.window_seconds, 300);
        assert_eq!(config.min_sources, 1);
    }

    #[test]
    fn test_deserialize_minimal_enabled() {
        let yaml = r#"
enabled: true
"#;
        let config: CorrelationConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.window_seconds, 300);
        assert_eq!(config.min_sources, 1);
        assert_eq!(config.confidence_threshold, 0.5);
        assert!(config.sources.is_empty());
        assert_eq!(config.default_weight, 1.0);
    }

    #[test]
    fn test_deserialize_full_config() {
        let yaml = r#"
enabled: true
window_seconds: 600
min_sources: 2
confidence_threshold: 0.7
default_weight: 0.5
sources:
  fastnetmon:
    weight: 2.0
    type: detector
  alertmanager:
    weight: 0.8
    type: telemetry
  dashboard:
    weight: 1.0
    type: manual
"#;
        let config: CorrelationConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.window_seconds, 600);
        assert_eq!(config.min_sources, 2);
        assert_eq!(config.confidence_threshold, 0.7);
        assert_eq!(config.default_weight, 0.5);
        assert_eq!(config.sources.len(), 3);
        assert_eq!(config.sources["fastnetmon"].weight, 2.0);
        assert_eq!(config.sources["fastnetmon"].r#type, "detector");
        assert_eq!(config.sources["alertmanager"].weight, 0.8);
        assert_eq!(config.sources["dashboard"].weight, 1.0);
    }

    #[test]
    fn test_source_weight_known() {
        let mut config = CorrelationConfig::default();
        config.sources.insert(
            "fastnetmon".to_string(),
            SourceConfig {
                weight: 2.0,
                r#type: "detector".to_string(),
                confidence_mapping: HashMap::new(),
            },
        );
        assert_eq!(config.source_weight("fastnetmon"), 2.0);
    }

    #[test]
    fn test_source_weight_unknown_uses_default() {
        let config = CorrelationConfig::default();
        assert_eq!(config.source_weight("unknown_detector"), 1.0);
    }

    #[test]
    fn test_source_weight_unknown_uses_custom_default() {
        let mut config = CorrelationConfig::default();
        config.default_weight = 0.5;
        assert_eq!(config.source_weight("unknown"), 0.5);
    }

    #[test]
    fn test_effective_min_sources_no_override() {
        let config = CorrelationConfig {
            min_sources: 2,
            ..Default::default()
        };
        assert_eq!(config.effective_min_sources(None), 2);
    }

    #[test]
    fn test_effective_min_sources_with_override() {
        let config = CorrelationConfig {
            min_sources: 2,
            ..Default::default()
        };
        let override_ = PlaybookCorrelationOverride {
            min_sources: Some(3),
            confidence_threshold: None,
        };
        assert_eq!(config.effective_min_sources(Some(&override_)), 3);
    }

    #[test]
    fn test_effective_min_sources_with_none_override() {
        let config = CorrelationConfig {
            min_sources: 2,
            ..Default::default()
        };
        let override_ = PlaybookCorrelationOverride {
            min_sources: None,
            confidence_threshold: None,
        };
        assert_eq!(config.effective_min_sources(Some(&override_)), 2);
    }

    #[test]
    fn test_effective_confidence_threshold_no_override() {
        let config = CorrelationConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };
        assert_eq!(config.effective_confidence_threshold(None), 0.7);
    }

    #[test]
    fn test_effective_confidence_threshold_with_override() {
        let config = CorrelationConfig {
            confidence_threshold: 0.5,
            ..Default::default()
        };
        let override_ = PlaybookCorrelationOverride {
            min_sources: None,
            confidence_threshold: Some(0.8),
        };
        assert_eq!(config.effective_confidence_threshold(Some(&override_)), 0.8);
    }

    #[test]
    fn test_playbook_correlation_override_deserialize() {
        let yaml = r#"
min_sources: 3
confidence_threshold: 0.9
"#;
        let override_: PlaybookCorrelationOverride = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(override_.min_sources, Some(3));
        assert_eq!(override_.confidence_threshold, Some(0.9));
    }

    #[test]
    fn test_playbook_correlation_override_partial() {
        let yaml = r#"
min_sources: 2
"#;
        let override_: PlaybookCorrelationOverride = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(override_.min_sources, Some(2));
        assert_eq!(override_.confidence_threshold, None);
    }

    #[test]
    fn test_playbook_correlation_override_empty() {
        let yaml = "{}";
        let override_: PlaybookCorrelationOverride = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(override_.min_sources, None);
        assert_eq!(override_.confidence_threshold, None);
    }

    #[test]
    fn test_settings_without_correlation_section() {
        // Simulates parsing a prefixd.yaml that has no correlation key.
        // The CorrelationConfig field uses #[serde(default)] so this must not fail.
        let yaml = r#"
pop: iad1
mode: dry-run
http:
  listen: "0.0.0.0:8080"
  auth:
    mode: none
bgp:
  mode: mock
  gobgp_grpc: "localhost:50051"
  local_asn: 65010
  router_id: "10.10.0.10"
guardrails:
  require_ttl: true
  dst_prefix_minlen: 32
  dst_prefix_maxlen: 32
  max_ports: 8
  allow_src_prefix_match: false
quotas:
  max_active_per_customer: 5
  max_active_per_pop: 200
  max_active_global: 500
  max_new_per_minute: 30
timers:
  default_ttl_seconds: 120
  min_ttl_seconds: 30
  max_ttl_seconds: 1800
  correlation_window_seconds: 300
  reconciliation_interval_seconds: 30
escalation:
  enabled: true
  min_persistence_seconds: 120
  min_confidence: 0.7
storage:
  connection_string: "postgres://user:pass@localhost/prefixd"
observability:
  log_format: pretty
  log_level: info
  audit_log_path: "./data/audit.jsonl"
  metrics_listen: "0.0.0.0:9090"
"#;
        // This will be tested via the Settings struct after we add the field
        let _config: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
    }

    #[test]
    fn test_settings_with_correlation_section() {
        // Simulates parsing a prefixd.yaml that includes a correlation section
        let yaml = r#"
enabled: true
window_seconds: 120
min_sources: 3
confidence_threshold: 0.8
sources:
  netflow:
    weight: 1.5
    type: telemetry
"#;
        let config: CorrelationConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.window_seconds, 120);
        assert_eq!(config.min_sources, 3);
        assert_eq!(config.confidence_threshold, 0.8);
        assert_eq!(config.sources.len(), 1);
        assert_eq!(config.sources["netflow"].weight, 1.5);
    }

    #[test]
    fn test_source_action_confidence_default_mapping() {
        let config = CorrelationConfig::default();
        assert_eq!(config.source_action_confidence("fastnetmon", "ban"), 0.9);
        assert_eq!(
            config.source_action_confidence("fastnetmon", "partial_block"),
            0.7
        );
        assert_eq!(config.source_action_confidence("fastnetmon", "alert"), 0.5);
        assert_eq!(
            config.source_action_confidence("fastnetmon", "unknown_action"),
            0.5
        );
    }

    #[test]
    fn test_source_action_confidence_override() {
        let mut config = CorrelationConfig::default();
        let mut mapping = HashMap::new();
        mapping.insert("ban".to_string(), 0.95);
        mapping.insert("alert".to_string(), 0.3);
        config.sources.insert(
            "fastnetmon".to_string(),
            SourceConfig {
                weight: 1.0,
                r#type: "detector".to_string(),
                confidence_mapping: mapping,
            },
        );
        // Overridden values
        assert_eq!(config.source_action_confidence("fastnetmon", "ban"), 0.95);
        assert_eq!(config.source_action_confidence("fastnetmon", "alert"), 0.3);
        // Not overridden — falls back to default
        assert_eq!(
            config.source_action_confidence("fastnetmon", "partial_block"),
            0.7
        );
    }

    #[test]
    fn test_source_action_confidence_unknown_source() {
        let config = CorrelationConfig::default();
        // Unknown source gets default mapping
        assert_eq!(
            config.source_action_confidence("unknown_source", "ban"),
            0.9
        );
    }

    #[test]
    fn test_confidence_mapping_deserialization() {
        let yaml = r#"
enabled: true
sources:
  fastnetmon:
    weight: 1.0
    type: detector
    confidence_mapping:
      ban: 0.95
      partial_block: 0.8
      alert: 0.3
"#;
        let config: CorrelationConfig = serde_yaml::from_str(yaml).unwrap();
        let fnm = &config.sources["fastnetmon"];
        assert_eq!(fnm.confidence_mapping["ban"], 0.95);
        assert_eq!(fnm.confidence_mapping["partial_block"], 0.8);
        assert_eq!(fnm.confidence_mapping["alert"], 0.3);
    }
}
