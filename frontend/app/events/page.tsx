import { Suspense } from "react"
import { EventsContentLive } from "@/components/dashboard/events-content-live"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { RefreshCw } from "lucide-react"

function LoadingState() {
  return (
    <div className="bg-card border border-border rounded-lg p-8 text-center">
      <RefreshCw className="h-8 w-8 mx-auto mb-2 animate-spin text-muted-foreground" />
      <p className="text-muted-foreground">Loading events...</p>
    </div>
  )
}

export default function EventsPage() {
  return (
    <DashboardLayout>
      <Suspense fallback={<LoadingState />}>
        <EventsContentLive />
      </Suspense>
    </DashboardLayout>
  )
}
