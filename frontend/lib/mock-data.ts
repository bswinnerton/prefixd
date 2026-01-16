// Mock data for the dashboard

export interface Mitigation {
  id: string
  status: "active" | "escalated" | "expired" | "withdrawn"
  victimIp: string
  vector: string
  action: {
    type: "police" | "discard"
    rate?: number
  }
  customer: string
  service: string
  createdAt: Date
  expiresAt: Date
  scopeHash: string
  triggeringEventId: string
  match: {
    destination: string
    protocol: string
    ports: string
  }
  announcements: {
    peer: string
    status: "announced" | "pending" | "failed"
    announcedAt: Date
    latency: number
  }[]
}

export interface Event {
  id: string
  timestamp: Date
  source: "fastnetmon" | "noc" | "custom"
  victimIp: string
  vector: string
  trafficBps: number
  trafficPps: number
  confidence: number
  resultingMitigation: string | "rejected" | "deduplicated"
}

export interface AuditEntry {
  id: string
  timestamp: Date
  actor: { type: "system" | "operator"; name: string }
  action: "announce" | "withdraw" | "reject" | "escalate" | "extend" | "create"
  target: string
  details: string
}

export interface ActivityItem {
  id: string
  timestamp: Date
  type: "mitigation" | "event" | "operator"
  description: string
  ip?: string
}

// Generate mock mitigations
export const mockMitigations: Mitigation[] = [
  {
    id: "mit-001-abc123",
    status: "active",
    victimIp: "203.0.113.10",
    vector: "udp_flood",
    action: { type: "police", rate: 5000000 },
    customer: "ACME Corp",
    service: "web-prod",
    createdAt: new Date(Date.now() - 2 * 60 * 1000),
    expiresAt: new Date(Date.now() + 3 * 60 * 1000),
    scopeHash: "a1b2c3d4e5f6",
    triggeringEventId: "evt-001",
    match: {
      destination: "203.0.113.10/32",
      protocol: "UDP",
      ports: "53",
    },
    announcements: [
      { peer: "jnpr-edge-1", status: "announced", announcedAt: new Date(Date.now() - 2 * 60 * 1000), latency: 12 },
      { peer: "jnpr-edge-2", status: "announced", announcedAt: new Date(Date.now() - 2 * 60 * 1000), latency: 8 },
    ],
  },
  {
    id: "mit-002-def456",
    status: "active",
    victimIp: "198.51.100.25",
    vector: "syn_flood",
    action: { type: "discard" },
    customer: "TechStart Inc",
    service: "api-gateway",
    createdAt: new Date(Date.now() - 5 * 60 * 1000),
    expiresAt: new Date(Date.now() + 10 * 60 * 1000),
    scopeHash: "f6e5d4c3b2a1",
    triggeringEventId: "evt-002",
    match: {
      destination: "198.51.100.25/32",
      protocol: "TCP",
      ports: "80, 443",
    },
    announcements: [
      { peer: "jnpr-edge-1", status: "announced", announcedAt: new Date(Date.now() - 5 * 60 * 1000), latency: 15 },
      { peer: "jnpr-edge-2", status: "announced", announcedAt: new Date(Date.now() - 5 * 60 * 1000), latency: 11 },
    ],
  },
  {
    id: "mit-003-ghi789",
    status: "escalated",
    victimIp: "192.0.2.100",
    vector: "ntp_amplification",
    action: { type: "police", rate: 10000000 },
    customer: "GlobalNet",
    service: "dns-primary",
    createdAt: new Date(Date.now() - 15 * 60 * 1000),
    expiresAt: new Date(Date.now() + 45 * 60 * 1000),
    scopeHash: "123456789abc",
    triggeringEventId: "evt-003",
    match: {
      destination: "192.0.2.100/32",
      protocol: "UDP",
      ports: "123",
    },
    announcements: [
      { peer: "jnpr-edge-1", status: "announced", announcedAt: new Date(Date.now() - 15 * 60 * 1000), latency: 18 },
      { peer: "jnpr-edge-2", status: "announced", announcedAt: new Date(Date.now() - 15 * 60 * 1000), latency: 14 },
    ],
  },
  {
    id: "mit-004-jkl012",
    status: "active",
    victimIp: "10.0.50.75",
    vector: "dns_amplification",
    action: { type: "police", rate: 2000000 },
    customer: "SecureHost",
    service: "customer-portal",
    createdAt: new Date(Date.now() - 8 * 60 * 1000),
    expiresAt: new Date(Date.now() + 2 * 60 * 1000),
    scopeHash: "def789abc123",
    triggeringEventId: "evt-004",
    match: {
      destination: "10.0.50.75/32",
      protocol: "UDP",
      ports: "53",
    },
    announcements: [
      { peer: "jnpr-edge-1", status: "announced", announcedAt: new Date(Date.now() - 8 * 60 * 1000), latency: 10 },
      { peer: "jnpr-edge-2", status: "announced", announcedAt: new Date(Date.now() - 8 * 60 * 1000), latency: 9 },
    ],
  },
  {
    id: "mit-005-mno345",
    status: "expired",
    victimIp: "172.16.0.50",
    vector: "icmp_flood",
    action: { type: "discard" },
    customer: "DataFlow",
    service: "monitoring",
    createdAt: new Date(Date.now() - 60 * 60 * 1000),
    expiresAt: new Date(Date.now() - 30 * 60 * 1000),
    scopeHash: "456789abcdef",
    triggeringEventId: "evt-005",
    match: {
      destination: "172.16.0.50/32",
      protocol: "ICMP",
      ports: "N/A",
    },
    announcements: [
      { peer: "jnpr-edge-1", status: "announced", announcedAt: new Date(Date.now() - 60 * 60 * 1000), latency: 20 },
      { peer: "jnpr-edge-2", status: "announced", announcedAt: new Date(Date.now() - 60 * 60 * 1000), latency: 16 },
    ],
  },
  {
    id: "mit-006-pqr678",
    status: "active",
    victimIp: "203.0.113.50",
    vector: "memcached_amplification",
    action: { type: "police", rate: 50000000 },
    customer: "CloudScale",
    service: "cache-cluster",
    createdAt: new Date(Date.now() - 3 * 60 * 1000),
    expiresAt: new Date(Date.now() + 12 * 60 * 1000),
    scopeHash: "789abcdef012",
    triggeringEventId: "evt-006",
    match: {
      destination: "203.0.113.50/32",
      protocol: "UDP",
      ports: "11211",
    },
    announcements: [
      { peer: "jnpr-edge-1", status: "announced", announcedAt: new Date(Date.now() - 3 * 60 * 1000), latency: 7 },
      { peer: "jnpr-edge-2", status: "announced", announcedAt: new Date(Date.now() - 3 * 60 * 1000), latency: 6 },
    ],
  },
  {
    id: "mit-007-stu901",
    status: "active",
    victimIp: "198.51.100.100",
    vector: "ssdp_amplification",
    action: { type: "discard" },
    customer: "MediaStream",
    service: "video-cdn",
    createdAt: new Date(Date.now() - 1 * 60 * 1000),
    expiresAt: new Date(Date.now() + 4 * 60 * 1000),
    scopeHash: "abcdef012345",
    triggeringEventId: "evt-007",
    match: {
      destination: "198.51.100.100/32",
      protocol: "UDP",
      ports: "1900",
    },
    announcements: [
      { peer: "jnpr-edge-1", status: "announced", announcedAt: new Date(Date.now() - 1 * 60 * 1000), latency: 5 },
      { peer: "jnpr-edge-2", status: "announced", announcedAt: new Date(Date.now() - 1 * 60 * 1000), latency: 4 },
    ],
  },
]

export const mockEvents: Event[] = [
  {
    id: "evt-001",
    timestamp: new Date(Date.now() - 2 * 60 * 1000),
    source: "fastnetmon",
    victimIp: "203.0.113.10",
    vector: "udp_flood",
    trafficBps: 1200000000,
    trafficPps: 250000,
    confidence: 95,
    resultingMitigation: "mit-001-abc123",
  },
  {
    id: "evt-002",
    timestamp: new Date(Date.now() - 5 * 60 * 1000),
    source: "fastnetmon",
    victimIp: "198.51.100.25",
    vector: "syn_flood",
    trafficBps: 800000000,
    trafficPps: 180000,
    confidence: 92,
    resultingMitigation: "mit-002-def456",
  },
  {
    id: "evt-003",
    timestamp: new Date(Date.now() - 15 * 60 * 1000),
    source: "noc",
    victimIp: "192.0.2.100",
    vector: "ntp_amplification",
    trafficBps: 5000000000,
    trafficPps: 500000,
    confidence: 98,
    resultingMitigation: "mit-003-ghi789",
  },
  {
    id: "evt-004",
    timestamp: new Date(Date.now() - 8 * 60 * 1000),
    source: "fastnetmon",
    victimIp: "10.0.50.75",
    vector: "dns_amplification",
    trafficBps: 600000000,
    trafficPps: 120000,
    confidence: 88,
    resultingMitigation: "mit-004-jkl012",
  },
  {
    id: "evt-005",
    timestamp: new Date(Date.now() - 60 * 60 * 1000),
    source: "custom",
    victimIp: "172.16.0.50",
    vector: "icmp_flood",
    trafficBps: 200000000,
    trafficPps: 50000,
    confidence: 75,
    resultingMitigation: "mit-005-mno345",
  },
  {
    id: "evt-008",
    timestamp: new Date(Date.now() - 25 * 60 * 1000),
    source: "fastnetmon",
    victimIp: "10.0.100.200",
    vector: "udp_flood",
    trafficBps: 300000000,
    trafficPps: 80000,
    confidence: 65,
    resultingMitigation: "rejected",
  },
  {
    id: "evt-009",
    timestamp: new Date(Date.now() - 30 * 60 * 1000),
    source: "fastnetmon",
    victimIp: "203.0.113.10",
    vector: "udp_flood",
    trafficBps: 400000000,
    trafficPps: 90000,
    confidence: 82,
    resultingMitigation: "deduplicated",
  },
]

export const mockAuditLog: AuditEntry[] = [
  {
    id: "audit-001",
    timestamp: new Date(Date.now() - 2 * 60 * 1000),
    actor: { type: "system", name: "prefixd-daemon" },
    action: "announce",
    target: "mit-001-abc123",
    details: "FlowSpec rule announced to all BGP peers for 203.0.113.10 UDP flood mitigation",
  },
  {
    id: "audit-002",
    timestamp: new Date(Date.now() - 2 * 60 * 1000),
    actor: { type: "system", name: "prefixd-daemon" },
    action: "create",
    target: "mit-001-abc123",
    details: "Mitigation created from event evt-001, policy: police 5Mbps",
  },
  {
    id: "audit-003",
    timestamp: new Date(Date.now() - 5 * 60 * 1000),
    actor: { type: "system", name: "prefixd-daemon" },
    action: "announce",
    target: "mit-002-def456",
    details: "FlowSpec rule announced to all BGP peers for 198.51.100.25 SYN flood mitigation",
  },
  {
    id: "audit-004",
    timestamp: new Date(Date.now() - 15 * 60 * 1000),
    actor: { type: "operator", name: "admin@noc" },
    action: "escalate",
    target: "mit-003-ghi789",
    details: "Mitigation escalated due to sustained attack volume exceeding threshold",
  },
  {
    id: "audit-005",
    timestamp: new Date(Date.now() - 25 * 60 * 1000),
    actor: { type: "system", name: "prefixd-daemon" },
    action: "reject",
    target: "evt-008",
    details: "Event rejected: confidence score 65% below threshold of 70%",
  },
  {
    id: "audit-006",
    timestamp: new Date(Date.now() - 30 * 60 * 1000),
    actor: { type: "system", name: "prefixd-daemon" },
    action: "reject",
    target: "evt-009",
    details: "Event deduplicated: active mitigation exists for same victim/vector combination",
  },
  {
    id: "audit-007",
    timestamp: new Date(Date.now() - 45 * 60 * 1000),
    actor: { type: "operator", name: "ops@noc" },
    action: "extend",
    target: "mit-003-ghi789",
    details: "Mitigation TTL extended by 30 minutes",
  },
  {
    id: "audit-008",
    timestamp: new Date(Date.now() - 90 * 60 * 1000),
    actor: { type: "system", name: "prefixd-daemon" },
    action: "withdraw",
    target: "mit-005-mno345",
    details: "Mitigation expired and withdrawn from all BGP peers",
  },
]

export const mockActivity: ActivityItem[] = [
  {
    id: "act-001",
    timestamp: new Date(Date.now() - 2 * 60 * 1000),
    type: "mitigation",
    description: "Mitigation created · UDP flood → police 5Mbps",
    ip: "203.0.113.10",
  },
  {
    id: "act-002",
    timestamp: new Date(Date.now() - 5 * 60 * 1000),
    type: "mitigation",
    description: "Mitigation created · SYN flood → discard",
    ip: "198.51.100.25",
  },
  {
    id: "act-003",
    timestamp: new Date(Date.now() - 15 * 60 * 1000),
    type: "operator",
    description: "Mitigation escalated by admin@noc",
    ip: "192.0.2.100",
  },
  {
    id: "act-004",
    timestamp: new Date(Date.now() - 25 * 60 * 1000),
    type: "event",
    description: "Event rejected · Low confidence (65%)",
    ip: "10.0.100.200",
  },
  {
    id: "act-005",
    timestamp: new Date(Date.now() - 30 * 60 * 1000),
    type: "event",
    description: "Event deduplicated · Active mitigation exists",
    ip: "203.0.113.10",
  },
  {
    id: "act-006",
    timestamp: new Date(Date.now() - 45 * 60 * 1000),
    type: "operator",
    description: "Mitigation extended by ops@noc (+30m)",
    ip: "192.0.2.100",
  },
  {
    id: "act-007",
    timestamp: new Date(Date.now() - 60 * 60 * 1000),
    type: "mitigation",
    description: "Mitigation created · ICMP flood → discard",
    ip: "172.16.0.50",
  },
  {
    id: "act-008",
    timestamp: new Date(Date.now() - 90 * 60 * 1000),
    type: "mitigation",
    description: "Mitigation withdrawn · TTL expired",
    ip: "172.16.0.50",
  },
  {
    id: "act-009",
    timestamp: new Date(Date.now() - 3 * 60 * 1000),
    type: "mitigation",
    description: "Mitigation created · Memcached amplification → police 50Mbps",
    ip: "203.0.113.50",
  },
  {
    id: "act-010",
    timestamp: new Date(Date.now() - 1 * 60 * 1000),
    type: "mitigation",
    description: "Mitigation created · SSDP amplification → discard",
    ip: "198.51.100.100",
  },
]

// Helper functions
export function formatBps(bps: number): string {
  if (bps >= 1000000000) {
    return `${(bps / 1000000000).toFixed(1)} Gbps`
  }
  if (bps >= 1000000) {
    return `${(bps / 1000000).toFixed(1)} Mbps`
  }
  if (bps >= 1000) {
    return `${(bps / 1000).toFixed(1)} Kbps`
  }
  return `${bps} bps`
}

export function formatPps(pps: number): string {
  if (pps >= 1000000) {
    return `${(pps / 1000000).toFixed(1)}M pps`
  }
  if (pps >= 1000) {
    return `${(pps / 1000).toFixed(0)}K pps`
  }
  return `${pps} pps`
}

export function formatRelativeTime(date: Date): string {
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffSecs = Math.floor(diffMs / 1000)
  const diffMins = Math.floor(diffSecs / 60)
  const diffHours = Math.floor(diffMins / 60)

  if (diffSecs < 60) return `${diffSecs}s ago`
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  return date.toLocaleDateString()
}

export function formatTimeRemaining(date: Date): { text: string; isWarning: boolean } {
  const now = new Date()
  const diffMs = date.getTime() - now.getTime()

  if (diffMs <= 0) return { text: "Expired", isWarning: true }

  const diffSecs = Math.floor(diffMs / 1000)
  const diffMins = Math.floor(diffSecs / 60)
  const remainingSecs = diffSecs % 60

  if (diffMins < 1) {
    return { text: `in ${diffSecs}s`, isWarning: true }
  }
  if (diffMins < 2) {
    return { text: `in ${diffMins}m ${remainingSecs}s`, isWarning: true }
  }
  return { text: `in ${diffMins}m`, isWarning: false }
}

export function formatTimestamp(date: Date): string {
  return (
    date.toLocaleTimeString("en-US", {
      timeZone: "UTC",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    }) + " UTC"
  )
}
