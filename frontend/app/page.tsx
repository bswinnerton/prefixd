import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { StatCard } from "@/components/dashboard/stat-card"
import { BgpSessionStatus } from "@/components/dashboard/bgp-session-status"
import { QuotaGauge } from "@/components/dashboard/quota-gauge"
import { ActivityFeed } from "@/components/dashboard/activity-feed"
import { mockMitigations } from "@/lib/mock-data"

export default function OverviewPage() {
  const activeMitigations = mockMitigations.filter((m) => m.status === "active" || m.status === "escalated")
  const policeActions = mockMitigations.filter(
    (m) => m.action.type === "police" && (m.status === "active" || m.status === "escalated"),
  )
  const discardActions = mockMitigations.filter(
    (m) => m.action.type === "discard" && (m.status === "active" || m.status === "escalated"),
  )

  const sparklineData = [3, 4, 5, 4, 6, 5, 7, 6, 7, 7, 6, 7]

  return (
    <DashboardLayout>
      <div className="space-y-4">
        <BgpSessionStatus />

        <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
          <StatCard title="Active Mitigations" value={activeMitigations.length} sparklineData={sparklineData} />
          <StatCard title="Police Actions" value={policeActions.length} accent="primary" />
          <StatCard title="Discard Actions" value={discardActions.length} accent="destructive" />
          <StatCard title="Events (24h)" value={42} trend="up" trendValue="+12%" />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
          <QuotaGauge
            title="Global Quota"
            current={7}
            max={500}
            secondary={{
              title: "Per-Customer Max",
              current: 3,
              max: 50,
            }}
          />
          <div className="lg:col-span-2">
            <ActivityFeed />
          </div>
        </div>
      </div>
    </DashboardLayout>
  )
}
