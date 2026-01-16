"use client"

import { useState, useMemo } from "react"
import Link from "next/link"
import { ExternalLink, Search, ChevronDown, ChevronUp, ChevronRight } from "lucide-react"
import { ActionTypeBadge } from "@/components/dashboard/action-type-badge"
import { ActorBadge } from "@/components/dashboard/actor-badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { mockAuditLog, formatTimestamp } from "@/lib/mock-data"
import { cn } from "@/lib/utils"

type SortField = "timestamp" | "actor" | "action" | "target"
type SortDirection = "asc" | "desc"

const actions = ["All", "announce", "withdraw", "reject", "escalate", "extend", "create"] as const
const actorTypes = ["All", "system", "operator"] as const

export function AuditLogContent() {
  const [actionFilter, setActionFilter] = useState<string>("All")
  const [actorFilter, setActorFilter] = useState<string>("All")
  const [searchQuery, setSearchQuery] = useState("")
  const [sortField, setSortField] = useState<SortField>("timestamp")
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc")
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set())
  const [currentPage, setCurrentPage] = useState(1)
  const itemsPerPage = 15

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"))
    } else {
      setSortField(field)
      setSortDirection("desc")
    }
  }

  const toggleExpand = (id: string) => {
    setExpandedRows((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const filteredLogs = useMemo(() => {
    return mockAuditLog
      .filter((entry) => {
        if (actionFilter !== "All" && entry.action !== actionFilter) return false
        if (actorFilter !== "All" && entry.actor.type !== actorFilter) return false
        if (searchQuery && !entry.target.includes(searchQuery) && !entry.details.includes(searchQuery)) return false
        return true
      })
      .sort((a, b) => {
        let comparison = 0
        switch (sortField) {
          case "timestamp":
            comparison = a.timestamp.getTime() - b.timestamp.getTime()
            break
          case "actor":
            comparison = a.actor.name.localeCompare(b.actor.name)
            break
          case "action":
            comparison = a.action.localeCompare(b.action)
            break
          case "target":
            comparison = a.target.localeCompare(b.target)
            break
        }
        return sortDirection === "asc" ? comparison : -comparison
      })
  }, [actionFilter, actorFilter, searchQuery, sortField, sortDirection])

  const totalPages = Math.ceil(filteredLogs.length / itemsPerPage)
  const paginatedLogs = filteredLogs.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null
    return sortDirection === "asc" ? <ChevronUp className="h-3 w-3 ml-1" /> : <ChevronDown className="h-3 w-3 ml-1" />
  }

  const getTargetLink = (target: string) => {
    if (target.startsWith("mit-")) {
      return (
        <Link
          href={`/mitigations?id=${target}`}
          className="inline-flex items-center gap-1 text-primary hover:underline font-mono text-xs"
        >
          {target}
          <ExternalLink className="h-3 w-3" />
        </Link>
      )
    }
    if (target.startsWith("evt-")) {
      return (
        <Link
          href={`/events?id=${target}`}
          className="inline-flex items-center gap-1 text-primary hover:underline font-mono text-xs"
        >
          {target}
          <ExternalLink className="h-3 w-3" />
        </Link>
      )
    }
    return <span className="font-mono text-xs text-foreground">{target}</span>
  }

  return (
    <div className="space-y-4">
      {/* Filter Bar */}
      <div className="flex flex-wrap items-center gap-3 p-4 bg-card border border-border rounded-lg">
        {/* Action Filter */}
        <Select value={actionFilter} onValueChange={setActionFilter}>
          <SelectTrigger className="w-32 h-8 bg-secondary border-border text-sm">
            <SelectValue placeholder="Action" />
          </SelectTrigger>
          <SelectContent>
            {actions.map((a) => (
              <SelectItem key={a} value={a}>
                {a === "All" ? "All Actions" : a}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {/* Actor Type Filter */}
        <Select value={actorFilter} onValueChange={setActorFilter}>
          <SelectTrigger className="w-32 h-8 bg-secondary border-border text-sm">
            <SelectValue placeholder="Actor" />
          </SelectTrigger>
          <SelectContent>
            {actorTypes.map((a) => (
              <SelectItem key={a} value={a}>
                {a === "All" ? "All Actors" : a}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {/* Search */}
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search target or details..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 h-8 bg-secondary border-border text-sm"
          />
        </div>
      </div>

      {/* Data Table */}
      <div className="bg-card border border-border rounded-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-secondary">
              <tr className="border-b border-border">
                <th className="w-8 px-2 py-3"></th>
                <th
                  className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleSort("timestamp")}
                >
                  <span className="flex items-center">
                    Timestamp
                    <SortIcon field="timestamp" />
                  </span>
                </th>
                <th
                  className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleSort("actor")}
                >
                  <span className="flex items-center">
                    Actor
                    <SortIcon field="actor" />
                  </span>
                </th>
                <th
                  className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleSort("action")}
                >
                  <span className="flex items-center">
                    Action
                    <SortIcon field="action" />
                  </span>
                </th>
                <th
                  className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleSort("target")}
                >
                  <span className="flex items-center">
                    Target
                    <SortIcon field="target" />
                  </span>
                </th>
                <th className="text-left px-4 py-3 font-medium text-muted-foreground">Details</th>
              </tr>
            </thead>
            <tbody>
              {paginatedLogs.map((entry, index) => {
                const isExpanded = expandedRows.has(entry.id)
                const isTruncated = entry.details.length > 60
                return (
                  <tr
                    key={entry.id}
                    className={cn(
                      "border-b border-border/50 hover:bg-secondary/50 transition-colors",
                      index % 2 === 1 && "bg-secondary/20",
                    )}
                  >
                    <td className="px-2 py-2">
                      {isTruncated && (
                        <button
                          onClick={() => toggleExpand(entry.id)}
                          className="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                        >
                          <ChevronRight className={cn("h-4 w-4 transition-transform", isExpanded && "rotate-90")} />
                        </button>
                      )}
                    </td>
                    <td className="px-4 py-2 font-mono text-muted-foreground whitespace-nowrap">
                      {formatTimestamp(entry.timestamp)}
                    </td>
                    <td className="px-4 py-2">
                      <ActorBadge type={entry.actor.type} name={entry.actor.name} />
                    </td>
                    <td className="px-4 py-2">
                      <ActionTypeBadge action={entry.action} />
                    </td>
                    <td className="px-4 py-2">{getTargetLink(entry.target)}</td>
                    <td className="px-4 py-2 text-muted-foreground max-w-md">
                      {isExpanded || !isTruncated ? (
                        entry.details
                      ) : (
                        <span className="truncate block max-w-xs">{entry.details.slice(0, 60)}...</span>
                      )}
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
            {Math.min(currentPage * itemsPerPage, filteredLogs.length)} of {filteredLogs.length} entries
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
    </div>
  )
}
