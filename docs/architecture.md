# Architecture

This document describes prefixd's design decisions and data flow.

## Overview

prefixd is a **policy daemon** that sits between attack detectors and BGP routers. It doesn't detect attacks or filter packets—it makes policy decisions about when and how to announce FlowSpec rules.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              prefixd                                        │
│                                                                             │
│  ┌──────────┐    ┌──────────────┐    ┌────────────┐    ┌──────────────┐   │
│  │  HTTP    │───▶│   Policy     │───▶│ Guardrails │───▶│  FlowSpec    │   │
│  │  API     │    │   Engine     │    │            │    │  Manager     │   │
│  └──────────┘    └──────────────┘    └────────────┘    └──────┬───────┘   │
│       │                │                   │                   │           │
│       │                │                   │                   │           │
│       ▼                ▼                   ▼                   ▼           │
│  ┌──────────┐    ┌──────────────┐    ┌────────────┐    ┌──────────────┐   │
│  │ Events   │    │  Playbooks   │    │  Safelist  │    │   GoBGP      │   │
│  │   DB     │    │   (YAML)     │    │    DB      │    │   gRPC       │   │
│  └──────────┘    └──────────────┘    └────────────┘    └──────────────┘   │
│                                                                             │
│                        ┌──────────────────┐                                │
│                        │  Reconciliation  │                                │
│                        │      Loop        │                                │
│                        └──────────────────┘                                │
└─────────────────────────────────────────────────────────────────────────────┘
         │                                                        │
         │                                                        │
         ▼                                                        ▼
┌─────────────────┐                                    ┌─────────────────┐
│    Detectors    │                                    │     Routers     │
│   (FastNetMon,  │                                    │   (Juniper,     │
│    Prometheus)  │                                    │    Arista)      │
└─────────────────┘                                    └─────────────────┘
```

## Design Principles

### 1. Signal-Driven, Not Detection

prefixd doesn't detect attacks. Detectors (FastNetMon, Prometheus alerts, custom scripts) signal attack events, and prefixd applies policy.

**Why?** Detection is hard and domain-specific. Rate-based detection misses low-and-slow attacks; ML-based detection has false positives. By separating detection from policy, you can:

- Use the best detector for each attack type
- Tune detection thresholds independently
- Correlate multiple weak signals (future: v1.5)

### 2. Fail-Open

If prefixd crashes or becomes unavailable:

- **Existing mitigations continue** until their TTL expires
- **No new mitigations** are created (attacks may go unblocked)
- **No permanent rules** are left behind

This is intentional. The alternative (fail-closed with permanent rules) risks blocking legitimate traffic indefinitely if prefixd dies.

### 3. /32 Only (IPv4)

prefixd enforces that mitigations target single IPs (/32 for IPv4, /128 for IPv6). Broader prefixes are rejected.

**Why?** A misconfigured detector sending a /24 could block 256 IPs. With /32-only, the blast radius of a false positive is one IP.

### 4. Mandatory TTL

Every mitigation must have a TTL. There are no permanent rules.

**Why?** Without TTL, a false positive requires manual intervention. With TTL, false positives auto-resolve when the mitigation expires.

### 5. Guardrails Everywhere

Every mitigation passes through guardrails:

- Safelist check (is this IP protected?)
- Prefix length check (/32 only)
- TTL bounds check (min/max)
- Port count check (max 8)
- Quota check (per-customer, per-POP, global)

Guardrails can't be bypassed. They're the last line of defense against bad data.

---

## Data Flow

### Event Ingestion

```
POST /v1/events
     │
     ▼
┌─────────────────┐
│ Parse & Validate│───▶ 400 Bad Request
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Rate Limit Check│───▶ 429 Too Many Requests
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Safelist Check  │───▶ 403 Forbidden (safelisted)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Inventory Lookup│───▶ Find customer/service context
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Playbook Match  │───▶ Find policy for this vector
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Guardrails      │───▶ 422 Validation Failed
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Create/Extend   │───▶ Duplicate? Extend TTL
│ Mitigation      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Announce via    │───▶ GoBGP gRPC AddPath
│ GoBGP           │
└────────┬────────┘
         │
         ▼
     201 Created
```

### Reconciliation Loop

Runs every 30 seconds:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Reconciliation Loop                         │
│                                                                 │
│  1. Load desired state (active mitigations from DB)             │
│                         │                                       │
│                         ▼                                       │
│  2. Load actual state (FlowSpec routes from GoBGP RIB)          │
│                         │                                       │
│                         ▼                                       │
│  3. Find expired mitigations (TTL passed)                       │
│     └──▶ Withdraw from GoBGP                                   │
│     └──▶ Mark as "expired" in DB                               │
│                         │                                       │
│                         ▼                                       │
│  4. Find missing rules (in DB, not in RIB)                      │
│     └──▶ Re-announce via GoBGP                                 │
│                         │                                       │
│                         ▼                                       │
│  5. Find orphan rules (in RIB, not in DB)                       │
│     └──▶ Withdraw from GoBGP                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Why reconciliation?**

- GoBGP might restart, losing RIB state
- prefixd might restart, missing the window to withdraw
- Network issues might cause announcement failures
- Manual GoBGP CLI operations might create inconsistency

Reconciliation ensures desired state eventually matches actual state.

---

## Component Details

### Policy Engine

The policy engine evaluates playbooks to determine mitigation actions.

```yaml
# Playbook example
playbooks:
  - name: udp_flood
    match:
      vector: udp_flood
    steps:
      - action: police
        rate_bps: 10000000
        ttl_seconds: 120
      - action: discard
        ttl_seconds: 300
        require_confidence_at_least: 0.8
```

**Evaluation order:**

1. Find playbook matching event vector
2. Check escalation eligibility (is there an existing mitigation?)
3. Select step based on confidence threshold
4. Apply port exclusions from inventory

### Guardrails

Guardrails are non-negotiable safety checks:

| Check | Rejection Reason |
|-------|------------------|
| `require_ttl` | TTL is mandatory |
| `min_ttl_seconds` | TTL below minimum |
| `max_ttl_seconds` | TTL above maximum |
| `dst_prefix_maxlen` | Prefix broader than /32 |
| `max_ports` | Too many ports (router memory) |
| `max_active_per_customer` | Customer quota exceeded |
| `max_active_per_pop` | POP quota exceeded |
| `max_active_global` | Global quota exceeded |
| Safelist | IP is protected |

### FlowSpec Manager

Translates mitigations into FlowSpec NLRI:

```
Mitigation                    FlowSpec NLRI
─────────────────────────────────────────────────
dst_prefix: 203.0.113.10/32   Type 1: Destination Prefix
protocol: UDP                 Type 3: IP Protocol
dst_ports: [!53]              Type 5: Destination Port (negated)
action: police 10M            Extended Community: traffic-rate
```

### GoBGP Client

gRPC client for GoBGP v4.x:

- **AddPath** - Announce FlowSpec rule
- **DeletePath** - Withdraw FlowSpec rule
- **ListPath** - Query RIB for reconciliation
- **ListPeer** - Monitor BGP session status

Features:
- Connection pooling
- Automatic retry with exponential backoff
- Timeout handling (10s connect, 30s request)

---

## State Management

### PostgreSQL Schema

```sql
-- Mitigations
CREATE TABLE mitigations (
    id UUID PRIMARY KEY,
    status VARCHAR(20),        -- pending, active, expired, withdrawn
    customer_id VARCHAR(255),
    service_id VARCHAR(255),
    dst_prefix INET,
    protocol VARCHAR(10),
    dst_ports INTEGER[],
    action VARCHAR(20),        -- police, discard
    rate_bps BIGINT,
    ttl_seconds INTEGER,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ,
    pop VARCHAR(50)
);

-- Events
CREATE TABLE events (
    id UUID PRIMARY KEY,
    source VARCHAR(255),
    victim_ip INET,
    vector VARCHAR(50),
    bps BIGINT,
    pps BIGINT,
    confidence REAL,
    created_at TIMESTAMPTZ
);

-- Safelist
CREATE TABLE safelist (
    prefix INET PRIMARY KEY,
    reason TEXT,
    created_by VARCHAR(255),
    created_at TIMESTAMPTZ
);

-- Operators (auth)
CREATE TABLE operators (
    id UUID PRIMARY KEY,
    username VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255),
    role VARCHAR(20),
    created_at TIMESTAMPTZ
);

-- Sessions (auth)
CREATE TABLE sessions (
    id VARCHAR(255) PRIMARY KEY,
    data BYTEA,
    expiry_date TIMESTAMPTZ
);
```

### Multi-POP Model

Multiple prefixd instances share one PostgreSQL:

```
prefixd (iad1)  ──┐
prefixd (fra1)  ──┼──▶ PostgreSQL
prefixd (sin1)  ──┘
```

Each instance:
- Filters by its own `pop` field
- Announces to its local GoBGP
- Has cross-POP visibility via `?pop=all`

---

## Security Model

### Authentication Layers

1. **Network level** - API should be on private network or behind load balancer
2. **Bearer token** - For API/CLI access
3. **Session cookie** - For dashboard access
4. **mTLS** - Optional mutual TLS for zero-trust environments

### Trust Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│ Trusted Zone                                                    │
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐                 │
│  │ prefixd  │◀──▶│ PostgreSQL│    │  GoBGP   │                 │
│  └──────────┘    └──────────┘    └──────────┘                 │
│       ▲                               │                        │
│       │                               ▼                        │
│       │                         ┌──────────┐                   │
│       │                         │  Routers │                   │
│       │                         └──────────┘                   │
└───────┼─────────────────────────────────────────────────────────┘
        │
        │ HTTPS + Auth
        │
┌───────┼─────────────────────────────────────────────────────────┐
│ Untrusted Zone                                                  │
│       │                                                         │
│       ▼                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐                 │
│  │Detectors │    │Dashboard │    │   CLI    │                 │
│  └──────────┘    └──────────┘    └──────────┘                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key principle:** Detectors are untrusted. They can suggest mitigations, but guardrails have final say.

---

## Performance Characteristics

### Latency

| Operation | Typical | P99 |
|-----------|---------|-----|
| Event ingestion | <5ms | <20ms |
| Inventory lookup | <1μs | <10μs |
| DB query | <2ms | <10ms |
| GoBGP announce | ~10ms | ~50ms |

### Throughput

| Operation | Sustained | Burst |
|-----------|-----------|-------|
| Event ingestion | 6,000/s | 10,000/s |
| DB writes | 2,000/s | 5,000/s |
| GoBGP announces | 100/s | 200/s |

### Bottleneck: GoBGP

GoBGP gRPC is the bottleneck for high-volume events. Each announcement requires:
- gRPC round-trip (~10ms)
- BGP UPDATE construction
- RIB insertion

For 1000+ mitigations/second, consider batching (not yet implemented).

---

## Failure Modes

### prefixd Crash

- Existing mitigations continue (TTL-based expiry)
- No new mitigations until restart
- Reconciliation syncs state on restart

### PostgreSQL Unavailable

- Event ingestion fails (503)
- Reconciliation pauses
- GoBGP rules remain (no withdrawal)

### GoBGP Unavailable

- Event ingestion fails at announcement step
- Mitigation saved to DB as "pending"
- Reconciliation retries on GoBGP recovery

### Router Crash

- GoBGP session drops
- Router recovers, re-establishes BGP
- GoBGP re-advertises routes
- No prefixd intervention required

---

## Future Architecture

### v1.5: Multi-Signal Correlation

```
FastNetMon  ──┐
Prometheus  ──┼──▶ Correlation ──▶ Policy Engine
Router CPU  ──┘    Engine
```

Combine weak signals into high-confidence decisions.

### v2.0: Native BGP Speaker

Replace GoBGP sidecar with embedded BGP:

```
prefixd ──── BGP ────▶ Routers
  │
  └── No GoBGP dependency
```

Reduces operational complexity but increases code complexity.

---

## Why These Choices?

### Why GoBGP sidecar instead of native BGP?

- **Time to market** - GoBGP is battle-tested
- **Flexibility** - GoBGP supports features we might need later
- **Debugging** - gobgp CLI is useful for troubleshooting
- **Risk isolation** - BGP bugs don't crash prefixd

### Why PostgreSQL instead of embedded DB?

- **Multi-POP** - Shared state across instances
- **Operational familiarity** - Teams know Postgres
- **Tooling** - pgAdmin, backups, monitoring
- **HA** - Postgres replication is well-understood

### Why Rust?

- **Performance** - Minimal latency in hot path
- **Safety** - No null pointers, no data races
- **Ecosystem** - Great async runtime (tokio), gRPC (tonic)
- **Binary size** - Single ~15MB binary, no runtime deps
