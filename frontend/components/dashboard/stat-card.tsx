import Link from "next/link"
import { cn } from "@/lib/utils"

interface StatCardProps {
  title: string
  value: string | number
  trend?: "up" | "down" | "neutral"
  trendValue?: string
  accent?: "default" | "primary" | "destructive" | "warning"
  sparklineData?: number[]
  href?: string
}

export function StatCard({ title, value, trend, trendValue, accent = "default", sparklineData, href }: StatCardProps) {
  const Wrapper = href ? Link : "div"
  const wrapperProps = href ? { href } : {}
  return (
    <Wrapper {...wrapperProps as any} className={cn("border border-border bg-card p-4 block", href && "hover:border-primary/50 transition-colors cursor-pointer")}>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs font-mono uppercase tracking-wide text-muted-foreground">{title}</p>
          <p
            className={cn(
              "text-2xl font-mono mt-1 tabular-nums",
              accent === "primary" && "text-primary",
              accent === "destructive" && "text-destructive",
              accent === "warning" && "text-warning",
              accent === "default" && "text-foreground",
            )}
          >
            {value}
          </p>
          {trend && trendValue && (
            <div className="flex items-center gap-1 mt-1">
              <span
                className={cn(
                  "text-xs font-mono",
                  trend === "up" && "text-primary",
                  trend === "down" && "text-destructive",
                  trend === "neutral" && "text-muted-foreground",
                )}
              >
                {trend === "up" && "↑"}
                {trend === "down" && "↓"}
                {trend === "neutral" && "→"}
              </span>
              <span className="text-xs text-muted-foreground font-mono">{trendValue}</span>
            </div>
          )}
        </div>
        {sparklineData && sparklineData.length > 0 && (
          <div className="h-8 w-16">
            <Sparkline data={sparklineData} />
          </div>
        )}
      </div>
    </Wrapper>
  )
}

function Sparkline({ data }: { data: number[] }) {
  const max = Math.max(...data)
  const min = Math.min(...data)
  const range = max - min || 1
  const width = 64
  const height = 32
  const padding = 2

  const points = data
    .map((value, index) => {
      const x = padding + (index / (data.length - 1)) * (width - 2 * padding)
      const y = height - padding - ((value - min) / range) * (height - 2 * padding)
      return `${x},${y}`
    })
    .join(" ")

  return (
    <svg width={width} height={height} className="text-primary opacity-60">
      <polyline fill="none" stroke="currentColor" strokeWidth="1" points={points} />
    </svg>
  )
}
