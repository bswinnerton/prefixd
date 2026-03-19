---
name: frontend-worker
description: Implements Next.js frontend features for the prefixd dashboard (pages, components, hooks, tests)
---

# Frontend Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Use for features that involve:
- Next.js pages and components
- SWR data fetching hooks
- API client functions
- Sidebar navigation and command palette entries
- Frontend tests (Vitest + Testing Library)

## Work Procedure

1. **Read the feature description thoroughly.** Understand which dashboard views to build, what data they display, and which validation contract assertions this feature fulfills.

2. **Read AGENTS.md** for mission boundaries and frontend coding conventions.

3. **Read existing frontend patterns** before writing new code. Key files:
   - `frontend/app/(dashboard)/mitigations/page.tsx` — list page with filters, pagination, table
   - `frontend/app/(dashboard)/mitigations/[id]/page.tsx` — detail page pattern
   - `frontend/app/(dashboard)/config/page.tsx` — tabbed page with Settings/Playbooks/Alerting
   - `frontend/app/(dashboard)/admin/page.tsx` — tabbed page with admin controls
   - `frontend/components/dashboard/sidebar.tsx` — navigation items with badges
   - `frontend/components/dashboard/command-palette.tsx` — command palette entries
   - `frontend/hooks/use-api.ts` — SWR hooks pattern
   - `frontend/lib/api.ts` — API fetch functions
   - `frontend/components/ui/` — shadcn/ui components

4. **Write tests FIRST (TDD).** For component behavior:
   - Add tests in `frontend/__tests__/` following existing patterns
   - Use Vitest globals (`describe`, `it`, `expect`) and `@testing-library/react`
   - Run `cd frontend && bun run test` to confirm tests fail (red)

5. **Implement.** Write components to make tests pass (green). Follow existing patterns:
   - Pages: under `app/(dashboard)/` for auto auth guard
   - Components: shadcn/ui primitives (Card, Table, Tabs, Dialog, Badge, Button)
   - Data fetching: SWR hooks with 5s refresh, WebSocket invalidation
   - State: React hooks (useState, useEffect), no external state management
   - Styling: Tailwind CSS with theme variables, support light + dark mode
   - Permissions: `usePermissions()` hook for role-based UI gating
   - Navigation: Next.js `<Link>` for client-side routing

6. **Add API functions** in `frontend/lib/api.ts`:
   - Follow existing fetch wrapper pattern with 401 debounce
   - Return typed responses

7. **Add SWR hooks** in `frontend/hooks/use-api.ts`:
   - Follow existing `useMitigations`, `useEvents` pattern
   - Include cursor pagination support if needed

8. **Update navigation:**
   - Add sidebar item in `frontend/components/dashboard/sidebar.tsx`
   - Add command palette entry in `frontend/components/dashboard/command-palette.tsx`
   - Add keyboard shortcut if specified

9. **Run full validation:**
   ```bash
   cd frontend && bun run test
   cd frontend && bun run build
   ```
   Fix any failures before proceeding.

10. **Manual verification** with agent-browser if Docker stack is available:
    - Navigate to new pages
    - Test tab switching, filters, pagination
    - Verify dark mode appearance
    - Check for console errors

11. **Commit** with a descriptive message (`feat:`, `fix:`).

## Example Handoff

```json
{
  "salientSummary": "Built the Correlation page at /correlation with three sub-tabs (Signals, Groups, Config). Signals tab shows source status cards with health indicators and a recent signals table. Groups tab has filterable list with cursor pagination and group detail view showing contributing events timeline and confidence breakdown. Config tab has correlation settings editor and signal source CRUD (admin-only). Added sidebar nav item with open group count badge. 6 Vitest tests cover component rendering and data hooks. Frontend builds clean.",
  "whatWasImplemented": "frontend/app/(dashboard)/correlation/page.tsx (tabbed page with Signals/Groups/Config), frontend/app/(dashboard)/correlation/groups/[id]/page.tsx (group detail), frontend/components/dashboard/correlation/ (SignalSourceCards, SignalGroupList, GroupDetail, CorrelationConfig, SourceWeightViz), frontend/hooks/use-api.ts (useSignalGroups, useSignalGroupDetail, useCorrelationConfig, useSignalSources), frontend/lib/api.ts (getSignalGroups, getSignalGroupDetail, getCorrelationConfig, updateCorrelationConfig), sidebar.tsx (Correlation nav item with badge), command-palette.tsx (Correlation entry with g r shortcut)",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd frontend && bun run test", "exitCode": 0, "observation": "40 tests passed (34 existing + 6 new)" },
      { "command": "cd frontend && bun run build", "exitCode": 0, "observation": "Build succeeded, all routes generated" }
    ],
    "interactiveChecks": [
      { "action": "Navigate to /correlation via sidebar", "observed": "Page loads with Signals tab active, source cards visible" },
      { "action": "Switch to Groups tab, apply status=expired filter", "observed": "Table filters correctly, URL params updated" },
      { "action": "Click group row to view detail", "observed": "Detail page shows contributing events timeline, confidence breakdown, corroboration badge" },
      { "action": "Toggle dark mode", "observed": "All elements visible, health dots and badges have good contrast" }
    ]
  },
  "tests": {
    "added": [
      { "file": "frontend/__tests__/correlation.test.tsx", "cases": [
        { "name": "renders Signals tab by default", "verifies": "Default tab is Signals with source cards" },
        { "name": "renders Groups tab with filters", "verifies": "Groups tab shows filter controls and table" },
        { "name": "renders Config tab with settings form", "verifies": "Config tab shows editable form" }
      ]}
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Backend API endpoints this feature depends on don't exist yet
- Backend response shape differs from what was expected
- Existing frontend tests fail before your changes
- Component library (shadcn/ui) doesn't have a needed primitive
- Design decisions needed (layout, interaction patterns) not covered in feature description
