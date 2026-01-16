"use client"

import { useState, useMemo } from "react"
import { ExternalLink, Search, ChevronDown, ChevronUp, Filter } from "lucide-react"
import Link from "next/link"
import { SourceBadge } from "@/components/dashboard/source-badge"
import { ConfidenceBar } from "@/components/dashboard/confidence-bar"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { mockEvents, formatTimestamp, formatBps, formatPps, type Event } from "@/lib/mock-data"
import { cn } from "@/lib/utils"

type SortField = "timestamp" | "source" | "victimIp" | "vector" | "traffic" | "confidence"
type SortDirection = "asc" | "desc"

const sources = ["All", "fastnetmon", "noc", "custom"] as const
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

export function EventsContent() {
  const [sourceFilter, setSourceFilter] = useState<string>("All")
  const [vectorFilter, setVectorFilter] = useState("All")
  const [searchQuery, setSearchQuery] = useState("")
  const [sortField, setSortField] = useState<SortField>("timestamp")
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc")
  const [currentPage, setCurrentPage] = useState(1)
  const [showFilters, setShowFilters] = useState(false)
  const itemsPerPage = 10

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"))
    } else {
      setSortField(field)
      setSortDirection("desc")
    }
  }

  const filteredEvents = useMemo(() => {
    return mockEvents
      .filter((e) => {
        if (sourceFilter !== "All" && e.source !== sourceFilter) return false
        if (vectorFilter !== "All" && e.vector !== vectorFilter) return false
        if (searchQuery && !e.victimIp.includes(searchQuery)) return false
        return true
      })
      .sort((a, b) => {
        let comparison = 0
        switch (sortField) {
          case "timestamp":
            comparison = a.timestamp.getTime() - b.timestamp.getTime()
            break
          case "source":
            comparison = a.source.localeCompare(b.source)
            break
          case "victimIp":
            comparison = a.victimIp.localeCompare(b.victimIp)
            break
          case "vector":
            comparison = a.vector.localeCompare(b.vector)
            break
          case "traffic":
            comparison = a.trafficBps - b.trafficBps
            break
          case "confidence":
            comparison = a.confidence - b.confidence
            break
        }
        return sortDirection === "asc" ? comparison : -comparison
      })
  }, [sourceFilter, vectorFilter, searchQuery, sortField, sortDirection])

  const totalPages = Math.ceil(filteredEvents.length / itemsPerPage)
  const paginatedEvents = filteredEvents.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null
    return sortDirection === "asc" ? <ChevronUp className="h-3 w-3 ml-1" /> : <ChevronDown className="h-3 w-3 ml-1" />
  }

  const getMitigationLink = (result: string) => {
    if (result === "rejected" || result === "deduplicated") {
      return (
        <span
          className={cn(
            "inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium",
            result === "rejected"
              ? "bg-warning/20 text-warning border border-warning/30"
              : "bg-muted text-muted-foreground border border-muted-foreground/30",
          )}
        >
          {result === "rejected" ? "Rejected" : "Deduplicated"}
        </span>
      )
    }
    return (
      <Link
        href={`/mitigations?id=${result}`}
        className="inline-flex items-center gap-1 text-primary hover:underline font-mono text-xs"
      >
        <span className="truncate max-w-[100px]">{result.slice(0, 12)}...</span>
        <ExternalLink className="h-3 w-3 shrink-0" />
      </Link>
    )
  }

  const MobileCard = ({ event }: { event: Event }) => (
    <div className="p-4 bg-card border border-border rounded-lg space-y-3">
      <div className="flex items-center justify-between">
        <SourceBadge source={event.source} />
        <span className="font-mono text-xs text-muted-foreground">{formatTimestamp(event.timestamp)}</span>
      </div>
      <div className="space-y-2">
        <div className="flex justify-between items-center">
          <span className="text-xs text-muted-foreground">Victim IP</span>
          <span className="font-mono text-sm text-foreground">{event.victimIp}</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-xs text-muted-foreground">Vector</span>
          <span className="text-sm text-muted-foreground">{event.vector.replace(/_/g, " ")}</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-xs text-muted-foreground">Traffic</span>
          <div className="text-right">
            <div className="font-mono text-sm text-foreground">{formatBps(event.trafficBps)}</div>
            <div className="text-xs text-muted-foreground">{formatPps(event.trafficPps)}</div>
          </div>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-xs text-muted-foreground">Confidence</span>
          <ConfidenceBar value={event.confidence} />
        </div>
        <div className="flex justify-between items-center">
          <span className="text-xs text-muted-foreground">Result</span>
          {getMitigationLink(event.resultingMitigation)}
        </div>
      </div>
    </div>
  )

  return (
    <div className="space-y-4">
      <div className="bg-card border border-border rounded-lg overflow-hidden">
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
          <Select value={sourceFilter} onValueChange={setSourceFilter}>
            <SelectTrigger className="w-full sm:w-36 h-10 bg-secondary border-border text-sm">
              <SelectValue placeholder="Source" />
            </SelectTrigger>
            <SelectContent>
              {sources.map((s) => (
                <SelectItem key={s} value={s}>
                  {s === "All" ? "All Sources" : s}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

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
        </div>
      </div>

      <div className="lg:hidden space-y-3">
        {paginatedEvents.map((event) => (
          <MobileCard key={event.id} event={event} />
        ))}
      </div>

      <div className="hidden lg:block bg-card border border-border rounded-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-secondary">
              <tr className="border-b border-border">
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
                  onClick={() => handleSort("source")}
                >
                  <span className="flex items-center">
                    Source
                    <SortIcon field="source" />
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
                <th
                  className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleSort("traffic")}
                >
                  <span className="flex items-center">
                    Traffic
                    <SortIcon field="traffic" />
                  </span>
                </th>
                <th
                  className="text-left px-4 py-3 font-medium text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleSort("confidence")}
                >
                  <span className="flex items-center">
                    Confidence
                    <SortIcon field="confidence" />
                  </span>
                </th>
                <th className="text-left px-4 py-3 font-medium text-muted-foreground">Resulting Mitigation</th>
              </tr>
            </thead>
            <tbody>
              {paginatedEvents.map((event, index) => (
                <tr
                  key={event.id}
                  className={cn(
                    "border-b border-border/50 hover:bg-secondary/50 transition-colors",
                    index % 2 === 1 && "bg-secondary/20",
                  )}
                >
                  <td className="px-4 py-3 font-mono text-muted-foreground whitespace-nowrap">
                    {formatTimestamp(event.timestamp)}
                  </td>
                  <td className="px-4 py-3">
                    <SourceBadge source={event.source} />
                  </td>
                  <td className="px-4 py-3 font-mono text-foreground">{event.victimIp}</td>
                  <td className="px-4 py-3 text-muted-foreground">{event.vector.replace(/_/g, " ")}</td>
                  <td className="px-4 py-3">
                    <div className="flex flex-col">
                      <span className="font-mono text-foreground">{formatBps(event.trafficBps)}</span>
                      <span className="text-xs text-muted-foreground">{formatPps(event.trafficPps)}</span>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <ConfidenceBar value={event.confidence} />
                  </td>
                  <td className="px-4 py-3">{getMitigationLink(event.resultingMitigation)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        <div className="flex items-center justify-between px-4 py-3 border-t border-border bg-secondary">
          <span className="text-sm text-muted-foreground">
            Showing {(currentPage - 1) * itemsPerPage + 1} to{" "}
            {Math.min(currentPage * itemsPerPage, filteredEvents.length)} of {filteredEvents.length} events
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
          {(currentPage - 1) * itemsPerPage + 1}-{Math.min(currentPage * itemsPerPage, filteredEvents.length)} of{" "}
          {filteredEvents.length}
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
  )
}
