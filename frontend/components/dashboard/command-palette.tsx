"use client"

import { useCallback, useState } from "react"
import { useRouter } from "next/navigation"
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
} from "@/components/ui/command"
import { LayoutDashboard, Shield, Activity, FileText, Settings, Zap, Clock, XCircle } from "lucide-react"
import { mockMitigations, mockEvents, type Mitigation } from "@/lib/mock-data"

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const router = useRouter()
  const [search, setSearch] = useState("")

  const runCommand = useCallback(
    (command: () => void) => {
      onOpenChange(false)
      command()
    },
    [onOpenChange],
  )

  const filteredMitigations = mockMitigations
    .filter(
      (m) =>
        m.id.toLowerCase().includes(search.toLowerCase()) ||
        m.victimIp.includes(search) ||
        m.vector.toLowerCase().includes(search.toLowerCase()) ||
        m.customer.toLowerCase().includes(search.toLowerCase()),
    )
    .slice(0, 5)

  const filteredEvents = mockEvents
    .filter(
      (e) =>
        e.id.toLowerCase().includes(search.toLowerCase()) ||
        e.victimIp.includes(search) ||
        e.vector.toLowerCase().includes(search.toLowerCase()),
    )
    .slice(0, 5)

  const getStatusIcon = (status: Mitigation["status"]) => {
    switch (status) {
      case "active":
        return <span className="h-1.5 w-1.5 bg-primary" />
      case "escalated":
        return <span className="h-1.5 w-1.5 bg-destructive" />
      case "expired":
        return <Clock className="h-3 w-3 text-muted-foreground" />
      case "withdrawn":
        return <XCircle className="h-3 w-3 text-muted-foreground" />
    }
  }

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <Command className="border border-border bg-popover shadow-2xl">
        <CommandInput
          placeholder="Search mitigations, events, pages..."
          value={search}
          onValueChange={setSearch}
          className="border-b border-border font-mono text-sm"
        />
        <CommandList className="max-h-[400px]">
          <CommandEmpty className="py-6 text-center text-xs font-mono text-muted-foreground">
            No results found.
          </CommandEmpty>

          <CommandGroup heading="Pages">
            <CommandItem onSelect={() => runCommand(() => router.push("/"))} className="font-mono text-xs">
              <LayoutDashboard className="mr-2 h-3 w-3 opacity-60" />
              <span>Overview</span>
              <CommandShortcut className="font-mono">g o</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => router.push("/mitigations"))} className="font-mono text-xs">
              <Shield className="mr-2 h-3 w-3 opacity-60" />
              <span>Mitigations</span>
              <CommandShortcut className="font-mono">g m</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => router.push("/events"))} className="font-mono text-xs">
              <Activity className="mr-2 h-3 w-3 opacity-60" />
              <span>Events</span>
              <CommandShortcut className="font-mono">g e</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => router.push("/audit-log"))} className="font-mono text-xs">
              <FileText className="mr-2 h-3 w-3 opacity-60" />
              <span>Audit Log</span>
              <CommandShortcut className="font-mono">g a</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => router.push("/config"))} className="font-mono text-xs">
              <Settings className="mr-2 h-3 w-3 opacity-60" />
              <span>Config</span>
              <CommandShortcut className="font-mono">g c</CommandShortcut>
            </CommandItem>
          </CommandGroup>

          {search.length > 0 && filteredMitigations.length > 0 && (
            <>
              <CommandSeparator />
              <CommandGroup heading="Mitigations">
                {filteredMitigations.map((m) => (
                  <CommandItem
                    key={m.id}
                    onSelect={() => runCommand(() => router.push(`/mitigations?id=${m.id}`))}
                    className="flex items-center gap-3 font-mono text-xs"
                  >
                    {getStatusIcon(m.status)}
                    <span>{m.victimIp}</span>
                    <span className="text-muted-foreground">{m.vector}</span>
                    <span className="ml-auto text-muted-foreground">{m.customer}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </>
          )}

          {search.length > 0 && filteredEvents.length > 0 && (
            <>
              <CommandSeparator />
              <CommandGroup heading="Events">
                {filteredEvents.map((e) => (
                  <CommandItem
                    key={e.id}
                    onSelect={() => runCommand(() => router.push(`/events?id=${e.id}`))}
                    className="flex items-center gap-3 font-mono text-xs"
                  >
                    <span className="h-1.5 w-1.5 bg-muted-foreground" />
                    <span>{e.victimIp}</span>
                    <span className="text-muted-foreground">{e.vector}</span>
                    <span className="ml-auto text-muted-foreground tabular-nums">{e.confidence}%</span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </>
          )}

          <CommandSeparator />
          <CommandGroup heading="Quick Actions">
            <CommandItem
              onSelect={() => runCommand(() => router.push("/mitigations?status=active"))}
              className="font-mono text-xs"
            >
              <Zap className="mr-2 h-3 w-3 text-primary" />
              <span>View Active Mitigations</span>
            </CommandItem>
            <CommandItem
              onSelect={() => runCommand(() => router.push("/mitigations?status=escalated"))}
              className="font-mono text-xs"
            >
              <span className="mr-2 h-1.5 w-1.5 bg-destructive" />
              <span>View Escalated Mitigations</span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
        <div className="flex items-center justify-between border-t border-border px-3 py-2 text-[10px] font-mono text-muted-foreground">
          <div className="flex items-center gap-4">
            <span>
              <kbd className="kbd">↑↓</kbd> navigate
            </span>
            <span>
              <kbd className="kbd">↵</kbd> select
            </span>
            <span>
              <kbd className="kbd">esc</kbd> close
            </span>
          </div>
        </div>
      </Command>
    </CommandDialog>
  )
}
