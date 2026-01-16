import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Settings } from "lucide-react"

export default function ConfigPage() {
  return (
    <DashboardLayout>
      <div className="max-w-2xl mx-auto">
        <Card className="bg-card border-border">
          <CardHeader className="text-center">
            <div className="flex justify-center mb-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-secondary">
                <Settings className="h-6 w-6 text-muted-foreground" />
              </div>
            </div>
            <CardTitle className="text-foreground">Configuration</CardTitle>
            <CardDescription className="text-muted-foreground">
              Configuration options will be available in a future release.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-center text-sm text-muted-foreground">
              This section will include BGP peer configuration, quota management, policy rules, and notification
              settings.
            </p>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  )
}
