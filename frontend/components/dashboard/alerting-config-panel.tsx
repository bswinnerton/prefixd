"use client"

import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Loader2, Bell, Send, CheckCircle2, XCircle, AlertCircle } from "lucide-react"
import { useAlertingConfig } from "@/hooks/use-api"
import { testAlerting, type AlertingTestResult } from "@/lib/api"
import { cn } from "@/lib/utils"

const DEST_ICONS: Record<string, string> = {
  slack: "#",
  discord: "D",
  teams: "T",
  telegram: "TG",
  pagerduty: "PD",
  opsgenie: "OG",
  generic: "W",
}

const DEST_COLORS: Record<string, string> = {
  slack: "bg-[#4A154B] text-white",
  discord: "bg-[#5865F2] text-white",
  teams: "bg-[#6264A7] text-white",
  telegram: "bg-[#0088CC] text-white",
  pagerduty: "bg-[#06AC38] text-white",
  opsgenie: "bg-[#2684FF] text-white",
  generic: "bg-secondary text-foreground",
}

function destSummary(d: Record<string, unknown>): string {
  const type = d.type as string
  if (type === "slack" && d.channel) return `${d.channel}`
  if (type === "telegram" && d.chat_id) return `chat ${d.chat_id}`
  if (type === "pagerduty" && d.events_url) return `${d.events_url}`
  if (type === "opsgenie" && d.region) return `${(d.region as string).toUpperCase()} region`
  if (type === "generic" && d.url) return `${d.url}`
  return ""
}

export function AlertingConfigPanel() {
  const { data, error, isLoading } = useAlertingConfig()
  const [testing, setTesting] = useState(false)
  const [testResults, setTestResults] = useState<AlertingTestResult[] | null>(null)
  const [testError, setTestError] = useState<string | null>(null)

  const handleTest = async () => {
    setTesting(true)
    setTestResults(null)
    setTestError(null)
    try {
      const response = await testAlerting()
      setTestResults(response.results)
    } catch (e) {
      setTestError(e instanceof Error ? e.message : "Test failed")
    } finally {
      setTesting(false)
    }
  }

  if (isLoading) {
    return (
      <Card>
        <CardContent className="p-4 flex items-center gap-2 text-muted-foreground">
          <Loader2 className="h-4 w-4 animate-spin" />
          <span className="text-sm font-mono">Loading alerting config...</span>
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardContent className="p-4 text-sm text-destructive font-mono">
          Failed to load alerting config: {error.message}
        </CardContent>
      </Card>
    )
  }

  if (!data || data.destinations.length === 0) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-mono">Alert Destinations</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-6 text-center">
            <Bell className="h-8 w-8 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground font-mono">No alert destinations configured</p>
            <p className="text-xs text-muted-foreground mt-1">
              Add destinations to the <code className="bg-secondary px-1 py-0.5">alerting</code> section in prefixd.yaml
            </p>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-3">
      <Card>
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-mono">Alert Destinations</CardTitle>
            <Button
              variant="outline"
              size="sm"
              onClick={handleTest}
              disabled={testing}
              className="font-mono text-xs"
            >
              {testing ? (
                <Loader2 className="h-3 w-3 mr-1.5 animate-spin" />
              ) : (
                <Send className="h-3 w-3 mr-1.5" />
              )}
              Send Test Alert
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-2">
          {data.destinations.map((dest, i) => {
            const testResult = testResults?.find(r => r.destination === dest.type)
            const summary = destSummary(dest as unknown as Record<string, unknown>)
            return (
              <div
                key={i}
                className="flex items-center gap-3 bg-secondary/50 px-3 py-2.5"
              >
                <span className={cn(
                  "shrink-0 flex items-center justify-center h-6 w-8 text-[10px] font-bold font-mono",
                  DEST_COLORS[dest.type] || DEST_COLORS.generic,
                )}>
                  {DEST_ICONS[dest.type] || "?"}
                </span>
                <div className="flex-1 min-w-0">
                  <span className="text-xs font-mono font-medium capitalize">{dest.type}</span>
                  {summary ? (
                    <span className="text-xs font-mono text-muted-foreground ml-2">
                      {summary}
                    </span>
                  ) : null}
                </div>
                {testResult && (
                  testResult.status === "ok" ? (
                    <CheckCircle2 className="h-4 w-4 text-success shrink-0" />
                  ) : (
                    <div className="flex items-center gap-1 shrink-0">
                      <XCircle className="h-4 w-4 text-destructive" />
                      {testResult.error && (
                        <span className="text-[10px] text-destructive font-mono max-w-[150px] truncate">
                          {testResult.error}
                        </span>
                      )}
                    </div>
                  )
                )}
              </div>
            )
          })}
        </CardContent>
      </Card>

      {data.events.length > 0 && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-mono">Event Filters</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-1.5">
              {data.events.map((event) => (
                <Badge key={event} variant="outline" className="text-[10px] font-mono">
                  {event}
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {testError && (
        <div className="flex items-center gap-2 p-3 bg-destructive/10 border border-destructive/50 text-sm">
          <AlertCircle className="h-4 w-4 text-destructive shrink-0" />
          <span className="text-destructive font-mono text-xs">{testError}</span>
        </div>
      )}
    </div>
  )
}
