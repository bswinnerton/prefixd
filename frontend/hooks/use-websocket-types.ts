export type WsMessageType =
  | "MitigationCreated"
  | "MitigationUpdated"
  | "MitigationExpired"
  | "MitigationWithdrawn"
  | "EventIngested"
  | "ResyncRequired"

export interface WsMessage {
  type: WsMessageType
  // MitigationCreated/Updated
  mitigation?: {
    mitigation_id: string
    status: string
    customer_id: string | null
    victim_ip: string
    vector: string
    action_type: string
    rate_bps: number | null
    created_at: string
    expires_at: string
    scope_hash: string
  }
  // MitigationExpired/Withdrawn
  mitigation_id?: string
  // EventIngested
  event?: {
    event_id: string
    external_event_id: string | null
    victim_ip: string
    vector: string
    confidence: number | null
    source: string
    ingested_at: string
  }
  // ResyncRequired
  reason?: string
}

export type ConnectionState = "connecting" | "connected" | "disconnected" | "error"
