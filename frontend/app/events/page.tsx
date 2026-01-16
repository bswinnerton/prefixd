import { Suspense } from "react"
import { EventsContent } from "@/components/dashboard/events-content"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"

export default function EventsPage() {
  return (
    <DashboardLayout>
      <Suspense fallback={null}>
        <EventsContent />
      </Suspense>
    </DashboardLayout>
  )
}
