import { Shield, AlertTriangle, User } from "lucide-react"
import { cn } from "@/lib/utils"
import { mockActivity, formatTimestamp } from "@/lib/mock-data"

export function ActivityFeed() {
  const activities = mockActivity.slice(0, 10)

  return (
    <div className="border border-border bg-card p-4 h-full">
      <h3 className="text-xs font-mono uppercase tracking-wide text-muted-foreground mb-3">Recent Activity</h3>
      <div className="space-y-0">
        {activities.map((activity, index) => (
          <div
            key={activity.id}
            className={cn(
              "flex items-start gap-3 py-2 transition-colors hover:bg-secondary/50",
              index !== activities.length - 1 && "border-b border-border/50",
            )}
          >
            <div className="mt-0.5 opacity-60">
              {activity.type === "mitigation" && <Shield className="h-3 w-3" />}
              {activity.type === "event" && <AlertTriangle className="h-3 w-3" />}
              {activity.type === "operator" && <User className="h-3 w-3" />}
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 text-xs">
                <span className="font-mono text-[10px] text-muted-foreground whitespace-nowrap tabular-nums">
                  {formatTimestamp(activity.timestamp)}
                </span>
                <span className="text-foreground truncate">{activity.description}</span>
              </div>
              {activity.ip && <span className="font-mono text-[10px] text-primary">{activity.ip}</span>}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
