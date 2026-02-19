import { RequireAuth } from "@/components/require-auth"
import { WebSocketProvider } from "@/components/websocket-provider"

export default function DashboardGroupLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <RequireAuth>
      <WebSocketProvider>
        {children}
      </WebSocketProvider>
    </RequireAuth>
  )
}
