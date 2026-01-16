import { cn } from "@/lib/utils"

interface BgpPeer {
  name: string
  status: "established" | "idle" | "connect" | "active"
  uptime?: string
}

const peers: BgpPeer[] = [
  { name: "jnpr-edge-1", status: "established", uptime: "14d 6h 32m" },
  { name: "jnpr-edge-2", status: "established", uptime: "14d 6h 30m" },
]

export function BgpSessionStatus() {
  return (
    <div className="border border-border bg-card p-4">
      <h3 className="text-xs font-mono uppercase tracking-wide text-muted-foreground mb-3">BGP Sessions</h3>
      <div className="flex flex-wrap gap-2">
        {peers.map((peer) => (
          <div
            key={peer.name}
            className="group relative flex items-center gap-2 bg-secondary px-3 py-2 border border-border"
          >
            <span
              className={cn(
                "h-1.5 w-1.5",
                peer.status === "established"
                  ? "bg-primary"
                  : peer.status === "idle"
                    ? "bg-muted-foreground"
                    : "bg-warning",
              )}
            />
            <span className="font-mono text-xs text-foreground">{peer.name}</span>
            <span
              className={cn(
                "text-[10px] font-mono uppercase",
                peer.status === "established"
                  ? "text-primary"
                  : peer.status === "idle"
                    ? "text-muted-foreground"
                    : "text-warning",
              )}
            >
              {peer.status}
            </span>
            {peer.uptime && (
              <div className="absolute -top-8 left-1/2 -translate-x-1/2 hidden group-hover:block bg-popover text-popover-foreground text-[10px] font-mono px-2 py-1 border border-border shadow-lg whitespace-nowrap z-10">
                uptime: {peer.uptime}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
