"use client"

import { cn } from "@/lib/utils"

type Severity = "critical" | "high" | "medium" | "low"

interface SeverityBadgeProps {
  severity: Severity
  size?: "sm" | "default"
}

function deriveSeverity(status: string, actionType: string): Severity {
  if (status === "escalated") return "critical"
  if (actionType === "discard") return "high"
  if (status === "active" && actionType === "police") return "medium"
  return "low"
}

const COLORS: Record<Severity, string> = {
  critical: "border-destructive/50 text-destructive bg-destructive/5",
  high: "border-orange-500/50 text-orange-600 bg-orange-500/5 dark:text-orange-400",
  medium: "border-warning/50 text-warning bg-warning/5",
  low: "border-border text-muted-foreground bg-muted/50",
}

const DOT_COLORS: Record<Severity, string> = {
  critical: "bg-destructive",
  high: "bg-orange-500",
  medium: "bg-warning",
  low: "bg-muted-foreground",
}

export function SeverityBadge({ severity, size = "default" }: SeverityBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center border font-mono uppercase tracking-wide",
        size === "sm" ? "px-1.5 py-0.5 text-[9px]" : "px-2 py-0.5 text-[10px]",
        COLORS[severity],
      )}
    >
      <span className={cn("mr-1 size-1.5", DOT_COLORS[severity])} />
      {severity}
    </span>
  )
}

export { deriveSeverity, type Severity }
