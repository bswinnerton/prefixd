"use client"

import { useAuth } from "./use-auth"

export function usePermissions() {
  const { operator } = useAuth()

  const role = operator?.role

  return {
    // Role checks
    isAdmin: role === "admin",
    isOperator: role === "operator" || role === "admin",
    isViewer: role === "viewer" || role === "operator" || role === "admin",

    // Permission checks
    canWithdraw: role === "admin" || role === "operator",
    canManageSafelist: role === "admin",
    canManageUsers: role === "admin",
    canReloadConfig: role === "admin",

    // Current role (for display)
    role,
  }
}
