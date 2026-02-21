#!/usr/bin/env bash
# Load testing for prefixd
# Requires: hey (sudo apt install hey), curl, jq, bc
#
# Usage:
#   ./scripts/load-test.sh              # run default suite (moderate)
#   ./scripts/load-test.sh quick        # fast smoke test (10s)
#   ./scripts/load-test.sh sustained    # sustained load (60s)
#   ./scripts/load-test.sh burst        # burst traffic (high concurrency)

set -euo pipefail

API="http://localhost"
PASS=0
FAIL=0
AUTH_ARGS=()

if [ -n "${PREFIXD_API_TOKEN:-}" ]; then
    AUTH_ARGS=(-H "Authorization: Bearer ${PREFIXD_API_TOKEN}")
fi

# --- helpers ---

log()  { printf "\033[1;34m[load]\033[0m %s\n" "$*"; }
pass() { printf "\033[1;32m  PASS\033[0m %s\n" "$*"; PASS=$((PASS + 1)); }
fail() { printf "\033[1;31m  FAIL\033[0m %s\n" "$*"; FAIL=$((FAIL + 1)); }

check_prereqs() {
    if ! command -v hey &>/dev/null; then
        echo "ERROR: hey not found. Install with: sudo apt install hey"
        exit 1
    fi
    if ! curl -sf "$API/v1/health" >/dev/null 2>&1; then
        echo "ERROR: prefixd not reachable at $API. Start docker compose first."
        exit 1
    fi
}

# Generate a unique event JSON body with a random IP
make_event_body() {
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    cat <<EOF
{"source":"load-test","timestamp":"$ts","victim_ip":"203.0.113.RAND","vector":"udp_flood","bps":1000000000,"pps":1000000,"top_dst_ports":[53],"confidence":0.95}
EOF
}

# Parse hey output for key metrics
_hey_rps=""
_hey_latency_avg=""
_hey_latency_p99=""

parse_hey() {
    local output="$1"

    _hey_rps=$(echo "$output" | awk '/Requests\/sec:/{print $2; exit}')
    _hey_latency_avg=$(echo "$output" | awk '/Average:/{print $2; exit}')
    _hey_latency_p99=$(echo "$output" | awk '/ 99% /{print $3; exit}')
    _hey_rps=${_hey_rps:-0}
    _hey_latency_avg=${_hey_latency_avg:-0}
    _hey_latency_p99=${_hey_latency_p99:-0}
    local status_dist
    status_dist=$(echo "$output" | grep "^\s*\[" | head -5)

    echo "  Requests/sec: $_hey_rps"
    echo "  Avg latency:  ${_hey_latency_avg}s"
    echo "  P99 latency:  ${_hey_latency_p99}s"
    if [ -n "$status_dist" ]; then
        echo "  Status codes:"
        echo "$status_dist" | sed 's/^/    /'
    fi
}

# --- test functions ---

# Health endpoint (baseline — should be very fast)
test_health_throughput() {
    local duration=${1:-10}
    local concurrency=${2:-10}

    log "Health endpoint: ${concurrency}c x ${duration}s"
    local output
    output=$(hey -z "${duration}s" -c "$concurrency" "${AUTH_ARGS[@]}" "$API/v1/health" 2>&1)
    parse_hey "$output"

    if [ "$(echo "$_hey_rps > 100" | bc -l 2>/dev/null)" = "1" ]; then
        pass "Health endpoint: ${_hey_rps} req/s (target: >100)"
    else
        fail "Health endpoint too slow: ${_hey_rps} req/s (target: >100)"
    fi
}

# Read endpoint (mitigations list)
test_read_throughput() {
    local duration=${1:-10}
    local concurrency=${2:-10}

    log "GET /v1/mitigations: ${concurrency}c x ${duration}s"
    local output
    output=$(hey -z "${duration}s" -c "$concurrency" "${AUTH_ARGS[@]}" "$API/v1/mitigations" 2>&1)
    parse_hey "$output"

    if [ "$(echo "$_hey_rps > 50" | bc -l 2>/dev/null)" = "1" ]; then
        pass "Mitigations list: ${_hey_rps} req/s (target: >50)"
    else
        fail "Mitigations list too slow: ${_hey_rps} req/s (target: >50)"
    fi
}

# Event ingestion (write path — the critical one)
test_ingestion_throughput() {
    local duration=${1:-10}
    local concurrency=${2:-5}

    log "POST /v1/events (ingestion): ${concurrency}c x ${duration}s"

    # Create a temp body file — hey doesn't support dynamic bodies,
    # so all events hit the same IP (will extend existing mitigation)
    local body_file
    body_file=$(mktemp /tmp/prefixd-load-XXXXXX.json)
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "{\"source\":\"load-test\",\"timestamp\":\"$ts\",\"victim_ip\":\"198.51.100.1\",\"vector\":\"udp_flood\",\"bps\":1000000000,\"pps\":1000000,\"top_dst_ports\":[53],\"confidence\":0.95}" > "$body_file"

    local output
    output=$(hey -z "${duration}s" -c "$concurrency" \
        -m POST \
        -H "Content-Type: application/json" \
        "${AUTH_ARGS[@]}" \
        -D "$body_file" \
        "$API/v1/events" 2>&1)

    rm -f "$body_file"
    parse_hey "$output"

    if [ "$(echo "$_hey_rps > 10" | bc -l 2>/dev/null)" = "1" ]; then
        pass "Event ingestion: ${_hey_rps} req/s (target: >10)"
    else
        fail "Event ingestion too slow: ${_hey_rps} req/s (target: >10)"
    fi

    if [ -n "$_hey_latency_p99" ] && [ "$(echo "$_hey_latency_p99 < 1.0" | bc -l 2>/dev/null)" = "1" ]; then
        pass "Ingestion P99 latency: ${_hey_latency_p99}s (target: <1s)"
    else
        fail "Ingestion P99 latency too high: ${_hey_latency_p99}s (target: <1s)"
    fi
}

# Burst: high concurrency spike
test_burst() {
    local requests=${1:-500}
    local concurrency=${2:-50}

    log "Burst: ${requests} requests at ${concurrency} concurrency"

    local body_file
    body_file=$(mktemp /tmp/prefixd-load-XXXXXX.json)
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "{\"source\":\"load-test\",\"timestamp\":\"$ts\",\"victim_ip\":\"198.51.100.2\",\"vector\":\"syn_flood\",\"bps\":5000000000,\"pps\":5000000,\"top_dst_ports\":[80,443],\"confidence\":0.98}" > "$body_file"

    local output
    output=$(hey -n "$requests" -c "$concurrency" \
        -m POST \
        -H "Content-Type: application/json" \
        "${AUTH_ARGS[@]}" \
        -D "$body_file" \
        "$API/v1/events" 2>&1)

    rm -f "$body_file"
    parse_hey "$output"

    # Check for 5xx errors in output
    local server_errors
    server_errors=$(echo "$output" | grep -oP '\[5\d\d\]\s+\K\d+' || echo "0")
    if [ "$server_errors" = "0" ]; then
        pass "Burst: no 5xx errors at ${concurrency} concurrency"
    else
        fail "Burst: ${server_errors} server errors at ${concurrency} concurrency"
    fi

    if [ "$(echo "$_hey_rps > 5" | bc -l 2>/dev/null)" = "1" ]; then
        pass "Burst throughput: ${_hey_rps} req/s"
    else
        fail "Burst throughput too low: ${_hey_rps} req/s"
    fi
}

# Metrics endpoint under load (should not block)
test_metrics_under_load() {
    local duration=${1:-10}

    log "Metrics endpoint during ingestion load: ${duration}s"

    # Start background ingestion
    local body_file
    body_file=$(mktemp /tmp/prefixd-load-XXXXXX.json)
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "{\"source\":\"load-test\",\"timestamp\":\"$ts\",\"victim_ip\":\"198.51.100.3\",\"vector\":\"udp_flood\",\"bps\":1000000000,\"pps\":1000000,\"top_dst_ports\":[53],\"confidence\":0.9}" > "$body_file"

    hey -z "${duration}s" -c 5 -m POST \
        -H "Content-Type: application/json" \
        "${AUTH_ARGS[@]}" \
        -D "$body_file" \
        "$API/v1/events" >/dev/null 2>&1 &
    local bg_pid=$!

    sleep 2

    # Hit metrics while ingestion is running
    local output
    output=$(hey -z "$((duration - 3))s" -c 2 "${AUTH_ARGS[@]}" "$API/metrics" 2>&1)

    wait "$bg_pid" 2>/dev/null || true
    rm -f "$body_file"
    parse_hey "$output"

    if [ -n "$_hey_latency_p99" ] && [ "$(echo "$_hey_latency_p99 < 0.5" | bc -l 2>/dev/null)" = "1" ]; then
        pass "Metrics P99 under load: ${_hey_latency_p99}s (target: <0.5s)"
    else
        fail "Metrics too slow under load: ${_hey_latency_p99}s (target: <0.5s)"
    fi
}

# Sustained load (longer duration)
test_sustained() {
    local duration=${1:-60}
    local concurrency=${2:-10}

    log "Sustained ingestion: ${concurrency}c x ${duration}s"

    local body_file
    body_file=$(mktemp /tmp/prefixd-load-XXXXXX.json)
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "{\"source\":\"load-test\",\"timestamp\":\"$ts\",\"victim_ip\":\"198.51.100.4\",\"vector\":\"udp_flood\",\"bps\":2000000000,\"pps\":2000000,\"top_dst_ports\":[53,123],\"confidence\":0.92}" > "$body_file"

    local output
    output=$(hey -z "${duration}s" -c "$concurrency" \
        -m POST \
        -H "Content-Type: application/json" \
        "${AUTH_ARGS[@]}" \
        -D "$body_file" \
        "$API/v1/events" 2>&1)

    rm -f "$body_file"
    parse_hey "$output"

    # After sustained load, check health
    sleep 2
    if curl -sf "$API/v1/health" >/dev/null 2>&1; then
        pass "API healthy after ${duration}s sustained load"
    else
        fail "API unhealthy after sustained load"
    fi

    if [ "$(echo "$_hey_rps > 10" | bc -l 2>/dev/null)" = "1" ]; then
        pass "Sustained throughput: ${_hey_rps} req/s over ${duration}s"
    else
        fail "Sustained throughput degraded: ${_hey_rps} req/s over ${duration}s"
    fi

    # Check DB pool isn't exhausted
    local pool_active
    pool_active=$(curl -sf "$API/metrics" 2>/dev/null | grep 'prefixd_db_pool_connections.*active' | grep -v '^#' | awk '{print $2}' || echo "0")
    log "DB pool active connections after load: $pool_active"
}

# --- profiles ---

run_quick() {
    log "=== Quick Smoke Test (10s per test) ==="
    echo ""
    test_health_throughput 5 5
    test_read_throughput 5 5
    test_ingestion_throughput 5 3
    echo ""
}

run_default() {
    log "=== Default Load Test Suite ==="
    echo ""
    test_health_throughput 10 10
    test_read_throughput 10 10
    test_ingestion_throughput 15 5
    test_burst 500 50
    test_metrics_under_load 10
    echo ""
}

run_sustained() {
    log "=== Sustained Load Test (60s) ==="
    echo ""
    test_sustained 60 10
    echo ""
}

run_burst() {
    log "=== Burst Test ==="
    echo ""
    test_burst 200 20
    test_burst 500 50
    test_burst 1000 100
    echo ""
}

# --- main ---

check_prereqs

# Capture starting stats
events_before=$(curl -sf "${AUTH_ARGS[@]}" "$API/v1/stats" 2>/dev/null | jq -r '.total_events // 0')
log "Starting stats: $events_before events in database"
echo ""

profile="${1:-default}"

case "$profile" in
    quick)     run_quick ;;
    default)   run_default ;;
    sustained) run_sustained ;;
    burst)     run_burst ;;
    all)
        run_quick
        run_default
        run_burst
        run_sustained
        ;;
    *)
        echo "Usage: $0 [quick|default|sustained|burst|all]"
        exit 1
        ;;
esac

# Final stats
events_after=$(curl -sf "${AUTH_ARGS[@]}" "$API/v1/stats" 2>/dev/null | jq -r '.total_events // 0')
events_ingested=$((events_after - events_before))
log "Events ingested during test: $events_ingested ($events_before -> $events_after)"

echo ""
echo "=============================="
echo " Load Test Results"
echo "=============================="
echo " PASS: $PASS"
echo " FAIL: $FAIL"
echo "=============================="

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
