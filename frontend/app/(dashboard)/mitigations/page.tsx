import { Suspense } from "react"
import { MitigationsContentLive } from "@/components/dashboard/mitigations-content-live"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { RefreshCw } from "lucide-react"

function LoadingState() {
  return (
    <div className="bg-card border border-border rounded-lg p-8 text-center">
      <RefreshCw className="h-8 w-8 mx-auto mb-2 animate-spin text-muted-foreground" />
      <p className="text-muted-foreground">Loading mitigations...</p>
    </div>
  )
}

export default function MitigationsPage() {
  return (
    <DashboardLayout>
      <Suspense fallback={<LoadingState />}>
        <MitigationsContentLive />
      </Suspense>
    </DashboardLayout>
  )
}
