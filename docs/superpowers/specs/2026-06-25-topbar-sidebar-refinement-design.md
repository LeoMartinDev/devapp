# Top Bar & Sidebar Refinement

**Date**: 2026-06-25

## Summary

Simplify the titlebar into a clean macOS/Zed-style identity bar (project name + git badge + compact tool buttons). Move Run/Stop from the titlebar into the sidebar, placing it prominently above the process list. Remove the center context section (process status text) from the titlebar entirely.

## Motivation

The current titlebar crams project identity, Run/Stop, process status text, terminal button, gear menu, and window controls into 38px. The result is visually cluttered: the project name sits too close to Run/Stop, the center status text ("setup succeeded") feels like noise, and there's no clear visual hierarchy.

## Design

### TitleBar (36px, seamless)

```
[  examples  ◉ main              [+] [⚙]        [_][□][X]  ]
```

- Height: 36px
- Background: `bg-canvas` (`#08090b`) — identical to body background, no visible seam
- No bottom border, no shadow
- Grid: `1fr auto` (two zones)
- Left zone (`1fr`): project name + optional git badge, padded 16px from left (74px on macOS for traffic lights)
- Right zone (`auto`): `+` button → gear menu → window controls (non-Mac)
- Removed: RunStopButton, terminal button, center context section

### Sidebar — Run/Stop migration

```
┌──────────────────┐
│  [▶ Run]   [■ Stop]│
│──────────────────│
│ PROCESSES   14:32 │
│  ◉ api  running   │
│  ◉ web  ready     │
│  >_ terminal-1    │
└──────────────────┘
```

- Run/Stop buttons: full-width, side by side, at the top of the sidebar
- Horizontal divider below
- "PROCESSES" header + session timestamp below the divider
- Rest of ProcessList unchanged

### Elements removed from titlebar

| Element | New location |
|---------|-------------|
| RunStopButton | Sidebar, top |
| Terminal icon button | Removed (accessible via sidebar terminals and Mod+T) |
| Center context text ("setup succeeded") | Removed entirely |

## Component Changes

### Modified: `TitleBar.svelte`

- Remove RunStopButton import and rendering
- Remove terminal IconButton
- Remove center div and associated CSS
- Change grid: `grid-template-columns: 1fr auto`
- Change height: `36px`
- Change background: inherit from body (`bg-canvas`)
- Keep: project name, git badge, `+` button, gear menu, WindowControls
- Keep: drag region, platform detection, padding

### Modified: `+page.svelte`

- Move RunStopButton import and rendering into the `processList` snippet, above the PROCESSES header
- Add divider between Run/Stop and the rest of the sidebar
- No changes to keyboard shortcuts

### Modified: `AppShell.svelte`

- Change `padding-top` from 38px to 36px to match new titlebar height

### Unchanged: `RunStopButton.svelte`

- No code changes — just imported and rendered in a different parent

### Unchanged: `WindowControls.svelte`, `ProjectMenu.svelte`

- No changes

## Behavior Details

- **With project loaded**: Titlebar shows project name + git badge. Run/Stop in sidebar.
- **No project**: Titlebar shows "devapp". Run hidden (disabled state). `+` visible.
- **Session active**: Run changes to Stop (red). Both visible in sidebar.
- **Launch locked**: `+` hidden. Run/Stop hidden.
- **No git**: Git badge absent. Titlebar shows only project name.
- **Resize**: Window controls and gear menu always visible. Git badge hides at <450px.

## Scope

### In scope

- TitleBar simplification (remove Run/Stop, terminal button, center section)
- Run/Stop migration to sidebar
- CSS adjustments (height 36px, bg-canvas, grid 2 cols, sidebar divider)
- AppShell padding-top update (38→36)

### Out of scope

- Changing Tauri window decorations (decorations stay true)
- Adding new visual elements (icons, avatars, etc.)
- Changing the LogViewer, TerminalPane, ConfigEditor, or dialogs
- Changing the git detection backend
- Keyboard shortcut changes
- macOS traffic light overlay (stays as-is with CSS padding-left)
