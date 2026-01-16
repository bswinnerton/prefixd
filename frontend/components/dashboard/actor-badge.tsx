import { cn } from "@/lib/utils"
import { Bot, User } from "lucide-react"

interface ActorBadgeProps {
  type: "system" | "operator"
  name: string
}

export function ActorBadge({ type, name }: ActorBadgeProps) {
  return (
    <div className="flex items-center gap-2">
      <span
        className={cn(
          "flex items-center justify-center h-5 w-5",
          type === "system" ? "bg-secondary text-muted-foreground" : "bg-primary/10 text-primary",
        )}
      >
        {type === "system" ? <Bot className="h-3 w-3" /> : <User className="h-3 w-3" />}
      </span>
      <span className="text-xs font-mono text-foreground">{name}</span>
    </div>
  )
}
