# prefixd Lab Environment

Containerlab topologies for testing FlowSpec with real Juniper routers.

## Prerequisites

1. Install containerlab:
   ```bash
   bash -c "$(curl -sL https://get.containerlab.dev)"
   ```

2. Download router images from Juniper (free account required):
   - **vJunos-router**: https://support.juniper.net/support/downloads/?p=vjunos-router
   - **cJunosEvolved**: https://support.juniper.net/support/downloads/?p=cjunosevolved

3. Import images into containerlab:
   ```bash
   # vJunos-router (MX-based)
   cd /path/to/downloaded/images
   containerlab tools vrnetlab import vjunos-router-23.2R1.15.qcow2

   # cJunosEvolved (PTX-based) 
   containerlab tools vrnetlab import cjunosevolved-24.2R1.17.qcow2
   ```

## Topologies

### vjunos-flowspec.clab.yml
Tests FlowSpec with vJunos-router (MX-based Junos). This matches most production deployments.

```
┌─────────────┐     eBGP      ┌─────────────┐
│   GoBGP     │◄─────────────►│  vJunos     │
│  (prefixd)  │  FlowSpec     │   Router    │
└─────────────┘               └─────────────┘
   AS 65001                      AS 65002
```

### cjunos-flowspec.clab.yml  
Tests FlowSpec with cJunosEvolved (PTX-based Junos Evolved). Future-proof testing.

```
┌─────────────┐     eBGP      ┌─────────────┐
│   GoBGP     │◄─────────────►│  cJunos     │
│  (prefixd)  │  FlowSpec     │  Evolved    │
└─────────────┘               └─────────────┘
   AS 65001                      AS 65002
```

## Quick Start

```bash
# Start vJunos lab
cd /path/to/prefixd/lab
sudo containerlab deploy -t vjunos-flowspec.clab.yml

# Wait for router to boot (~5-10 min for vJunos, ~15 min for cJunos)
sudo containerlab inspect -t vjunos-flowspec.clab.yml

# SSH to router (default: admin/admin@123)
ssh admin@clab-vjunos-flowspec-router

# Verify BGP session
show bgp summary

# Check FlowSpec rules
show route table inetflow.0
show firewall filter __flowspec_default_inet__
```

## Testing FlowSpec

1. Start prefixd connected to the lab GoBGP:
   ```bash
   # Update configs/prefixd.yaml to point to lab GoBGP
   # bgp.gobgp_grpc: "172.20.20.2:50051"
   cargo run -- --config ./configs
   ```

2. Inject a test event:
   ```bash
   curl -X POST http://localhost:8080/v1/events \
     -H "Content-Type: application/json" \
     -d '{
       "timestamp": "2026-01-18T00:00:00Z",
       "source": "lab_test",
       "victim_ip": "203.0.113.10",
       "vector": "udp_flood",
       "bps": 1000000000,
       "pps": 1000000,
       "top_dst_ports": [53],
       "confidence": 0.9
     }'
   ```

3. Verify on router:
   ```bash
   # Check FlowSpec route received
   show route table inetflow.0

   # Check firewall filter created
   show firewall filter __flowspec_default_inet__

   # Check filter counters
   show firewall filter __flowspec_default_inet__ detail
   ```

## Cleanup

```bash
sudo containerlab destroy -t vjunos-flowspec.clab.yml
```

## Resource Requirements

| Router | RAM | CPU | Boot Time |
|--------|-----|-----|-----------|
| vJunos-router | 5 GB | 4 cores | ~5-10 min |
| cJunosEvolved | 8 GB | 4 cores | ~15 min |

## Troubleshooting

### Router not accepting FlowSpec
Check import policy:
```
show configuration policy-options policy-statement FLOWSPEC-IMPORT
show configuration protocols bgp group GOBGP import
```

### BGP session not establishing
Check reachability and config:
```
ping 172.20.20.2
show bgp neighbor 172.20.20.2
show configuration protocols bgp
```

### FlowSpec rules not installing
Check validation:
```
show route receive-protocol bgp 172.20.20.2 table inetflow.0 detail
show route validation-state inetflow.0
```
