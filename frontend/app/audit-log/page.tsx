import { Suspense } from "react"
import { AuditLogContent } from "@/components/dashboard/audit-log-content"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"

export default function AuditLogPage() {
  return (
    <DashboardLayout>
      <Suspense fallback={null}>
        <AuditLogContent />
      </Suspense>
    </DashboardLayout>
  )
}
