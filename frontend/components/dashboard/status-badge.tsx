import { cn } from "@/lib/utils"

interface StatusBadgeProps {
  status: "active" | "escalated" | "expired" | "withdrawn" | "announced" | "pending" | "failed"
  size?: "sm" | "default"
}

export function StatusBadge({ status, size = "default" }: StatusBadgeProps) {
  const isPositive = status === "active" || status === "announced"
  const isNegative = status === "escalated" || status === "failed"
  const isPending = status === "pending"
  const isInactive = status === "expired" || status === "withdrawn"

  return (
    <span
      className={cn(
        "inline-flex items-center border font-mono uppercase tracking-wide",
        size === "sm" ? "px-1.5 py-0.5 text-[10px]" : "px-2 py-1 text-[10px]",
        isPositive && "border-primary/50 text-primary bg-primary/5",
        isNegative && "border-destructive/50 text-destructive bg-destructive/5",
        isPending && "border-warning/50 text-warning bg-warning/5",
        isInactive && "border-border text-muted-foreground bg-muted/50",
      )}
    >
      <span
        className={cn(
          "mr-1.5 h-1.5 w-1.5",
          isPositive && "bg-primary",
          isNegative && "bg-destructive",
          isPending && "bg-warning",
          isInactive && "bg-muted-foreground",
        )}
      />
      {status}
    </span>
  )
}
