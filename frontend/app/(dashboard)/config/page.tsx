"use client"

import { useState } from "react"
import { useSWRConfig } from "swr"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { useConfigSettings, useConfigPlaybooks } from "@/hooks/use-api"
import { reloadConfig } from "@/lib/api"
import { usePermissions } from "@/hooks/use-permissions"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { RefreshCw, Loader2, FileCode, Zap } from "lucide-react"

function formatRate(bps: number): string {
  if (bps >= 1_000_000_000) return `${(bps / 1_000_000_000).toFixed(1)} Gbps`
  if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(0)} Mbps`
  if (bps >= 1_000) return `${(bps / 1_000).toFixed(0)} Kbps`
  return `${bps} bps`
}

function formatTtl(seconds: number): string {
  if (seconds >= 3600) return `${(seconds / 3600).toFixed(1)}h`
  if (seconds >= 60) return `${(seconds / 60).toFixed(0)}m`
  return `${seconds}s`
}

export default function ConfigPage() {
  const { data: settingsData, error: settingsError } = useConfigSettings()
  const { data: playbooksData, error: playbooksError } = useConfigPlaybooks()
  const { mutate } = useSWRConfig()
  const { canReloadConfig } = usePermissions()
  const [reloading, setReloading] = useState(false)
  const [reloadResult, setReloadResult] = useState<string | null>(null)

  const handleReload = async () => {
    setReloading(true)
    setReloadResult(null)
    try {
      await reloadConfig()
      await Promise.all([mutate("config-settings"), mutate("config-playbooks")])
      setReloadResult("Config reloaded successfully")
      setTimeout(() => setReloadResult(null), 5000)
    } catch (e) {
      setReloadResult(`Reload failed: ${e instanceof Error ? e.message : "unknown error"}`)
    } finally {
      setReloading(false)
    }
  }

  return (
    <DashboardLayout>
      <div className="flex-1 overflow-auto">
        <div className="p-4 sm:p-6 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-lg font-mono font-medium">Configuration</h1>
              <p className="text-xs text-muted-foreground font-mono mt-0.5">
                Running daemon configuration (read-only)
              </p>
            </div>
            <div className="flex items-center gap-3">
              {reloadResult && (
                <span className={`text-xs font-mono ${reloadResult.includes("failed") ? "text-destructive" : "text-success"}`}>
                  {reloadResult}
                </span>
              )}
              {canReloadConfig && (
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleReload}
                        disabled={reloading}
                        className="font-mono text-xs hover:text-foreground"
                      >
                        {reloading ? (
                          <Loader2 className="h-3 w-3 mr-1.5 animate-spin" />
                        ) : (
                          <RefreshCw className="h-3 w-3 mr-1.5" />
                        )}
                        Reload
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent className="font-mono text-xs">
                      Hot-reload inventory and playbooks from disk
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
            </div>
          </div>

          <Tabs defaultValue="settings">
            <TabsList className="font-mono">
              <TabsTrigger value="settings" className="text-xs">
                <FileCode className="h-3 w-3 mr-1.5" />
                Settings
              </TabsTrigger>
              <TabsTrigger value="playbooks" className="text-xs">
                <Zap className="h-3 w-3 mr-1.5" />
                Playbooks
                {playbooksData && (
                  <Badge variant="secondary" className="ml-1.5 text-[10px] px-1 py-0">
                    {playbooksData.total_playbooks}
                  </Badge>
                )}
              </TabsTrigger>
            </TabsList>

            <TabsContent value="settings" className="mt-4">
              {settingsError ? (
                <Card>
                  <CardContent className="p-4 text-sm text-destructive font-mono">
                    Failed to load settings: {settingsError.message}
                  </CardContent>
                </Card>
              ) : !settingsData ? (
                <Card>
                  <CardContent className="p-4 flex items-center gap-2 text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span className="text-sm font-mono">Loading settings...</span>
                  </CardContent>
                </Card>
              ) : (
                <Card>
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-sm font-mono">prefixd.yaml</CardTitle>
                      <span className="text-[10px] font-mono text-muted-foreground">
                        Loaded: {new Date(settingsData.loaded_at).toLocaleString()}
                      </span>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <pre className="text-xs font-mono bg-secondary/50 p-4 overflow-auto max-h-[600px] whitespace-pre-wrap">
                      {JSON.stringify(settingsData.settings, null, 2)}
                    </pre>
                  </CardContent>
                </Card>
              )}
            </TabsContent>

            <TabsContent value="playbooks" className="mt-4 space-y-3">
              {playbooksError ? (
                <Card>
                  <CardContent className="p-4 text-sm text-destructive font-mono">
                    Failed to load playbooks: {playbooksError.message}
                  </CardContent>
                </Card>
              ) : !playbooksData ? (
                <Card>
                  <CardContent className="p-4 flex items-center gap-2 text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span className="text-sm font-mono">Loading playbooks...</span>
                  </CardContent>
                </Card>
              ) : playbooksData.playbooks.length === 0 ? (
                <Card>
                  <CardContent className="p-4 text-sm text-muted-foreground font-mono">
                    No playbooks configured
                  </CardContent>
                </Card>
              ) : (
                playbooksData.playbooks.map((playbook) => (
                  <Card key={playbook.name}>
                    <CardHeader className="pb-2">
                      <div className="flex items-center gap-2">
                        <CardTitle className="text-sm font-mono">{playbook.name}</CardTitle>
                        <Badge variant="outline" className="text-[10px] font-mono">
                          {playbook.match.vector}
                        </Badge>
                        {playbook.match.require_top_ports && (
                          <Badge variant="secondary" className="text-[10px] font-mono">
                            top ports required
                          </Badge>
                        )}
                      </div>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-1.5">
                        {playbook.steps.map((step, i) => (
                          <div
                            key={i}
                            className="flex items-center gap-3 text-xs font-mono bg-secondary/50 px-3 py-2"
                          >
                            <span className="text-muted-foreground w-4">{i + 1}.</span>
                            <Badge
                              variant={step.action === "discard" ? "destructive" : "default"}
                              className="text-[10px] font-mono"
                            >
                              {step.action}
                            </Badge>
                            {step.rate_bps && (
                              <span className="text-muted-foreground">
                                {formatRate(step.rate_bps)}
                              </span>
                            )}
                            <span className="text-muted-foreground">
                              TTL {formatTtl(step.ttl_seconds)}
                            </span>
                            {step.require_confidence_at_least && (
                              <span className="text-muted-foreground">
                                confidence ≥ {step.require_confidence_at_least}
                              </span>
                            )}
                            {step.require_persistence_seconds && (
                              <span className="text-muted-foreground">
                                persist {formatTtl(step.require_persistence_seconds)}
                              </span>
                            )}
                          </div>
                        ))}
                      </div>
                    </CardContent>
                  </Card>
                ))
              )}
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </DashboardLayout>
  )
}
