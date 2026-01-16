import { cn } from "@/lib/utils"
import { formatBps } from "@/lib/mock-data"

interface ActionBadgeProps {
  type: "police" | "discard"
  rate?: number
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
      {type === "police" && rate ? `police ${formatBps(rate)}` : "discard"}
    </span>
  )
}
