import { cn } from "@/lib/utils"

interface ConfidenceBarProps {
  value: number
}

export function ConfidenceBar({ value }: ConfidenceBarProps) {
  const isHigh = value >= 80

  return (
    <div className="flex items-center gap-2">
      <div className="w-16 h-1 bg-secondary overflow-hidden">
        <div
          className={cn("h-full transition-all", isHigh ? "bg-primary" : "bg-muted-foreground")}
          style={{ width: `${value}%` }}
        />
      </div>
      <span className="text-[10px] font-mono text-muted-foreground tabular-nums">{value}%</span>
    </div>
  )
}
