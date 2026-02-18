"use client"

import { useEffect } from "react"
import { useRouter } from "next/navigation"
import { useAuth } from "@/hooks/use-auth"
import { usePermissions } from "@/hooks/use-permissions"
import { Loader2 } from "lucide-react"

interface RequireAuthProps {
  children: React.ReactNode
}

export function RequireAuth({ children }: RequireAuthProps) {
  const { isAuthenticated, isLoading } = useAuth()
  const { settled, authDisabled } = usePermissions()
  const router = useRouter()

  useEffect(() => {
    // Wait until both auth and health have resolved before redirecting
    if (settled && !authDisabled && !isAuthenticated) {
      router.push("/login")
    }
  }, [settled, authDisabled, isAuthenticated, router])

  // Still resolving auth or health state
  if (!settled || isLoading) {
    return (
      <div className="min-h-dvh flex items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // Auth disabled (auth: none) -- allow through
  if (authDisabled) {
    return <>{children}</>
  }

  if (!isAuthenticated) {
    return null
  }

  return <>{children}</>
}
