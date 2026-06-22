# UI Simplification — Design Spec

**Date:** 2026-06-21
**Status:** Validated, ready for implementation plan
**Scope:** Frontend only — Svelte components, runtime store. No backend changes.

## Context

The current UI is functionally complete but visually overloaded. The user
likes the color palette and the overall design language but finds the app too
dense and busy. The goal is to reduce visual noise while keeping the same
structure (sidebar + content) and the same underlying capabilities.

The color tokens, typography, and fonts in `src/app.css` are **out of scope** —
this is a structural and density cleanup, not a re-skin.

## Decisions (from brainstorming)

All choices below were validated through visual mockups in the brainstorming
companion:

1. **Layout direction**: Epurate in place. Keep sidebar (left) + content
   (right). No new regions, no removed regions.
2. **Process row**: dot + name + status text (discreet). Remove accent bar and
   `kind` badge.
3. **Content header**: title + single ⚙ menu. Remove path subtitle. Fold the
   three icon-buttons (terminal, settings, edit) into one ⚙ menu. The
   segmented Logs/Terminal control is removed (see decision 6 — the toggle it
   drives no longer exists).
4. **Logs toolbar**: keep search + autoscroll + pause. Remove kbd `/` hint,
   counter `X/Y`, and move copy/clear into the ⚙ menu.
5. **Sidebar header**: keep Run/Stop full-width button, remove path subtitle.
6. **Terminal model change (the major one)**: the Logs/Terminal toggle is
   removed entirely. Processes and terminals coexist in a single sidebar list.
   Opening a terminal is done from the ⚙ menu. The content pane adapts to the
   type of the selected sidebar item.

## Architecture

### The unified sidebar item model

The sidebar currently has two separate selection states:
`selectedProcessRuntimeId` and `selectedTerminalId`, switched by a
`runtimeView: "logs" | "terminal"` toggle. This is replaced by a single
selection that can point at either a process or a terminal.

The store gains a **discriminated selection** concept. Concretely:

- A new derived value on `RuntimeStore` represents "what is currently shown in
  the content pane": either a process or a terminal.
- Selecting a process clears terminal selection; selecting a terminal clears
  process selection. (Mutual exclusion — one content pane, one selection.)
- The content pane renders based on the kind of the selected item, not on a
  view toggle.

The existing store fields (`selectedProcessRuntimeId`,
`selectedTerminalId`) can remain as the underlying storage, but a single
derived `selection` accessor (or a discriminated helper) is added so the
content pane and the sidebar list each read one coherent value. This keeps the
change localized to the store + the two consumers.

### Sidebar list (unified)

`ProcessList.svelte` becomes a unified list rendering both processes and
terminals in a single flat sequence, in this order:

1. Processes (in session order, as today).
2. Terminals (in open order).

Each row is one of two kinds:

**Process row**
- Status dot (existing `StatusDot`).
- Name.
- Status text, discreet (e.g. `running`, `failed`, `succeeded · exit 0`) —
  same info as today, kept because exit codes matter operationally.
- On hover/selected only: restart button (stop is dropped from the row; see
  below).

**Terminal row**
- Terminal icon (`>_`) instead of a status dot.
- Title (terminal `title`).
- On hover only: close (`×`) button.

Removed from every row:
- The accent left bar (`absolute left-0 ... bg-accent`).
- The `kind` badge (`task` / `service`).
- The per-row stop button. Stopping an individual process is a secondary
  action; it is reachable from the ⚙ menu when a process is selected (the menu
  is contextual to the selection). Restart stays on the row because it is the
  most-used per-process action.

Selection styling: background `bg-surface-raised` for the selected row, no
accent bar.

### Content header

Reduced to one line:

- Title: selected process name, or selected terminal title.
- Inline status text: process status (e.g. `running`), or terminal cwd — the
  one piece of secondary context that earns its place.
- ⚙ menu button on the right.

Removed from the content header:
- The three icon-buttons (open terminal, runtime settings, edit project).
- The `SegmentedControl` Logs/Terminal.
- The separator.
- The path subtitle (redundant with project baseDir, already visible via the
  ⚙ menu / project dialog).

The ⚙ menu is **contextual**: its entries depend on what is selected.

- **Always**: Edit project, Runtime config.
- **When a process is selected**: Restart process, Stop process, Copy logs,
  Clear logs, Open terminal.
- **When a terminal is selected**: Close terminal, Open another terminal.

A new `ui/Menu.svelte` primitive (or equivalent lightweight popover) is needed.
The existing `IconButton` + `SegmentedControl` usages here are removed.

### Content body — adaptive

The body renders based on the selection kind:

- **Process selected** → `LogViewer` (with the slimmed toolbar, see below).
- **Terminal selected** → `TerminalPane` (xterm), full height.
- **Nothing selected / no session** → existing empty state.

The `runtimeView` state in `+page.svelte` is removed. The body choice is now
derived from the selection.

`TerminalPane.svelte` keeps its own internal close affordance removed —
closing goes through the ⚙ menu and the sidebar `×`. (Its current internal
header with a Close button is dropped; the pane becomes header-less and fills
the content body, since the content header already shows the terminal title.)

### Logs toolbar (slimmed)

`LogViewer.svelte` toolbar keeps:
- Search input (existing).
- Autoscroll toggle button.
- Pause toggle button.

Removed from the toolbar:
- The kbd `/` hint badge.
- The `X/Y` counter.
- Copy button → moves to ⚙ menu (process selected).
- Clear button → moves to ⚙ menu (process selected).

The keyboard shortcut `/` to focus search is **kept** (behavior), just not
visually hinted.

The paused banner and truncated-count banner remain (they are transient
status, not chrome).

### Sidebar header

`+page.svelte` `sidebarHeader` snippet:
- Project name + `+` (register) on line 1.
- Run/Stop full-width button on line 2.

Removed: the `baseDir` subtitle line.

### Project settings dialog & config editor

Unchanged. They are already dialogs; no density work needed there.

## Out of scope

- Color palette, fonts, typography (`src/app.css`).
- Backend (Rust/Tauri): no command changes, no new events, no type changes.
  The terminal commands (`openTerminal`, `closeTerminal`, `writeTerminal`,
  `resizeTerminal`) already exist and support multiple terminals.
- Config editor form/raw YAML.
- Project settings dialog.

## Files touched

| File | Change |
|---|---|
| `src/lib/stores/runtime.svelte.ts` | Add unified selection concept; remove view-toggle coupling. |
| `src/lib/components/ProcessList.svelte` | Render processes + terminals in one flat list; drop accent bar, kind badge, stop button; add terminal rows. |
| `src/lib/components/LogViewer.svelte` | Slim toolbar: keep search + autoscroll + pause; remove kbd hint, counter, copy, clear. |
| `src/lib/components/TerminalPane.svelte` | Remove internal header/close button; become header-less. |
| `src/lib/components/AppShell.svelte` | No structural change (still sidebar + content). |
| `src/routes/+page.svelte` | Remove `runtimeView` toggle; make content body adaptive to selection; simplify content header to title + ⚙ menu; simplify sidebar header (drop path). |
| `src/lib/components/ui/Menu.svelte` | **New** — lightweight popover menu primitive. |
| `src/lib/components/ui/SegmentedControl.svelte` | Becomes unused (it was only referenced by `+page.svelte`). Candidate for deletion during cleanup; confirm no other references before removing. |

## Risks & notes

- **Per-process stop moves to a menu.** Today stop is one hover-click on the
  row. After this change it is two clicks (select process, open ⚙, stop). This
  is an accepted trade-off for density; stop is less frequent than restart and
  the global Stop (whole session) remains one click in the sidebar header.
- **TerminalPane loses its own header.** Its close button is the main thing
  being removed; ensure closing is discoverable via the sidebar `×` (always
  visible on hover) and the ⚙ menu.
- **Menu primitive is net-new.** Keep it minimal: trigger button, list of
  items, click-away to close, keyboard optional in V1.
- **No backend changes** means no migration; all data the UI needs already
  flows through existing events and commands.

## Success criteria

- The sidebar shows processes and terminals in one flat list with no accent
  bars or kind badges.
- There is no Logs/Terminal toggle anywhere in the UI.
- Selecting a process shows logs; selecting a terminal shows xterm; both in
  the same content pane.
- The content header is a single line: title + status + ⚙.
- The logs toolbar shows only search + autoscroll + pause.
- The ⚙ menu exposes: project edit, runtime config, and selection-contextual
  actions (restart/stop/copy/clear for processes; close/open for terminals).
- `deno task check` passes.
