import { RequireAuth } from "@/components/require-auth"

export default function DashboardGroupLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return <RequireAuth>{children}</RequireAuth>
}
