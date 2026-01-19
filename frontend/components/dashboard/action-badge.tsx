import { cn } from "@/lib/utils"

interface ActionBadgeProps {
  type: "police" | "discard"
  rate?: number
}

function formatBps(bps: number): string {
  if (bps >= 1_000_000_000) return `${(bps / 1_000_000_000).toFixed(1)} Gbps`
  if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(1)} Mbps`
  if (bps >= 1_000) return `${(bps / 1_000).toFixed(1)} Kbps`
  return `${bps} bps`
}

export function ActionBadge({ type, rate }: ActionBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center px-2 py-0.5 text-[10px] font-mono uppercase tracking-wide border",
        type === "police"
          ? "bg-primary/5 text-primary border-primary/30"
          : "bg-destructive/5 text-destructive border-destructive/30",
      )}
    >
      {type === "police" && rate ? `police ${formatBps(rate)}` : type}
    </span>
  )
}
