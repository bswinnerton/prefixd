use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use prefixd::config::{
    AllowedPorts, Asset, AuthConfig, AuthMode, BgpConfig, BgpMode, Customer, EscalationConfig,
    GuardrailsConfig, HttpConfig, Inventory, ObservabilityConfig, Playbook, PlaybookAction,
    PlaybookMatch, PlaybookStep, Playbooks, QuotasConfig, RateLimitConfig, SafelistConfig,
    Service, Settings, ShutdownConfig, StorageConfig, StorageDriver, TimersConfig,
};
use prefixd::db;
use prefixd::domain::{ActionParams, ActionType, AttackVector, MatchCriteria, Mitigation, MitigationStatus};

fn test_settings() -> Settings {
    Settings {
        pop: "bench1".to_string(),
        mode: prefixd::config::OperationMode::DryRun,
        http: HttpConfig {
            listen: "127.0.0.1:0".to_string(),
            auth: AuthConfig {
                mode: AuthMode::None,
                bearer_token_env: None,
            },
            rate_limit: RateLimitConfig::default(),
            tls: None,
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
            max_active_per_customer: 100,
            max_active_per_pop: 1000,
            max_active_global: 5000,
            max_new_per_minute: 1000,
            max_announcements_per_peer: 1000,
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
            driver: StorageDriver::Sqlite,
            path: ":memory:".to_string(),
        },
        observability: ObservabilityConfig {
            log_format: prefixd::config::LogFormat::Pretty,
            log_level: "warn".to_string(),
            audit_log_path: "/dev/null".to_string(),
            metrics_listen: "127.0.0.1:0".to_string(),
        },
        safelist: SafelistConfig { prefixes: vec![] },
        shutdown: ShutdownConfig::default(),
    }
}

fn test_inventory() -> Inventory {
    let mut customers = Vec::new();
    for i in 0..100 {
        customers.push(Customer {
            customer_id: format!("cust_{}", i),
            name: format!("Customer {}", i),
            prefixes: vec![format!("203.0.{}.0/24", i)],
            policy_profile: prefixd::config::PolicyProfile::Normal,
            services: vec![Service {
                service_id: format!("svc_{}", i),
                name: "DNS".to_string(),
                assets: (0..10)
                    .map(|j| Asset {
                        ip: format!("203.0.{}.{}", i, j + 10),
                        role: Some("server".to_string()),
                    })
                    .collect(),
                allowed_ports: AllowedPorts {
                    udp: vec![53],
                    tcp: vec![53, 80, 443],
                },
            }],
        });
    }
    Inventory::new(customers)
}

fn test_playbooks() -> Playbooks {
    Playbooks {
        playbooks: vec![
            Playbook {
                name: "udp_flood".to_string(),
                match_criteria: PlaybookMatch {
                    vector: AttackVector::UdpFlood,
                    require_top_ports: false,
                },
                steps: vec![PlaybookStep {
                    action: PlaybookAction::Police,
                    rate_bps: Some(5_000_000),
                    ttl_seconds: 120,
                    require_confidence_at_least: None,
                    require_persistence_seconds: None,
                }],
            },
            Playbook {
                name: "syn_flood".to_string(),
                match_criteria: PlaybookMatch {
                    vector: AttackVector::SynFlood,
                    require_top_ports: false,
                },
                steps: vec![PlaybookStep {
                    action: PlaybookAction::Discard,
                    rate_bps: None,
                    ttl_seconds: 180,
                    require_confidence_at_least: None,
                    require_persistence_seconds: None,
                }],
            },
        ],
    }
}

fn make_mitigation(i: usize) -> Mitigation {
    Mitigation {
        mitigation_id: uuid::Uuid::new_v4(),
        scope_hash: format!("hash_{}", i),
        pop: "bench1".to_string(),
        customer_id: Some(format!("cust_{}", i % 100)),
        service_id: Some(format!("svc_{}", i % 100)),
        victim_ip: format!("203.0.{}.{}", i % 100, (i % 10) + 10),
        vector: AttackVector::UdpFlood,
        match_criteria: MatchCriteria {
            dst_prefix: format!("203.0.{}.{}/32", i % 100, (i % 10) + 10),
            protocol: Some(17),
            dst_ports: vec![53],
        },
        action_type: ActionType::Police,
        action_params: ActionParams {
            rate_bps: Some(5_000_000),
        },
        status: MitigationStatus::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(120),
        withdrawn_at: None,
        triggering_event_id: uuid::Uuid::new_v4(),
        last_event_id: uuid::Uuid::new_v4(),
        escalated_from_id: None,
        reason: "benchmark".to_string(),
        rejection_reason: None,
    }
}

// Benchmark: Inventory IP lookup
fn bench_inventory_lookup(c: &mut Criterion) {
    let inventory = test_inventory();

    c.bench_function("inventory_lookup_hit", |b| {
        b.iter(|| black_box(inventory.lookup_ip("203.0.50.15")))
    });

    c.bench_function("inventory_lookup_miss", |b| {
        b.iter(|| black_box(inventory.lookup_ip("10.0.0.1")))
    });

    c.bench_function("inventory_is_owned", |b| {
        b.iter(|| black_box(inventory.is_owned("203.0.50.15")))
    });
}

// Benchmark: Scope hash computation
fn bench_scope_hash(c: &mut Criterion) {
    let criteria = MatchCriteria {
        dst_prefix: "203.0.1.10/32".to_string(),
        protocol: Some(17),
        dst_ports: vec![53, 80, 443],
    };

    c.bench_function("scope_hash_compute", |b| {
        b.iter(|| black_box(criteria.compute_scope_hash()))
    });
}

// Benchmark: Database operations (async)
fn bench_database_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("db_insert_mitigation", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let pool = db::init_memory_pool().await.unwrap();
            let repo = db::Repository::from_sqlite(pool);

            let start = std::time::Instant::now();
            for i in 0..iters {
                let m = make_mitigation(i as usize);
                let _ = repo.insert_mitigation(&m).await;
            }
            start.elapsed()
        })
    });

    c.bench_function("db_get_mitigation", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let pool = db::init_memory_pool().await.unwrap();
            let repo = db::Repository::from_sqlite(pool);

            // Insert some mitigations first
            let mut ids = Vec::new();
            for i in 0..100 {
                let m = make_mitigation(i);
                repo.insert_mitigation(&m).await.unwrap();
                ids.push(m.mitigation_id);
            }

            let start = std::time::Instant::now();
            for i in 0..iters {
                let id = ids[i as usize % ids.len()];
                let _ = repo.get_mitigation(id).await;
            }
            start.elapsed()
        })
    });

    c.bench_function("db_list_mitigations", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let pool = db::init_memory_pool().await.unwrap();
            let repo = db::Repository::from_sqlite(pool);

            // Insert mitigations
            for i in 0..100 {
                let m = make_mitigation(i);
                repo.insert_mitigation(&m).await.unwrap();
            }

            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = repo.list_mitigations(None, None, 50, 0).await;
            }
            start.elapsed()
        })
    });

    c.bench_function("db_count_active", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let pool = db::init_memory_pool().await.unwrap();
            let repo = db::Repository::from_sqlite(pool);

            // Insert mitigations
            for i in 0..100 {
                let m = make_mitigation(i);
                repo.insert_mitigation(&m).await.unwrap();
            }

            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = repo.count_active_global().await;
            }
            start.elapsed()
        })
    });

    c.bench_function("db_is_safelisted", |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            let pool = db::init_memory_pool().await.unwrap();
            let repo = db::Repository::from_sqlite(pool);

            // Add some safelist entries
            for i in 0..10 {
                repo.insert_safelist(&format!("10.0.{}.0/24", i), "bench", None)
                    .await
                    .unwrap();
            }

            let start = std::time::Instant::now();
            for i in 0..iters {
                let ip = format!("10.0.{}.1", i % 20);
                let _ = repo.is_safelisted(&ip).await;
            }
            start.elapsed()
        })
    });
}

// Benchmark: Mitigation serialization
fn bench_serialization(c: &mut Criterion) {
    let mitigation = make_mitigation(0);

    c.bench_function("mitigation_serialize_json", |b| {
        b.iter(|| black_box(serde_json::to_string(&mitigation)))
    });

    let json = serde_json::to_string(&mitigation).unwrap();
    c.bench_function("mitigation_deserialize_json", |b| {
        b.iter(|| black_box(serde_json::from_str::<Mitigation>(&json)))
    });
}

// Benchmark: MatchCriteria operations
fn bench_match_criteria(c: &mut Criterion) {
    let criteria1 = MatchCriteria {
        dst_prefix: "203.0.1.10/32".to_string(),
        protocol: Some(17),
        dst_ports: vec![53, 80, 443, 8080],
    };

    let criteria2 = MatchCriteria {
        dst_prefix: "203.0.1.10/32".to_string(),
        protocol: Some(17),
        dst_ports: vec![53, 443],
    };

    c.bench_function("match_criteria_clone", |b| {
        b.iter(|| black_box(criteria1.clone()))
    });

    c.bench_function("match_criteria_hash_4_ports", |b| {
        b.iter(|| black_box(criteria1.compute_scope_hash()))
    });

    c.bench_function("match_criteria_hash_2_ports", |b| {
        b.iter(|| black_box(criteria2.compute_scope_hash()))
    });
}

// Benchmark: UUID generation
fn bench_uuid(c: &mut Criterion) {
    c.bench_function("uuid_v4_generate", |b| {
        b.iter(|| black_box(uuid::Uuid::new_v4()))
    });

    let id = uuid::Uuid::new_v4();
    c.bench_function("uuid_to_string", |b| {
        b.iter(|| black_box(id.to_string()))
    });
}

// Group benchmarks with different sample sizes for DB operations
fn bench_db_scaling(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("db_list_scaling");

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("list_mitigations", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter_custom(|iters| async move {
                    let pool = db::init_memory_pool().await.unwrap();
                    let repo = db::Repository::from_sqlite(pool);

                    for i in 0..size {
                        let m = make_mitigation(i);
                        repo.insert_mitigation(&m).await.unwrap();
                    }

                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        let _ = repo.list_mitigations(None, None, 50, 0).await;
                    }
                    start.elapsed()
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Inventory scaling
fn bench_inventory_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("inventory_scaling");

    for num_customers in [10, 50, 100, 500].iter() {
        let mut customers = Vec::new();
        for i in 0..*num_customers {
            customers.push(Customer {
                customer_id: format!("cust_{}", i),
                name: format!("Customer {}", i),
                prefixes: vec![format!("203.{}.0.0/16", i % 256)],
                policy_profile: prefixd::config::PolicyProfile::Normal,
                services: vec![],
            });
        }
        let inventory = Inventory::new(customers);

        group.bench_with_input(
            BenchmarkId::new("lookup", num_customers),
            num_customers,
            |b, _| {
                b.iter(|| black_box(inventory.lookup_ip("203.50.1.1")))
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_inventory_lookup,
    bench_scope_hash,
    bench_database_operations,
    bench_serialization,
    bench_match_criteria,
    bench_uuid,
    bench_db_scaling,
    bench_inventory_scaling,
);

criterion_main!(benches);
