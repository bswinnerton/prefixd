import { Suspense } from "react"
import { MitigationsContent } from "@/components/dashboard/mitigations-content"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"

export default function MitigationsPage() {
  return (
    <DashboardLayout>
      <Suspense fallback={null}>
        <MitigationsContent />
      </Suspense>
    </DashboardLayout>
  )
}
