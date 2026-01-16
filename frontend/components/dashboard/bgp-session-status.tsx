"use client"

import { useHealth, usePops } from "@/hooks/use-api"
import { cn } from "@/lib/utils"

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h ${mins}m`
  if (hours > 0) return `${hours}h ${mins}m`
  return `${mins}m`
}

export function BgpSessionStatus() {
  const { data: health, error } = useHealth()
  const { data: pops } = usePops()

  const isUp = health?.bgp_session_up ?? false

  return (
    <div className="border border-border bg-card p-4">
      <h3 className="text-xs font-mono uppercase tracking-wide text-muted-foreground mb-3">System Status</h3>
      <div className="flex flex-wrap gap-2">
        <div className="group relative flex items-center gap-2 bg-secondary px-3 py-2 border border-border">
          <span className={cn("h-1.5 w-1.5", isUp ? "bg-primary" : "bg-destructive")} />
          <span className="font-mono text-xs text-foreground">BGP</span>
          <span className={cn("text-[10px] font-mono uppercase", isUp ? "text-primary" : "text-destructive")}>
            {isUp ? "UP" : error ? "ERROR" : "DOWN"}
          </span>
        </div>

        {health && (
          <div className="group relative flex items-center gap-2 bg-secondary px-3 py-2 border border-border">
            <span className="h-1.5 w-1.5 bg-primary" />
            <span className="font-mono text-xs text-foreground">{health.pop}</span>
            <span className="text-[10px] font-mono text-muted-foreground">
              {formatUptime(health.uptime_seconds)}
            </span>
          </div>
        )}

        {pops && pops.length > 1 && (
          <div className="flex items-center gap-2 bg-secondary px-3 py-2 border border-border">
            <span className="font-mono text-xs text-muted-foreground">
              {pops.length} POPs
            </span>
          </div>
        )}
      </div>
    </div>
  )
}
