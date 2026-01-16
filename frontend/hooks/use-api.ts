"use client"

import useSWR from "swr"
import * as api from "@/lib/api"

const REFRESH_INTERVAL = 5000 // 5 seconds

export function useHealth() {
  return useSWR("health", api.getHealth, {
    refreshInterval: REFRESH_INTERVAL,
    revalidateOnFocus: true,
  })
}

export function useStats() {
  return useSWR("stats", api.getStats, {
    refreshInterval: REFRESH_INTERVAL,
    revalidateOnFocus: true,
  })
}

export function useMitigations(params?: Parameters<typeof api.getMitigations>[0]) {
  const key = params ? ["mitigations", JSON.stringify(params)] : "mitigations"
  return useSWR(key, () => api.getMitigations(params), {
    refreshInterval: REFRESH_INTERVAL,
    revalidateOnFocus: true,
  })
}

export function useMitigation(id: string | null) {
  return useSWR(id ? ["mitigation", id] : null, () => api.getMitigation(id!), {
    refreshInterval: REFRESH_INTERVAL,
  })
}

export function useSafelist() {
  return useSWR("safelist", api.getSafelist, {
    refreshInterval: REFRESH_INTERVAL,
  })
}

export function usePops() {
  return useSWR("pops", api.getPops, {
    refreshInterval: 30000, // 30 seconds for pops
  })
}

export function useEvents(params?: Parameters<typeof api.getEvents>[0]) {
  const key = params ? ["events", JSON.stringify(params)] : "events"
  return useSWR(key, () => api.getEvents(params), {
    refreshInterval: REFRESH_INTERVAL,
    revalidateOnFocus: true,
  })
}

// Parallel fetch all dashboard data in one request (async-parallel pattern)
export function useDashboard() {
  return useSWR("dashboard", api.getDashboardData, {
    refreshInterval: REFRESH_INTERVAL,
    revalidateOnFocus: true,
  })
}
