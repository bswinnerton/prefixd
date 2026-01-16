"use client"

import { useState, useMemo } from "react"
import { Eye, Search, ChevronDown, ChevronUp, Filter } from "lucide-react"
import { StatusBadge } from "@/components/dashboard/status-badge"
import { ActionBadge } from "@/components/dashboard/action-badge"
import { MitigationDetailPanel } from "@/components/dashboard/mitigation-detail-panel"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { mockMitigations, type Mitigation, formatRelativeTime, formatTimeRemaining } from "@/lib/mock-data"
import { cn } from "@/lib/utils"

type SortField = "status" | "victimIp" | "vector" | "customer" | "createdAt" | "expiresAt"
type SortDirection = "asc" | "desc"

const vectors = [
  "All",
  "udp_flood",
  "syn_flood",
  "ntp_amplification",
  "dns_amplification",
  "icmp_flood",
  "memcached_amplification",
  "ssdp_amplification",
]
const customers = ["All", ...new Set(mockMitigations.map((m) => m.customer))]

export function MitigationsContent() {
  const [selectedMitigation, setSelectedMitigation] = useState<Mitigation | null>(null)
  const [statusFilters, setStatusFilters] = useState<string[]>(["active", "escalated"])
  const [vectorFilter, setVectorFilter] = useState("All")
  const [customerFilter, setCustomerFilter] = useState("All")
  const [searchQuery, setSearchQuery] = useState("")
  const [sortField, setSortField] = useState<SortField>("createdAt")
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc")
  const [currentPage, setCurrentPage] = useState(1)
  const [showFilters, setShowFilters] = useState(false)
  const itemsPerPage = 10

  const toggleStatusFilter = (status: string) => {
    setStatusFilters((prev) => (prev.includes(status) ? prev.filter((s) => s !== status) : [...prev, status]))
  }

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"))
    } else {
      setSortField(field)
      setSortDirection("desc")
    }
  }

  const filteredMitigations = useMemo(() => {
    return mockMitigations
      .filter((m) => {
        if (statusFilters.length > 0 && !statusFilters.includes(m.status)) return false
        if (vectorFilter !== "All" && m.vector !== vectorFilter) return false
        if (customerFilter !== "All" && m.customer !== customerFilter) return false
        if (searchQuery && !m.victimIp.includes(searchQuery)) return false
        return true
      })
      .sort((a, b) => {
        let comparison = 0
        switch (sortField) {
          case "status":
            comparison = a.status.localeCompare(b.status)
            break
          case "victimIp":
            comparison = a.victimIp.localeCompare(b.victimIp)
            break
          case "vector":
            comparison = a.vector.localeCompare(b.vector)
            break
          case "customer":
            comparison = a.customer.localeCompare(b.customer)
            break
          case "createdAt":
            comparison = a.createdAt.getTime() - b.createdAt.getTime()
            break
          case "expiresAt":
            comparison = a.expiresAt.getTime() - b.expiresAt.getTime()
            break
        }
        return sortDirection === "asc" ? comparison : -comparison
      })
  }, [statusFilters, vectorFilter, customerFilter, searchQuery, sortField, sortDirection])

  const totalPages = Math.ceil(filteredMitigations.length / itemsPerPage)
  const paginatedMitigations = filteredMitigations.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null
    return sortDirection === "asc" ? <ChevronUp className="h-3 w-3 ml-1" /> : <ChevronDown className="h-3 w-3 ml-1" />
  }

  const MobileCard = ({ mitigation }: { mitigation: Mitigation }) => {
    const expiresInfo = formatTimeRemaining(mitigation.expiresAt)
    return (
      <div
        className="p-4 bg-card border border-border rounded-lg space-y-3"
        onClick={() => setSelectedMitigation(mitigation)}
      >
        <div className="flex items-center justify-between">
          <StatusBadge status={mitigation.status} />
          <Button
            variant="ghost"
            size="icon"
            className="h-10 w-10"
            onClick={(e) => {
              e.stopPropagation()
              setSelectedMitigation(mitigation)
            }}
          >
            <Eye className="h-4 w-4" />
          </Button>
        </div>
        <div className="space-y-2">
          <div className="flex justify-between items-center">
            <span className="text-xs text-muted-foreground">Victim IP</span>
            <span className="font-mono text-sm text-foreground">{mitigation.victimIp}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-xs text-muted-foreground">Vector</span>
            <span className="text-sm text-muted-foreground">{mitigation.vector.replace(/_/g, " ")}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-xs text-muted-foreground">Action</span>
            <ActionBadge type={mitigation.action.type} rate={mitigation.action.rate} />
          </div>
          <div className="flex justify-between items-center">
            <span className="text-xs text-muted-foreground">Customer</span>
            <span className="text-sm text-foreground">{mitigation.customer}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-xs text-muted-foreground">Expires</span>
            <span className={cn("text-sm font-mono", expiresInfo.isWarning ? "text-warning" : "text-muted-foreground")}>
              {expiresInfo.text}
            </span>
          </div>
        </div>
      </div>
    )
  }

  return (
    <>
      <div className="space-y-4">
        <div className="bg-card border border-border rounded-lg overflow-hidden">
          {/* Search and filter toggle row */}
          <div className="flex items-center gap-2 p-3 sm:p-4">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search IP..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9 h-10 bg-secondary border-border text-base font-mono"
              />
            </div>
            <Button
              variant="outline"
              size="icon"
              className="h-10 w-10 lg:hidden shrink-0 bg-transparent"
              onClick={() => setShowFilters(!showFilters)}
            >
              <Filter className="h-4 w-4" />
            </Button>
          </div>

          <div
            className={cn(
              "border-t border-border p-3 sm:p-4 space-y-3",
              "lg:flex lg:flex-wrap lg:items-center lg:gap-3 lg:space-y-0",
              showFilters ? "block" : "hidden lg:flex",
            )}
          >
            {/* Status Pills */}
            <div className="flex flex-wrap items-center gap-2">
              {(["active", "escalated", "expired", "withdrawn"] as const).map((status) => (
                <button
                  key={status}
                  onClick={() => toggleStatusFilter(status)}
                  className={cn(
                    "px-3 py-2 rounded-full text-xs font-medium transition-colors capitalize min-h-[36px]",
                    statusFilters.includes(status)
                      ? status === "active"
                        ? "bg-success/20 text-success border border-success/50"
                        : status === "escalated"
                          ? "bg-destructive/20 text-destructive border border-destructive/50"
                          : "bg-secondary text-foreground border border-border"
                      : "bg-secondary/50 text-muted-foreground border border-transparent hover:border-border",
                  )}
                >
                  {status}
                </button>
              ))}
            </div>

            <div className="hidden lg:block h-6 w-px bg-border" />

            {/* Dropdown filters row */}
            <div className="flex flex-wrap gap-2">
              <Select value={vectorFilter} onValueChange={setVectorFilter}>
                <SelectTrigger className="w-full sm:w-40 h-10 bg-secondary border-border text-sm">
                  <SelectValue placeholder="Vector" />
                </SelectTrigger>
                <SelectContent>
                  {vectors.map((v) => (
                    <SelectItem key={v} value={v}>
                      {v === "All" ? "All Vectors" : v.replace(/_/g, " ")}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <Select value={customerFilter} onValueChange={setCustomerFilter}>
                <SelectTrigger className="w-full sm:w-40 h-10 bg-secondary border-border text-sm">
                  <SelectValue placeholder="Customer" />
                </SelectTrigger>
                <SelectContent>
                  {customers.map((c) => (
                    <SelectItem key={c} value={c}>
                      {c === "All" ? "All Customers" : c}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>

        <div className="lg:hidden space-y-3">
          {paginatedMitigations.map((mitigation) => (
            <MobileCard key={mitigation.id} mitigation={mitigation} />
          ))}
        </div>

        <div className="hidden lg:block bg-card border border-border rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="sticky top-0 bg-secondary">
                <tr className="border-b border-border">
                  <th
                    className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                    onClick={() => handleSort("status")}
                  >
                    <span className="flex items-center">
                      Status
                      <SortIcon field="status" />
                    </span>
                  </th>
                  <th
                    className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                    onClick={() => handleSort("victimIp")}
                  >
                    <span className="flex items-center">
                      Victim IP
                      <SortIcon field="victimIp" />
                    </span>
                  </th>
                  <th
                    className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                    onClick={() => handleSort("vector")}
                  >
                    <span className="flex items-center">
                      Vector
                      <SortIcon field="vector" />
                    </span>
                  </th>
                  <th className="text-left px-4 py-3 font-medium text-muted-foreground">Action</th>
                  <th
                    className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                    onClick={() => handleSort("customer")}
                  >
                    <span className="flex items-center">
                      Customer
                      <SortIcon field="customer" />
                    </span>
                  </th>
                  <th
                    className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                    onClick={() => handleSort("createdAt")}
                  >
                    <span className="flex items-center">
                      Created
                      <SortIcon field="createdAt" />
                    </span>
                  </th>
                  <th
                    className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                    onClick={() => handleSort("expiresAt")}
                  >
                    <span className="flex items-center">
                      Expires
                      <SortIcon field="expiresAt" />
                    </span>
                  </th>
                  <th className="text-center px-4 py-3 font-medium text-muted-foreground">Actions</th>
                </tr>
              </thead>
              <tbody>
                {paginatedMitigations.map((mitigation, index) => {
                  const expiresInfo = formatTimeRemaining(mitigation.expiresAt)
                  return (
                    <tr
                      key={mitigation.id}
                      className={cn(
                        "border-b border-border/50 hover:bg-secondary/50 transition-colors cursor-pointer",
                        index % 2 === 1 && "bg-secondary/20",
                      )}
                      onClick={() => setSelectedMitigation(mitigation)}
                    >
                      <td className="px-4 py-3">
                        <StatusBadge status={mitigation.status} />
                      </td>
                      <td className="px-4 py-3 font-mono text-foreground">{mitigation.victimIp}</td>
                      <td className="px-4 py-3 text-muted-foreground">{mitigation.vector.replace(/_/g, " ")}</td>
                      <td className="px-4 py-3">
                        <ActionBadge type={mitigation.action.type} rate={mitigation.action.rate} />
                      </td>
                      <td className="px-4 py-3 text-foreground">{mitigation.customer}</td>
                      <td className="px-4 py-3 text-muted-foreground">{formatRelativeTime(mitigation.createdAt)}</td>
                      <td
                        className={cn(
                          "px-4 py-3 font-mono",
                          expiresInfo.isWarning ? "text-warning" : "text-muted-foreground",
                        )}
                      >
                        {expiresInfo.text}
                      </td>
                      <td className="px-4 py-3 text-center">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          onClick={(e) => {
                            e.stopPropagation()
                            setSelectedMitigation(mitigation)
                          }}
                        >
                          <Eye className="h-4 w-4" />
                        </Button>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          <div className="flex items-center justify-between px-4 py-3 border-t border-border bg-secondary">
            <span className="text-sm text-muted-foreground">
              Showing {(currentPage - 1) * itemsPerPage + 1} to{" "}
              {Math.min(currentPage * itemsPerPage, filteredMitigations.length)} of {filteredMitigations.length}{" "}
              mitigations
            </span>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                disabled={currentPage === 1}
              >
                Previous
              </Button>
              <span className="text-sm text-muted-foreground">
                Page {currentPage} of {totalPages || 1}
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                disabled={currentPage === totalPages || totalPages === 0}
              >
                Next
              </Button>
            </div>
          </div>
        </div>

        <div className="lg:hidden flex flex-col gap-3 p-4 bg-card border border-border rounded-lg">
          <span className="text-sm text-muted-foreground text-center">
            {(currentPage - 1) * itemsPerPage + 1}-{Math.min(currentPage * itemsPerPage, filteredMitigations.length)} of{" "}
            {filteredMitigations.length}
          </span>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              className="flex-1 h-11 bg-transparent"
              onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
              disabled={currentPage === 1}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              className="flex-1 h-11 bg-transparent"
              onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
              disabled={currentPage === totalPages || totalPages === 0}
            >
              Next
            </Button>
          </div>
        </div>
      </div>

      {/* Detail Panel */}
      {selectedMitigation && (
        <>
          <div className="fixed inset-0 bg-background/80 z-40" onClick={() => setSelectedMitigation(null)} />
          <MitigationDetailPanel mitigation={selectedMitigation} onClose={() => setSelectedMitigation(null)} />
        </>
      )}
    </>
  )
}
