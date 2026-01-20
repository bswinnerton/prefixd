# FastNetMon Community Integration

This guide covers integrating FastNetMon Community Edition with prefixd for automated FlowSpec-based DDoS mitigation.

## Overview

FastNetMon detects DDoS attacks via NetFlow/sFlow analysis and calls a notify script when attacks are detected. The `prefixd-fastnetmon.sh` script bridges FastNetMon to prefixd's event API.

```
FastNetMon → notify_script → prefixd → GoBGP → Routers (FlowSpec)
```

## Prerequisites

- FastNetMon Community Edition installed and configured
- prefixd running with API accessible
- `curl` and optionally `jq` installed on the FastNetMon host
- Network connectivity from FastNetMon to prefixd API

## Installation

1. Copy the notify script to the FastNetMon host:

```bash
sudo cp scripts/prefixd-fastnetmon.sh /usr/local/bin/
sudo chmod +x /usr/local/bin/prefixd-fastnetmon.sh
```

2. Configure environment variables in `/etc/default/prefixd`:

```bash
# prefixd API endpoint
PREFIXD_API="http://prefixd-host:8080"

# Bearer token for authentication (if auth.mode=bearer)
PREFIXD_TOKEN="your-api-token"

# Log file location (optional)
PREFIXD_LOG="/var/log/prefixd-fastnetmon.log"
```

3. Configure FastNetMon to use the script. Edit `/etc/fastnetmon.conf`:

```ini
notify_script_path = /usr/local/bin/prefixd-fastnetmon.sh
notify_script_pass_details = on
```

4. Restart FastNetMon:

```bash
sudo systemctl restart fastnetmon
```

## How It Works

### Ban Flow

1. FastNetMon detects attack on victim IP
2. Calls script with args: `$1=IP $2=direction $3=pps $4=ban`
3. Script computes stable `event_id` from `IP|direction`
4. Sends `POST /v1/events` with `action: "ban"`
5. prefixd evaluates playbook, creates mitigation, announces FlowSpec

### Unban Flow

1. FastNetMon attack subsides
2. Calls script with args: `$1=IP $2=direction $3=pps $4=unban`
3. Script sends same `event_id` with `action: "unban"`
4. prefixd finds original event, withdraws mitigation

### Idempotency

The `event_id` is computed as `sha256(IP|direction)`, ensuring:
- Duplicate bans are rejected (409 Conflict)
- Unbans match their corresponding bans
- Safe retries if network issues occur

## Vector Detection

The script infers attack vectors from FastNetMon's stdin details:

| FastNetMon Detail | prefixd Vector |
|------------------|----------------|
| Contains "udp"   | `udp_flood`    |
| Contains "syn"   | `syn_flood`    |
| Contains "ack"   | `ack_flood`    |
| Contains "icmp"  | `icmp_flood`   |
| Other            | `unknown`      |

## Testing

Test the script manually:

```bash
# Simulate ban
echo "Attack details: UDP flood" | /usr/local/bin/prefixd-fastnetmon.sh 192.0.2.1 incoming 1000000 ban

# Check prefixd
curl http://prefixd:8080/v1/mitigations?status=active

# Simulate unban
echo "" | /usr/local/bin/prefixd-fastnetmon.sh 192.0.2.1 incoming 0 unban
```

## Troubleshooting

### Check logs

```bash
tail -f /var/log/prefixd-fastnetmon.log
```

### Common issues

1. **Connection refused**: Verify `PREFIXD_API` is reachable
2. **401 Unauthorized**: Check `PREFIXD_TOKEN` matches prefixd config
3. **422 Unprocessable**: Victim IP not in prefixd inventory
4. **No mitigation created**: Check playbooks match the vector

### Debug mode

Add `set -x` at the top of the script for verbose output.

## Advanced: Custom Vector Mapping

Edit the script's vector detection section to add custom mappings:

```bash
# Add DNS amplification detection
if [[ "$RAW_LOWER" == *"port 53"* ]] && [[ "$RAW_LOWER" == *"udp"* ]]; then
    VECTOR="dns_amplification"
fi
```

Note: Custom vectors require matching playbook rules in prefixd.

## Security Considerations

1. **Token security**: Store `PREFIXD_TOKEN` in `/etc/default/prefixd` with restricted permissions:
   ```bash
   sudo chmod 600 /etc/default/prefixd
   ```

2. **Network security**: Use TLS for production deployments (configure `PREFIXD_API=https://...`)

3. **Log rotation**: Configure logrotate for `/var/log/prefixd-fastnetmon.log`
