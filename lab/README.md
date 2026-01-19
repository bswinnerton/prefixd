# prefixd Lab Environment

Containerlab topologies for testing FlowSpec with routers.

## Lab Options

| Lab | Router | Virtualization | Notes |
|-----|--------|----------------|-------|
| `frr-flowspec.clab.yml` | FRR | None (native container) | **Recommended** - works everywhere |
| `vjunos-flowspec.clab.yml` | vJunos-router | Nested KVM (Intel only) | Requires Intel CPU with VMX |
| `cjunos-flowspec.clab.yml` | cJunosEvolved | Nested KVM (Intel only) | Requires Intel CPU with VMX |

## FRR Lab (Recommended)

FRR (Free Range Routing) runs natively in containers without nested virtualization. Works on any Linux host.

### Quick Start

```bash
# Start prefixd stack first
cd /path/to/prefixd
docker compose up -d

# Deploy FRR lab
cd lab
sudo containerlab deploy -t frr-flowspec.clab.yml

# Connect prefixd-gobgp to lab network (if not already connected)
docker network connect clab-mgmt prefixd-gobgp 2>/dev/null || true

# Add FRR as GoBGP neighbor
docker exec prefixd-gobgp gobgp neighbor add 172.30.30.3 as 65002
docker exec prefixd-gobgp gobgp neighbor 172.30.30.3 afi-safi add ipv4-flowspec

# Verify BGP session
docker exec prefixd-gobgp gobgp neighbor

# Check FlowSpec in FRR
docker exec clab-frr-flowspec-router vtysh -c "show bgp ipv4 flowspec"
```

### Testing FlowSpec

```bash
# Inject a test event
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

# Verify FlowSpec received by FRR
docker exec clab-frr-flowspec-router vtysh -c "show bgp ipv4 flowspec"
docker exec clab-frr-flowspec-router vtysh -c "show bgp ipv4 flowspec detail"
```

### Cleanup

```bash
docker network disconnect clab-mgmt prefixd-gobgp
sudo containerlab destroy -t frr-flowspec.clab.yml
```

---

## Juniper Labs (Intel CPU Required)

> **Warning**: Juniper vJunos and cJunosEvolved require nested KVM virtualization with Intel VMX. They will not work on AMD processors due to the VMs checking specifically for VMX (not SVM).

### Prerequisites

1. **Intel CPU with nested virtualization enabled**:
   ```bash
   # Check if nested virt is enabled
   cat /sys/module/kvm_intel/parameters/nested
   # Should show "Y" or "1"
   
   # Enable if needed
   sudo modprobe -r kvm_intel
   sudo modprobe kvm_intel nested=1
   echo "options kvm_intel nested=1" | sudo tee /etc/modprobe.d/kvm-nested.conf
   ```

2. **Install containerlab**:
   ```bash
   bash -c "$(curl -sL https://get.containerlab.dev)"
   ```

3. **Download router images** from Juniper (free account required):
   - **vJunos-router**: https://support.juniper.net/support/downloads/?p=vjunos-router
   - **cJunosEvolved**: https://support.juniper.net/support/downloads/?p=cjunosevolved

4. **Build vrnetlab images**:
   ```bash
   git clone https://github.com/hellt/vrnetlab.git ~/vrnetlab
   
   # vJunos-router
   cp /path/to/vJunos-router-*.qcow2 ~/vrnetlab/juniper/vjunosrouter/
   cd ~/vrnetlab/juniper/vjunosrouter && make
   
   # cJunosEvolved
   cp /path/to/cjunosevolved-*.qcow2 ~/vrnetlab/juniper/vjunosevolved/
   cd ~/vrnetlab/juniper/vjunosevolved && make
   ```

### Topologies

#### vjunos-flowspec.clab.yml
Tests FlowSpec with vJunos-router (MX-based Junos). Matches most production deployments.

```
┌─────────────┐     eBGP      ┌─────────────┐
│   GoBGP     │◄─────────────►│  vJunos     │
│  (prefixd)  │  FlowSpec     │   Router    │
└─────────────┘               └─────────────┘
   AS 65001                      AS 65002
```

#### cjunos-flowspec.clab.yml  
Tests FlowSpec with cJunosEvolved (PTX-based Junos Evolved).

```
┌─────────────┐     eBGP      ┌─────────────┐
│   GoBGP     │◄─────────────►│  cJunos     │
│  (prefixd)  │  FlowSpec     │  Evolved    │
└─────────────┘               └─────────────┘
   AS 65001                      AS 65002
```

### Quick Start (Juniper)

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

### Resource Requirements

| Router | RAM | CPU | Boot Time |
|--------|-----|-----|-----------|
| vJunos-router | 5 GB | 4 cores | ~5-10 min |
| cJunosEvolved | 8 GB | 4 cores | ~15 min |

### Troubleshooting

#### "Nested VMX not enabled" error
You're on an AMD CPU. Use the FRR lab instead, or run on Intel hardware.

#### Router not accepting FlowSpec
Check import policy:
```
show configuration policy-options policy-statement FLOWSPEC-IMPORT
show configuration protocols bgp group GOBGP import
```

#### BGP session not establishing
Check reachability and config:
```
ping 10.0.0.1
show bgp neighbor
show configuration protocols bgp
```
