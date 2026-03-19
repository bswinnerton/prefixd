use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

use prefixd::AppState;
use prefixd::api::create_test_router;
use prefixd::bgp::MockAnnouncer;
use prefixd::config::{
    AllowedPorts, Asset, AuthConfig, AuthMode, BgpConfig, BgpMode, Customer, EscalationConfig,
    GuardrailsConfig, HttpConfig, Inventory, ObservabilityConfig, Playbook, PlaybookAction,
    PlaybookMatch, PlaybookStep, Playbooks, QuotasConfig, RateLimitConfig, SafelistConfig, Service,
    Settings, ShutdownConfig, StorageConfig, TimersConfig,
};
use prefixd::db::{MockRepository, RepositoryTrait};
use prefixd::domain::AttackVector;

fn test_settings() -> Settings {
    Settings {
        pop: "test1".to_string(),
        mode: prefixd::config::OperationMode::DryRun,
        http: HttpConfig {
            listen: "127.0.0.1:0".to_string(),
            auth: AuthConfig {
                mode: AuthMode::None,
                bearer_token_env: None,
                ldap: None,
                radius: None,
            },
            rate_limit: RateLimitConfig::default(),
            tls: None,
            cors_origin: None,
        },
        bgp: BgpConfig {
            mode: BgpMode::Mock,
            gobgp_grpc: "127.0.0.1:50051".to_string(),
            local_asn: 65000,
            router_id: "10.0.0.1".to_string(),
            neighbors: vec![],
        },
        guardrails: GuardrailsConfig {
            require_ttl: true,
            min_ttl_seconds: Some(30),
            max_ttl_seconds: Some(1800),
            dst_prefix_minlen: 32,
            dst_prefix_maxlen: 32,
            dst_prefix_minlen_v6: None,
            dst_prefix_maxlen_v6: None,
            max_ports: 8,
            allow_src_prefix_match: false,
            allow_tcp_flags_match: false,
            allow_fragment_match: false,
            allow_packet_length_match: false,
        },
        quotas: QuotasConfig {
            max_active_per_customer: 5,
            max_active_per_pop: 200,
            max_active_global: 500,
            max_new_per_minute: 30,
            max_announcements_per_peer: 100,
        },
        timers: TimersConfig {
            default_ttl_seconds: 120,
            min_ttl_seconds: 30,
            max_ttl_seconds: 1800,
            correlation_window_seconds: 300,
            reconciliation_interval_seconds: 30,
            quiet_period_after_withdraw_seconds: 120,
        },
        escalation: EscalationConfig {
            enabled: true,
            min_persistence_seconds: 120,
            min_confidence: 0.7,
            max_escalated_duration_seconds: 1800,
        },
        storage: StorageConfig {
            connection_string: "postgres://unused:unused@localhost/unused".to_string(),
        },
        observability: ObservabilityConfig {
            log_format: prefixd::config::LogFormat::Pretty,
            log_level: "info".to_string(),
            audit_log_path: "/dev/null".to_string(),
            metrics_listen: "127.0.0.1:0".to_string(),
        },
        safelist: SafelistConfig { prefixes: vec![] },
        shutdown: ShutdownConfig::default(),
        alerting: Default::default(),
        correlation: Default::default(),
    }
}

fn test_inventory() -> Inventory {
    Inventory::new(vec![Customer {
        customer_id: "cust_test".to_string(),
        name: "Test Customer".to_string(),
        prefixes: vec!["203.0.113.0/24".to_string()],
        policy_profile: prefixd::config::PolicyProfile::Normal,
        services: vec![Service {
            service_id: "svc_dns".to_string(),
            name: "DNS".to_string(),
            assets: vec![Asset {
                ip: "203.0.113.10".to_string(),
                role: Some("dns".to_string()),
            }],
            allowed_ports: AllowedPorts {
                udp: vec![53],
                tcp: vec![53],
            },
        }],
    }])
}

fn test_playbooks() -> Playbooks {
    Playbooks {
        playbooks: vec![Playbook {
            name: "udp_flood_test".to_string(),
            match_criteria: PlaybookMatch {
                vector: AttackVector::UdpFlood,
                require_top_ports: false,
            },
            correlation: None,
            steps: vec![PlaybookStep {
                action: PlaybookAction::Police,
                rate_bps: Some(5_000_000),
                ttl_seconds: 120,
                require_confidence_at_least: None,
                require_persistence_seconds: None,
            }],
        }],
    }
}

async fn setup_app_with_config_dir(config_dir: std::path::PathBuf) -> axum::Router {
    let repo: Arc<dyn RepositoryTrait> = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());

    let state = AppState::new(
        test_settings(),
        test_inventory(),
        test_playbooks(),
        repo,
        announcer,
        config_dir,
    )
    .expect("failed to create app state");

    create_test_router(state)
}

async fn setup_app() -> axum::Router {
    setup_app_with_config_dir(std::path::PathBuf::from(".")).await
}

async fn setup_app_bearer_with_config_dir(config_dir: std::path::PathBuf) -> axum::Router {
    unsafe {
        std::env::set_var("TEST_PREFIXD_TOKEN", "test-secret-token-123");
    }

    let repo: Arc<dyn RepositoryTrait> = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());

    let state = AppState::new(
        test_settings_with_bearer(),
        test_inventory(),
        test_playbooks(),
        repo,
        announcer,
        config_dir,
    )
    .expect("failed to create app state");

    create_test_router(state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Public health returns slim response
    assert!(json["status"].is_string(), "status should be a string");
    assert!(json["version"].is_string(), "version should be a string");
    assert!(
        json["auth_mode"].is_string(),
        "auth_mode should be a string"
    );
    // Sensitive fields should NOT be present on public health
    assert!(
        json["bgp_sessions"].is_null(),
        "bgp_sessions should not be on public health"
    );
    assert!(
        json["database"].is_null(),
        "database should not be on public health"
    );
}

#[tokio::test]
async fn test_health_detail_endpoint() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/health/detail")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["status"].is_string(), "status should be a string");
    assert!(json["version"].is_string(), "version should be a string");
    assert!(
        json["bgp_sessions"].is_object(),
        "bgp_sessions should be present on detail"
    );
    assert!(
        json["database"].is_string(),
        "database should be present on detail"
    );
    assert!(
        json["gobgp"].is_object(),
        "gobgp should be present on detail"
    );
    assert!(
        json["uptime_seconds"].is_number(),
        "uptime_seconds should be present on detail"
    );
}

#[tokio::test]
async fn test_config_settings_endpoint() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/config/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["settings"].is_object(), "settings should be present");
    assert!(json["loaded_at"].is_string(), "loaded_at should be present");
    // Verify allowlist redaction: sensitive fields must not appear
    assert_eq!(
        json["settings"]["storage"]["connection_string"],
        "[redacted]"
    );
    assert!(
        json["settings"]["http"]["auth"]["bearer_token_env"].is_null(),
        "bearer_token_env should not be in allowlist"
    );
    assert!(
        json["settings"]["bgp"]["gobgp_grpc"].is_null(),
        "gobgp_grpc should not be in allowlist"
    );
    assert!(
        json["settings"]["bgp"]["router_id"].is_null(),
        "router_id should not be in allowlist"
    );
}

#[tokio::test]
async fn test_config_inventory_endpoint() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/config/inventory")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["customers"].is_array(), "customers should be an array");
    assert!(
        json["total_customers"].is_number(),
        "total_customers should be a number"
    );
    assert!(
        json["total_services"].is_number(),
        "total_services should be a number"
    );
    assert!(
        json["total_assets"].is_number(),
        "total_assets should be a number"
    );
    assert!(json["loaded_at"].is_string(), "loaded_at should be present");
}

#[tokio::test]
async fn test_config_playbooks_endpoint() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/config/playbooks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["playbooks"].is_array(), "playbooks should be an array");
    assert!(
        json["total_playbooks"].is_number(),
        "total_playbooks should be a number"
    );
    assert!(json["loaded_at"].is_string(), "loaded_at should be present");
}

#[tokio::test]
async fn test_list_mitigations_empty() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ingest_event() {
    let app = setup_app().await;

    let event_json = r#"{
        "timestamp": "2026-01-16T14:00:00Z",
        "source": "test",
        "victim_ip": "203.0.113.10",
        "vector": "udp_flood",
        "bps": 100000000,
        "pps": 50000,
        "top_dst_ports": [53],
        "confidence": 0.9
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(event_json))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_list_mitigations_filters_by_victim_ip() {
    let app = setup_app().await;

    let event_a = r#"{
        "timestamp": "2026-01-16T14:00:00Z",
        "source": "test",
        "victim_ip": "203.0.113.10",
        "vector": "udp_flood",
        "pps": 50000
    }"#;
    let event_b = r#"{
        "timestamp": "2026-01-16T14:00:01Z",
        "source": "test",
        "victim_ip": "203.0.113.11",
        "vector": "syn_flood",
        "pps": 25000
    }"#;

    for body in [event_a, event_b] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations?victim_ip=203.0.113.10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["count"], 1);
    assert_eq!(json["mitigations"][0]["victim_ip"], "203.0.113.10");
}

#[tokio::test]
async fn test_bulk_withdraw_mitigations() {
    let app = setup_app().await;

    // Ingest two events for different IPs to create two mitigations
    let events = vec![
        r#"{
            "timestamp": "2026-01-16T14:00:00Z",
            "source": "test",
            "victim_ip": "203.0.113.20",
            "vector": "udp_flood",
            "pps": 50000
        }"#,
        r#"{
            "timestamp": "2026-01-16T14:00:01Z",
            "source": "test",
            "victim_ip": "203.0.113.21",
            "vector": "udp_flood",
            "pps": 25000
        }"#,
    ];

    for body in events {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    // Get mitigation IDs
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let mitigations = json["mitigations"].as_array().unwrap();
    assert!(
        mitigations.len() >= 2,
        "expected at least 2 mitigations, got {}",
        mitigations.len()
    );

    let id1 = mitigations[0]["mitigation_id"].as_str().unwrap();
    let id2 = mitigations[1]["mitigation_id"].as_str().unwrap();

    // Bulk withdraw both plus a fake ID
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let withdraw_body = serde_json::json!({
        "mitigation_ids": [id1, id2, fake_id],
        "operator_id": "test_operator",
        "reason": "bulk test"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/mitigations/withdraw")
                .header("content-type", "application/json")
                .body(Body::from(withdraw_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["withdrawn"], 2);
    assert_eq!(json["failed"], 1);
    assert_eq!(json["results"].as_array().unwrap().len(), 3);
}

// Auth tests with bearer token
fn test_settings_with_bearer() -> Settings {
    let mut settings = test_settings();
    settings.http.auth.mode = AuthMode::Bearer;
    settings.http.auth.bearer_token_env = Some("TEST_PREFIXD_TOKEN".to_string());
    settings
}

async fn setup_app_with_bearer() -> axum::Router {
    // Set the test token in environment
    // SAFETY: Tests run serially, no other threads reading this env var
    unsafe {
        std::env::set_var("TEST_PREFIXD_TOKEN", "test-secret-token-123");
    }

    let repo: Arc<dyn RepositoryTrait> = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());

    let state = AppState::new(
        test_settings_with_bearer(),
        test_inventory(),
        test_playbooks(),
        repo,
        announcer,
        std::path::PathBuf::from("."),
    )
    .expect("failed to create app state");

    create_test_router(state)
}

#[tokio::test]
async fn test_bearer_auth_missing_token_returns_401() {
    let app = setup_app_with_bearer().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_bearer_auth_invalid_token_returns_401() {
    let app = setup_app_with_bearer().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations")
                .header("Authorization", "Bearer wrong-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_bearer_auth_valid_token_returns_200() {
    let app = setup_app_with_bearer().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations")
                .header("Authorization", "Bearer test-secret-token-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_public_endpoint_no_auth_required() {
    let app = setup_app_with_bearer().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_security_headers_present() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .map(|v| v.to_str().unwrap()),
        Some("nosniff")
    );
    assert_eq!(
        response
            .headers()
            .get("x-frame-options")
            .map(|v| v.to_str().unwrap()),
        Some("DENY")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .map(|v| v.to_str().unwrap()),
        Some("no-store")
    );
}

#[tokio::test]
async fn test_timeseries_returns_buckets() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/stats/timeseries?metric=mitigations&range=24h&bucket=1h")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["metric"], "mitigations");
    // MockRepository returns empty buckets
    assert!(json["buckets"].is_array());
}

#[tokio::test]
async fn test_ip_history_returns_structure() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/ip/192.0.2.1/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ip"], "192.0.2.1");
    assert!(json["events"].is_array());
    assert!(json["mitigations"].is_array());
}

#[tokio::test]
async fn test_ip_history_rejects_invalid_ip() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/ip/not-an-ip/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_playbooks_validation_error_returns_400() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/playbooks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"playbooks":[]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_playbooks_invalid_json_returns_400() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/playbooks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"playbooks":"not-an-array"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_playbooks_success_writes_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    let app = setup_app_with_config_dir(config_dir.clone()).await;

    let body = r#"{
        "playbooks": [{
            "name": "syn_discard_test",
            "match": { "vector": "syn_flood", "require_top_ports": true },
            "steps": [
                { "action": "police", "rate_bps": 3000000, "ttl_seconds": 90 },
                { "action": "discard", "ttl_seconds": 240, "require_confidence_at_least": 0.8, "require_persistence_seconds": 120 }
            ]
        }]
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/playbooks")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let written = std::fs::read_to_string(config_dir.join("playbooks.yaml")).unwrap();
    assert!(written.contains("syn_discard_test"));
    assert!(written.contains("syn_flood"));
}

#[tokio::test]
async fn test_update_playbooks_bearer_operator_forbidden() {
    let app = setup_app_with_bearer().await;

    let body = r#"{
        "playbooks": [{
            "name": "test_playbook",
            "match": { "vector": "udp_flood" },
            "steps": [{ "action": "police", "rate_bps": 5000000, "ttl_seconds": 120 }]
        }]
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/playbooks")
                .header("Authorization", "Bearer test-secret-token-123")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ─── Alerting PUT tests ───

#[tokio::test]
async fn test_update_alerting_success() {
    let dir = tempfile::tempdir().unwrap();
    let app = setup_app_with_config_dir(dir.path().to_path_buf()).await;

    let body = serde_json::json!({
        "destinations": [
            {
                "type": "slack",
                "webhook_url": "https://hooks.slack.com/services/T/B/xxx",
                "channel": "#alerts"
            }
        ],
        "events": ["mitigation.created", "mitigation.withdrawn"]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/alerting")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["destinations"][0]["type"], "slack");
    assert_eq!(json["destinations"][0]["webhook_url"], "***");
    assert_eq!(json["events"].as_array().unwrap().len(), 2);

    // Verify file was written
    let alerting_path = dir.path().join("alerting.yaml");
    assert!(alerting_path.exists());
}

#[tokio::test]
async fn test_update_alerting_validation_error() {
    let dir = tempfile::tempdir().unwrap();
    let app = setup_app_with_config_dir(dir.path().to_path_buf()).await;

    let body = serde_json::json!({
        "destinations": [
            {
                "type": "slack",
                "webhook_url": "",
                "channel": "#alerts"
            }
        ],
        "events": []
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/alerting")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_alerting_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    let app = setup_app_with_config_dir(dir.path().to_path_buf()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/alerting")
                .header("content-type", "application/json")
                .body(Body::from("not json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_alerting_operator_forbidden() {
    let dir = tempfile::tempdir().unwrap();
    let app = setup_app_bearer_with_config_dir(dir.path().to_path_buf()).await;

    let body = serde_json::json!({
        "destinations": [],
        "events": []
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/alerting")
                .header("Authorization", "Bearer test-secret-token-123")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_alerting_rejects_link_local_url() {
    let dir = tempfile::tempdir().unwrap();
    let app = setup_app_with_config_dir(dir.path().to_path_buf()).await;

    let body = serde_json::json!({
        "destinations": [
            {
                "type": "generic",
                "url": "https://169.254.169.254/latest/meta-data",
                "secret": "test",
                "headers": {}
            }
        ],
        "events": []
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/config/alerting")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reload_config_reloads_alerting_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().to_path_buf();
    let app = setup_app_with_config_dir(config_dir.clone()).await;

    std::fs::write(
        config_dir.join("alerting.yaml"),
        r##"
destinations:
  - type: slack
    webhook_url: https://hooks.slack.com/services/T/B/C
    channel: "#ops-alerts"
events:
  - mitigation.created
"##,
    )
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/config/reload")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json["reloaded"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "alerting")
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/config/alerting")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["destinations"][0]["type"], "slack");
    assert_eq!(json["events"][0], "mitigation.created");
}

#[tokio::test]
async fn test_cursor_pagination_mitigations() {
    let app = setup_app().await;

    // Ingest 3 events to create 3 mitigations
    for i in 0..3 {
        let body = serde_json::json!({
            "timestamp": format!("2026-01-16T14:00:0{}Z", i),
            "source": "test",
            "victim_ip": format!("203.0.113.{}", 50 + i),
            "vector": "udp_flood",
            "pps": 50000
        });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    // Page 1: limit=2
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations?limit=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["has_more"], true);
    assert!(
        json["next_cursor"].is_string(),
        "next_cursor should be present"
    );

    // Page 2: use cursor
    let cursor = json["next_cursor"].as_str().unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/mitigations?limit=2&cursor={}", cursor))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 1);
    assert_eq!(json["has_more"], false);
    assert!(json["next_cursor"].is_null(), "no more pages");
}

#[tokio::test]
async fn test_cursor_pagination_events() {
    let app = setup_app().await;

    // Ingest 3 events
    for i in 0..3 {
        let body = serde_json::json!({
            "timestamp": format!("2026-01-16T14:00:0{}Z", i),
            "source": "test",
            "victim_ip": format!("203.0.113.{}", 60 + i),
            "vector": "udp_flood",
            "pps": 50000
        });
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // Page 1: limit=2
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/events?limit=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["has_more"], true);

    // Page 2
    let cursor = json["next_cursor"].as_str().unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/events?limit=2&cursor={}", cursor))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 1);
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn test_date_range_filtering_events() {
    let app = setup_app().await;

    // Ingest event
    let body = r#"{
        "timestamp": "2026-01-16T14:00:00Z",
        "source": "test",
        "victim_ip": "203.0.113.70",
        "vector": "udp_flood",
        "pps": 50000
    }"#;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Query with date range that includes the event
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/events?start=2020-01-01T00:00:00Z&end=2030-01-01T00:00:00Z")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["count"].as_u64().unwrap() >= 1);

    // Query with date range that excludes the event (future only)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events?start=2030-01-01T00:00:00Z")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 0);
}

#[tokio::test]
async fn test_bulk_acknowledge_mitigations() {
    let app = setup_app().await;

    // Ingest 2 events
    for i in 0..2 {
        let body = serde_json::json!({
            "timestamp": format!("2026-01-16T14:00:0{}Z", i),
            "source": "test",
            "victim_ip": format!("203.0.113.{}", 80 + i),
            "vector": "udp_flood",
            "pps": 50000
        });
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // Get mitigation IDs
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let mitigations = json["mitigations"].as_array().unwrap();
    assert!(mitigations.len() >= 2);

    let id1 = mitigations[0]["mitigation_id"].as_str().unwrap();
    let id2 = mitigations[1]["mitigation_id"].as_str().unwrap();
    let fake_id = "00000000-0000-0000-0000-000000000000";

    // Bulk acknowledge: 2 real + 1 fake
    let ack_body = serde_json::json!({
        "mitigation_ids": [id1, id2, fake_id],
        "operator_id": "test_operator"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/mitigations/acknowledge")
                .header("content-type", "application/json")
                .body(Body::from(ack_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["acknowledged"], 2);
    assert_eq!(json["failed"], 1);

    // Verify acknowledged filter: acknowledged=true
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations?acknowledged=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);

    // acknowledged=false should return 0
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations?acknowledged=false")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 0);

    // Re-acknowledge should fail (already acknowledged)
    let ack_body = serde_json::json!({
        "mitigation_ids": [id1],
        "operator_id": "test_operator"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/mitigations/acknowledge")
                .header("content-type", "application/json")
                .body(Body::from(ack_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["acknowledged"], 0);
    assert_eq!(json["failed"], 1);
}

// ---------------------------------------------------------------------------
// Event batch ingestion
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_batch_event_ingestion_all_accepted() {
    let app = setup_app().await;

    let body = serde_json::json!({
        "events": [
            {
                "timestamp": "2026-01-16T14:00:00Z",
                "source": "fastnetmon",
                "victim_ip": "203.0.113.200",
                "vector": "udp_flood",
                "pps": 50000
            },
            {
                "timestamp": "2026-01-16T14:00:01Z",
                "source": "fastnetmon",
                "victim_ip": "203.0.113.201",
                "vector": "syn_flood",
                "bps": 1000000000
            },
            {
                "timestamp": "2026-01-16T14:00:02Z",
                "source": "fastnetmon",
                "victim_ip": "203.0.113.202",
                "vector": "icmp_flood",
                "pps": 100000
            }
        ]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events/batch")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["accepted"], 3);
    assert_eq!(json["rejected"], 0);
    let results = json["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    for (i, r) in results.iter().enumerate() {
        assert_eq!(r["index"], i);
        assert!(r["event_id"].is_string());
        assert!(!r["status"].as_str().unwrap().contains("rejected"));
    }
}

#[tokio::test]
async fn test_batch_event_ingestion_partial_success() {
    let app = setup_app().await;

    let body = serde_json::json!({
        "events": [
            {
                "timestamp": "2026-01-16T14:00:00Z",
                "source": "fastnetmon",
                "victim_ip": "203.0.113.210",
                "vector": "udp_flood",
                "pps": 50000
            },
            {
                "timestamp": "2026-01-16T14:00:01Z",
                "source": "fastnetmon",
                "victim_ip": "not_an_ip",
                "vector": "syn_flood"
            },
            {
                "timestamp": "2026-01-16T14:00:02Z",
                "source": "fastnetmon",
                "victim_ip": "203.0.113.211",
                "vector": "ack_flood",
                "pps": 80000
            }
        ]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events/batch")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::MULTI_STATUS);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["accepted"], 2);
    assert_eq!(json["rejected"], 1);
    let results = json["results"].as_array().unwrap();
    assert_eq!(results[1]["status"], "rejected");
    assert!(results[1]["error"].as_str().unwrap().contains("IP"));
}

#[tokio::test]
async fn test_batch_event_ingestion_empty_batch() {
    let app = setup_app().await;

    let body = serde_json::json!({ "events": [] });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events/batch")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_batch_event_ingestion_exceeds_limit() {
    let app = setup_app().await;

    let events: Vec<serde_json::Value> = (0..101)
        .map(|i| {
            serde_json::json!({
                "timestamp": "2026-01-16T14:00:00Z",
                "source": "test",
                "victim_ip": format!("203.0.113.{}", i % 256),
                "vector": "udp_flood"
            })
        })
        .collect();

    let body = serde_json::json!({ "events": events });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events/batch")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Incident reports
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_incident_report_by_mitigation_id() {
    let app = setup_app().await;

    // Ingest event to create a mitigation
    let body = serde_json::json!({
        "timestamp": "2026-01-16T14:00:00Z",
        "source": "fastnetmon",
        "victim_ip": "203.0.113.230",
        "vector": "udp_flood",
        "bps": 5000000000i64,
        "pps": 500000
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let event_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let mitigation_id = event_json["mitigation_id"].as_str().unwrap_or_default();

    // Generate report by mitigation_id
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!(
                    "/v1/reports/incident?mitigation_id={}",
                    mitigation_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let ct = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("text/markdown"));

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let markdown = String::from_utf8(body.to_vec()).unwrap();
    assert!(markdown.contains("# Incident Report"));
    assert!(markdown.contains("203.0.113.230"));
    assert!(markdown.contains("## Summary"));
    assert!(markdown.contains("## Timeline"));
    assert!(markdown.contains("## Events"));
    assert!(markdown.contains("## Mitigations"));
}

#[tokio::test]
async fn test_incident_report_by_ip() {
    let app = setup_app().await;

    // Ingest event
    let body = serde_json::json!({
        "timestamp": "2026-01-16T15:00:00Z",
        "source": "fastnetmon",
        "victim_ip": "203.0.113.231",
        "vector": "syn_flood",
        "bps": 2000000000i64
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Generate report by IP
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/reports/incident?ip=203.0.113.231")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let markdown = String::from_utf8(body.to_vec()).unwrap();
    assert!(markdown.contains("# Incident Report"));
    assert!(markdown.contains("203.0.113.231"));
}

#[tokio::test]
async fn test_incident_report_missing_params() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/reports/incident")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Per-destination event routing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_alerting_config_per_destination_events_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let alerting_path = dir.path().join("alerting.yaml");
    std::fs::write(
        &alerting_path,
        r#"
destinations:
  - type: generic
    url: "https://example.com/all"
  - type: generic
    url: "https://example.com/critical"
    events:
      - mitigation.created
      - mitigation.escalated
events:
  - mitigation.created
  - mitigation.withdrawn
"#,
    )
    .unwrap();

    let config = prefixd::alerting::AlertingConfig::load(&alerting_path).unwrap();
    assert_eq!(config.destinations.len(), 2);

    // First dest has no per-destination events
    assert!(config.destinations[0].events().is_empty());
    // Second dest has per-destination events
    assert_eq!(config.destinations[1].events().len(), 2);

    // Save and reload
    config.save(&alerting_path).unwrap();
    let reloaded = prefixd::alerting::AlertingConfig::load(&alerting_path).unwrap();
    assert_eq!(reloaded.destinations[1].events().len(), 2);
}

// ---------------------------------------------------------------------------
// Notification preferences API
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_notification_preferences_get_default() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/preferences")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["muted_events"], serde_json::json!([]));
    assert_eq!(json["quiet_hours_start"], serde_json::Value::Null);
    assert_eq!(json["quiet_hours_end"], serde_json::Value::Null);
}

#[tokio::test]
async fn test_notification_preferences_put_invalid_hour() {
    let app = setup_app().await;

    let body = serde_json::json!({
        "muted_events": [],
        "quiet_hours_start": 25,
        "quiet_hours_end": 8,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/preferences")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_notification_preferences_put_invalid_event() {
    let app = setup_app().await;

    let body = serde_json::json!({
        "muted_events": ["not.a.real.event"],
        "quiet_hours_start": null,
        "quiet_hours_end": null,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/preferences")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_notification_preferences_put_half_configured_quiet_hours() {
    let app = setup_app().await;

    let body = serde_json::json!({
        "muted_events": [],
        "quiet_hours_start": 2,
        "quiet_hours_end": null,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/preferences")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_notification_preferences_response_includes_null_quiet_hours() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/preferences")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let raw = String::from_utf8_lossy(&body);
    assert!(raw.contains("\"quiet_hours_start\":null"));
    assert!(raw.contains("\"quiet_hours_end\":null"));
}

// ==========================================================================
// Correlation integration tests
// ==========================================================================

fn test_settings_with_correlation(
    enabled: bool,
    min_sources: u32,
    confidence_threshold: f32,
) -> Settings {
    let mut settings = test_settings();
    settings.correlation = prefixd::correlation::CorrelationConfig {
        enabled,
        window_seconds: 300,
        min_sources,
        confidence_threshold,
        sources: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "detector_a".to_string(),
                prefixd::correlation::SourceConfig {
                    weight: 1.0,
                    r#type: "detector".to_string(),
                },
            );
            m.insert(
                "detector_b".to_string(),
                prefixd::correlation::SourceConfig {
                    weight: 1.5,
                    r#type: "detector".to_string(),
                },
            );
            m
        },
        default_weight: 1.0,
    };
    settings
}

async fn setup_app_correlation(
    enabled: bool,
    min_sources: u32,
    confidence_threshold: f32,
) -> axum::Router {
    let repo: Arc<dyn RepositoryTrait> = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());
    let settings = test_settings_with_correlation(enabled, min_sources, confidence_threshold);

    let state = AppState::new(
        settings,
        test_inventory(),
        test_playbooks(),
        repo,
        announcer,
        std::path::PathBuf::from("."),
    )
    .expect("failed to create app state");

    create_test_router(state)
}

fn make_event_json(source: &str, victim_ip: &str, confidence: f32) -> String {
    format!(
        r#"{{
            "timestamp": "2026-01-16T14:00:00Z",
            "source": "{}",
            "victim_ip": "{}",
            "vector": "udp_flood",
            "bps": 100000000,
            "pps": 50000,
            "top_dst_ports": [53],
            "confidence": {}
        }}"#,
        source, victim_ip, confidence
    )
}

async fn post_event(app: &axum::Router, event_json: &str) -> (StatusCode, serde_json::Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(event_json.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

/// VAL-ENGINE-010: Single source triggers when min_sources=1 (backward compat)
#[tokio::test]
async fn test_correlation_min_sources_1_triggers_immediately() {
    let app = setup_app_correlation(true, 1, 0.5).await;

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (status, json) = post_event(&app, &event).await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(json["status"], "accepted");
    assert!(
        json["mitigation_id"].is_string(),
        "should create mitigation with min_sources=1: {:?}",
        json
    );
}

/// VAL-ENGINE-009: min_sources=2 and one source does NOT create mitigation
#[tokio::test]
async fn test_correlation_min_sources_2_one_source_no_mitigation() {
    let app = setup_app_correlation(true, 2, 0.5).await;

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (status, json) = post_event(&app, &event).await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(json["status"], "accepted");
    assert!(
        json["mitigation_id"].is_null(),
        "should NOT create mitigation with 1 source when min_sources=2"
    );
}

/// VAL-ENGINE-009: min_sources=2 and two sources creates mitigation
#[tokio::test]
async fn test_correlation_min_sources_2_two_sources_creates_mitigation() {
    let app = setup_app_correlation(true, 2, 0.5).await;

    // First event from detector_a — no mitigation
    let event_a = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (status_a, json_a) = post_event(&app, &event_a).await;
    assert_eq!(status_a, StatusCode::ACCEPTED);
    assert!(
        json_a["mitigation_id"].is_null(),
        "first source alone shouldn't trigger"
    );

    // Second event from detector_b — mitigation created
    let event_b = make_event_json("detector_b", "203.0.113.10", 0.8);
    let (status_b, json_b) = post_event(&app, &event_b).await;
    assert_eq!(status_b, StatusCode::ACCEPTED);
    assert_eq!(json_b["status"], "accepted");
    assert!(
        json_b["mitigation_id"].is_string(),
        "second source should trigger mitigation: {:?}",
        json_b
    );
}

/// VAL-ENGINE-020: Events bypass correlation when disabled
#[tokio::test]
async fn test_correlation_disabled_bypasses_entirely() {
    let app = setup_app_correlation(false, 2, 0.5).await;

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (status, json) = post_event(&app, &event).await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(json["status"], "accepted");
    assert!(
        json["mitigation_id"].is_string(),
        "should create mitigation immediately when correlation disabled"
    );
}

/// VAL-ENGINE-029: EventResponse shape unchanged
#[tokio::test]
async fn test_correlation_event_response_shape_unchanged() {
    let app = setup_app_correlation(true, 1, 0.5).await;

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (status, json) = post_event(&app, &event).await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(json["event_id"].is_string(), "event_id must be present");
    assert!(json["status"].is_string(), "status must be present");
    // mitigation_id may be string or null
    assert!(
        json["mitigation_id"].is_string() || json["mitigation_id"].is_null(),
        "mitigation_id must be string or null"
    );
}

/// VAL-ENGINE-013: Derived confidence must meet threshold
#[tokio::test]
async fn test_correlation_low_confidence_no_mitigation() {
    // Two sources but very low confidence with threshold 0.7
    let app = setup_app_correlation(true, 2, 0.7).await;

    let event_a = make_event_json("detector_a", "203.0.113.10", 0.3);
    post_event(&app, &event_a).await;

    let event_b = make_event_json("detector_b", "203.0.113.10", 0.3);
    let (status, json) = post_event(&app, &event_b).await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(
        json["mitigation_id"].is_null(),
        "low confidence should not trigger even with 2 sources: {:?}",
        json
    );
}

/// VAL-ENGINE-011: Duplicate source counts as one for corroboration
#[tokio::test]
async fn test_correlation_duplicate_source_counts_as_one() {
    let app = setup_app_correlation(true, 2, 0.5).await;

    // Two events from same source
    let event_a = make_event_json("detector_a", "203.0.113.10", 0.9);
    post_event(&app, &event_a).await;

    // Use a different event_id to avoid duplicate detection
    let event_b = make_event_json("detector_a", "203.0.113.10", 0.8);
    let (status, json) = post_event(&app, &event_b).await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(
        json["mitigation_id"].is_null(),
        "same source twice should count as 1 distinct, not trigger with min_sources=2"
    );
}

/// VAL-ENGINE-030: Batch endpoint works with correlation
#[tokio::test]
async fn test_correlation_batch_endpoint_independent_groups() {
    let app = setup_app_correlation(true, 1, 0.5).await;

    let batch_json = r#"{
        "events": [
            {
                "timestamp": "2026-01-16T14:00:00Z",
                "source": "detector_a",
                "victim_ip": "203.0.113.10",
                "vector": "udp_flood",
                "bps": 100000000,
                "pps": 50000,
                "top_dst_ports": [53],
                "confidence": 0.9
            },
            {
                "timestamp": "2026-01-16T14:00:01Z",
                "source": "detector_a",
                "victim_ip": "203.0.113.11",
                "vector": "udp_flood",
                "bps": 50000000,
                "pps": 25000,
                "top_dst_ports": [53],
                "confidence": 0.8
            }
        ]
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events/batch")
                .header("content-type", "application/json")
                .body(Body::from(batch_json))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be 202 (all accepted) with min_sources=1
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["accepted"], 2);
    // Each event should create a mitigation independently
    let results = json["results"].as_array().unwrap();
    assert!(results[0]["mitigation_id"].is_string());
    assert!(results[1]["mitigation_id"].is_string());
    // Different victim IPs = different mitigations
    assert_ne!(results[0]["mitigation_id"], results[1]["mitigation_id"]);
}

/// VAL-ENGINE-018 / VAL-ENGINE-033: Mitigation detail/list includes correlation context
#[tokio::test]
async fn test_correlation_mitigation_detail_includes_correlation() {
    let app = setup_app_correlation(true, 1, 0.5).await;

    // Create a correlated mitigation
    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (_, event_json) = post_event(&app, &event).await;
    let mitigation_id = event_json["mitigation_id"].as_str().unwrap();

    // GET /v1/mitigations/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/mitigations/{}", mitigation_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have correlation field
    assert!(
        json["correlation"].is_object(),
        "correlation should be present: {:?}",
        json
    );
    let corr = &json["correlation"];
    assert!(corr["signal_group_id"].is_string());
    assert!(corr["derived_confidence"].is_number());
    assert!(corr["source_count"].is_number());
    assert!(corr["corroboration_met"].is_boolean());
    assert!(corr["contributing_sources"].is_array());
    assert!(corr["explanation"].is_string());
}

/// VAL-ENGINE-019: Non-correlated mitigation has null correlation
#[tokio::test]
async fn test_correlation_disabled_mitigation_no_correlation_field() {
    let app = setup_app_correlation(false, 1, 0.5).await;

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (_, event_json) = post_event(&app, &event).await;
    let mitigation_id = event_json["mitigation_id"].as_str().unwrap();

    // GET /v1/mitigations/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/mitigations/{}", mitigation_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // correlation should be absent (skipped when None)
    assert!(
        json["correlation"].is_null(),
        "correlation should be null/absent when disabled: {:?}",
        json
    );
}

/// VAL-ENGINE-004: Signal group resolves when mitigation created
#[tokio::test]
async fn test_correlation_signal_group_resolves_on_mitigation() {
    let repo = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());
    let settings = test_settings_with_correlation(true, 1, 0.5);

    let state = AppState::new(
        settings,
        test_inventory(),
        test_playbooks(),
        repo.clone(),
        announcer,
        std::path::PathBuf::from("."),
    )
    .expect("failed to create app state");

    let app = create_test_router(state);

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    post_event(&app, &event).await;

    // Check that the signal group was resolved
    let groups = repo
        .list_signal_groups(
            &prefixd::correlation::SignalGroupFilter {
                status: Some(prefixd::correlation::SignalGroupStatus::Resolved),
                ..Default::default()
            },
            &prefixd::db::ListParams {
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(groups.len(), 1, "should have one resolved group");
    assert!(
        groups[0].corroboration_met,
        "corroboration_met should be true"
    );
    assert_eq!(groups[0].source_count, 1);
}

/// VAL-ENGINE-033: Mitigations list includes correlation summary
#[tokio::test]
async fn test_correlation_mitigations_list_includes_summary() {
    let app = setup_app_correlation(true, 1, 0.5).await;

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    post_event(&app, &event).await;

    // GET /v1/mitigations
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/mitigations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let mitigations = json["mitigations"].as_array().unwrap();
    assert!(!mitigations.is_empty());
    assert!(
        mitigations[0]["correlation"].is_object(),
        "list should include correlation summary: {:?}",
        mitigations[0]
    );
    assert!(mitigations[0]["correlation"]["signal_group_id"].is_string());
}

/// VAL-CROSS-009: Corroborated mitigations pass through guardrails — safelisted IP rejected
#[tokio::test]
async fn test_correlation_guardrails_still_apply() {
    // Create app with safelist
    let repo = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());
    let settings = test_settings_with_correlation(true, 1, 0.5);

    // Add IP to safelist
    repo.insert_safelist("203.0.113.10", "admin", Some("core router"))
        .await
        .unwrap();

    let state = AppState::new(
        settings,
        test_inventory(),
        test_playbooks(),
        repo,
        announcer,
        std::path::PathBuf::from("."),
    )
    .expect("failed to create app state");

    let app = create_test_router(state);

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    let (status, json) = post_event(&app, &event).await;

    // Should be rejected by guardrails
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(json["error"].as_str().unwrap().contains("safelist"));
}

// ── Signal Groups API Tests ────────────────────────────────────────────

/// Helper: create an app with correlation enabled and a shared repo reference
async fn setup_app_correlation_with_repo(
    min_sources: u32,
    confidence_threshold: f32,
) -> (axum::Router, Arc<dyn RepositoryTrait>) {
    let repo: Arc<dyn RepositoryTrait> = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());
    let settings = test_settings_with_correlation(true, min_sources, confidence_threshold);

    let state = AppState::new(
        settings,
        test_inventory(),
        test_playbooks(),
        repo.clone(),
        announcer,
        std::path::PathBuf::from("."),
    )
    .expect("failed to create app state");

    (create_test_router(state), repo)
}

/// VAL-ENGINE-016: GET /v1/signal-groups returns paginated list with cursor, has_more
#[tokio::test]
async fn test_signal_groups_list_basic() {
    let (app, _repo) = setup_app_correlation_with_repo(1, 0.5).await;

    // Ingest events to create signal groups
    let event1 = make_event_json("detector_a", "203.0.113.10", 0.9);
    let event2 = make_event_json("detector_a", "203.0.113.11", 0.8);
    post_event(&app, &event1).await;
    post_event(&app, &event2).await;

    // GET /v1/signal-groups
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["groups"].is_array());
    assert_eq!(json["groups"].as_array().unwrap().len(), 2);
    assert_eq!(json["count"], 2);
    assert!(!json["has_more"].as_bool().unwrap());
    assert!(json["next_cursor"].is_null());
}

/// VAL-ENGINE-016: Cursor pagination works correctly
#[tokio::test]
async fn test_signal_groups_list_pagination() {
    let (app, _repo) = setup_app_correlation_with_repo(1, 0.5).await;

    // Create 3 signal groups (3 different IPs)
    for i in 10..13 {
        let event = make_event_json("detector_a", &format!("203.0.113.{}", i), 0.9);
        post_event(&app, &event).await;
    }

    // Request page 1 with limit=2
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?limit=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["groups"].as_array().unwrap().len(), 2);
    assert_eq!(json["count"], 2);
    assert!(json["has_more"].as_bool().unwrap());
    let cursor = json["next_cursor"].as_str().unwrap();

    // Request page 2 using cursor
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/signal-groups?limit=2&cursor={}", cursor))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json2: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json2["groups"].as_array().unwrap().len(), 1);
    assert_eq!(json2["count"], 1);
    assert!(!json2["has_more"].as_bool().unwrap());
}

/// VAL-ENGINE-016: Status filter returns only matching groups
#[tokio::test]
async fn test_signal_groups_list_status_filter() {
    let (app, _repo) = setup_app_correlation_with_repo(1, 0.5).await;

    // Create events → signal groups with min_sources=1 → groups become resolved
    let event1 = make_event_json("detector_a", "203.0.113.10", 0.9);
    let event2 = make_event_json("detector_a", "203.0.113.11", 0.8);
    post_event(&app, &event1).await;
    post_event(&app, &event2).await;

    // With min_sources=1 and confidence above threshold, groups should be resolved
    // Filter for resolved
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?status=resolved")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // All groups should be resolved since min_sources=1 and confidence >= 0.5
    for group in json["groups"].as_array().unwrap() {
        assert_eq!(group["status"], "resolved");
    }

    // Filter for open — should return 0 since all were resolved
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?status=open")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["groups"].as_array().unwrap().len(), 0);
}

/// VAL-ENGINE-016: Vector filter returns only matching groups
#[tokio::test]
async fn test_signal_groups_list_vector_filter() {
    let (app, _repo) = setup_app_correlation_with_repo(1, 0.5).await;

    // Create events (all go through udp_flood playbook)
    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    post_event(&app, &event).await;

    // Filter for udp_flood (should match)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?vector=udp_flood")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["groups"].as_array().unwrap().len() >= 1);

    // Filter for syn_flood (should not match)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?vector=syn_flood")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["groups"].as_array().unwrap().len(), 0);
}

/// VAL-ENGINE-032: Date range filter works with start/end params
#[tokio::test]
async fn test_signal_groups_list_date_range_filter() {
    let (app, _repo) = setup_app_correlation_with_repo(1, 0.5).await;

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    post_event(&app, &event).await;

    // Use a future start date — should return 0 groups
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?start=2099-01-01T00:00:00Z")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["groups"].as_array().unwrap().len(), 0);

    // Use a past start date — should return groups
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?start=2020-01-01T00:00:00Z")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["groups"].as_array().unwrap().len() >= 1);

    // Use a past end date — should return 0 groups
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups?end=2020-01-01T00:00:00Z")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["groups"].as_array().unwrap().len(), 0);
}

/// VAL-ENGINE-017: GET /v1/signal-groups/{id} returns group detail with contributing events
#[tokio::test]
async fn test_signal_group_detail_with_events() {
    let (app, _repo) = setup_app_correlation_with_repo(1, 0.5).await;

    // Create an event to generate a signal group
    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    post_event(&app, &event).await;

    // List groups to get the group ID
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let group_id = json["groups"][0]["group_id"].as_str().unwrap().to_string();

    // GET /v1/signal-groups/{id}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/signal-groups/{}", group_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let detail: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify group metadata
    assert_eq!(detail["group_id"], group_id);
    assert_eq!(detail["victim_ip"], "203.0.113.10");
    assert_eq!(detail["vector"], "udp_flood");
    assert!(detail["derived_confidence"].is_number());
    assert!(detail["source_count"].is_number());
    assert!(detail["status"].is_string());
    assert!(detail["corroboration_met"].is_boolean());

    // Verify events list
    assert!(detail["events"].is_array());
    let events = detail["events"].as_array().unwrap();
    assert!(!events.is_empty());
    let ev = &events[0];
    assert!(ev["event_id"].is_string());
    assert!(ev["source_weight"].is_number());
    assert!(ev["source"].is_string());
    assert!(ev["confidence"].is_number());
    assert!(ev["ingested_at"].is_string());
}

/// VAL-ENGINE-017: GET /v1/signal-groups/{id} returns 404 for unknown group
#[tokio::test]
async fn test_signal_group_detail_not_found() {
    let (app, _repo) = setup_app_correlation_with_repo(1, 0.5).await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/signal-groups/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// VAL-ENGINE-016/017: Both endpoints require authentication (401 without)
#[tokio::test]
async fn test_signal_groups_auth_required() {
    // Create app with bearer auth
    let repo: Arc<dyn RepositoryTrait> = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());
    let mut settings = test_settings_with_correlation(true, 1, 0.5);
    settings.http.auth = prefixd::config::AuthConfig {
        mode: prefixd::config::AuthMode::Bearer,
        bearer_token_env: Some("TEST_PREFIXD_TOKEN".to_string()),
        ldap: None,
        radius: None,
    };
    unsafe {
        std::env::set_var("TEST_PREFIXD_TOKEN", "test-secret-token-123");
    }

    let state = AppState::new(
        settings,
        test_inventory(),
        test_playbooks(),
        repo,
        announcer,
        std::path::PathBuf::from("."),
    )
    .expect("failed to create app state");

    let app = create_test_router(state);

    // GET /v1/signal-groups without auth → 401
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // GET /v1/signal-groups/{id} without auth → 401
    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/signal-groups/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // GET /v1/signal-groups with auth → 200
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups")
                .header("authorization", "Bearer test-secret-token-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// VAL-ENGINE-034: OpenAPI spec includes signal groups endpoints
#[tokio::test]
async fn test_openapi_includes_signal_groups() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let spec: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let paths = spec["paths"].as_object().unwrap();
    assert!(
        paths.contains_key("/v1/signal-groups"),
        "OpenAPI spec should include /v1/signal-groups"
    );
    assert!(
        paths.contains_key("/v1/signal-groups/{id}"),
        "OpenAPI spec should include /v1/signal-groups/{{id}}"
    );

    // Verify schemas are registered
    let schemas = spec["components"]["schemas"].as_object().unwrap();
    assert!(
        schemas.contains_key("SignalGroup"),
        "OpenAPI spec should include SignalGroup schema"
    );
    assert!(
        schemas.contains_key("SignalGroupEvent"),
        "OpenAPI spec should include SignalGroupEvent schema"
    );
    assert!(
        schemas.contains_key("SignalGroupsListResponse"),
        "OpenAPI spec should include SignalGroupsListResponse schema"
    );
    assert!(
        schemas.contains_key("SignalGroupDetailResponse"),
        "OpenAPI spec should include SignalGroupDetailResponse schema"
    );
    assert!(
        schemas.contains_key("CorrelationContext"),
        "OpenAPI spec should include CorrelationContext schema"
    );
    assert!(
        schemas.contains_key("CorrelationExplanation"),
        "OpenAPI spec should include CorrelationExplanation schema"
    );
    assert!(
        schemas.contains_key("SourceContribution"),
        "OpenAPI spec should include SourceContribution schema"
    );
}

/// VAL-ENGINE-017: Signal group detail with multiple contributing events
#[tokio::test]
async fn test_signal_group_detail_multiple_events() {
    // Use min_sources=2 so the group stays open after first event
    let (app, _repo) = setup_app_correlation_with_repo(2, 0.5).await;

    // Submit events from 2 different sources for same victim/vector
    let event1 = make_event_json("detector_a", "203.0.113.10", 0.9);
    let event2 = make_event_json("detector_b", "203.0.113.10", 0.7);
    post_event(&app, &event1).await;
    post_event(&app, &event2).await;

    // List groups — should have exactly 1 group
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/signal-groups")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json["groups"].as_array().unwrap().len(),
        1,
        "Should have exactly one signal group for same (victim_ip, vector)"
    );

    let group_id = json["groups"][0]["group_id"].as_str().unwrap().to_string();

    // Get detail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/v1/signal-groups/{}", group_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let detail: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have 2 contributing events
    let events = detail["events"].as_array().unwrap();
    assert_eq!(events.len(), 2, "Should have 2 contributing events");

    // Verify both sources are represented
    let sources: Vec<&str> = events.iter().filter_map(|e| e["source"].as_str()).collect();
    assert!(sources.contains(&"detector_a"));
    assert!(sources.contains(&"detector_b"));

    // Verify source_weight values
    for ev in events {
        assert!(ev["source_weight"].as_f64().unwrap() > 0.0);
    }
}

/// VAL-CROSS-012: Incident reports include correlation data
#[tokio::test]
async fn test_correlation_incident_report_includes_correlation() {
    let repo = Arc::new(MockRepository::new());
    let announcer = Arc::new(MockAnnouncer::new());
    let settings = test_settings_with_correlation(true, 1, 0.5);

    let state = AppState::new(
        settings,
        test_inventory(),
        test_playbooks(),
        repo.clone(),
        announcer,
        std::path::PathBuf::from("."),
    )
    .expect("failed to create app state");

    let app = create_test_router(state);

    let event = make_event_json("detector_a", "203.0.113.10", 0.9);
    post_event(&app, &event).await;

    // Get incident report
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/reports/incident?ip=203.0.113.10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let md = String::from_utf8_lossy(&body);

    assert!(
        md.contains("## Correlation"),
        "incident report should include Correlation section: {}",
        md
    );
    assert!(
        md.contains("Derived Confidence"),
        "incident report should include derived confidence"
    );
    assert!(
        md.contains("Source Count"),
        "incident report should include source count"
    );
}
