# Native Titlebar as Integrated Toolbar

**Date**: 2026-06-25

## Summary

Replace the native OS titlebar and the internal `AppShell` header with a single
integrated titlebar that embeds project identity, git context, Run/Stop,
terminal, and gear menu directly inside the window frame. On macOS, the native
traffic lights are preserved via `titleBarStyle: overlay`. On Windows and Linux,
a fully custom HTML titlebar replaces window decorations.

## Motivation

Currently the app has two horizontal bars consuming vertical space: the native OS
titlebar (identity only) and the internal `AppShell` header (actions + context).
This wastes space in a dense operational tool. Merging them into one titlebar
gives more room to logs and terminals while keeping all controls at a glance.

## Platform Architecture

| | macOS | Windows | Linux |
|---|---|---|---|
| Mechanism | `titleBarStyle: "overlay"` | `decorations: false` | `decorations: false` |
| Window controls | Native traffic lights (left) | HTML custom, Win11 style | HTML custom, GNOME/KDE style detected |
| Titlebar height | 38px (native frame height) | 38px (custom) | 38px (custom) |
| Background | Semi-transparent (vibrancy) | Opaque, theme background | Opaque, theme background |
| Drag region | `data-tauri-drag-region` | `data-tauri-drag-region` + `app-region: drag` | `data-tauri-drag-region` |

On macOS, traffic lights occupy the left ~70px. Content starts after that
padding. On Windows, custom window controls are on the right. On Linux, window
controls are on the right (GNOME) or right (KDE).

## Titlebar Layout

```
macOS:
┌──────────────────────────────────────────────────────────────────────┐
│ ● ● ●  my-project (main)  ▶  [setup: running]         ⬡  ⚙          │
└──────────────────────────────────────────────────────────────────────┘

Windows/Linux:
┌──────────────────────────────────────────────────────────────────────┐
│ my-project (main)  ▶  [setup: running]         ⬡  ⚙  ─  □  ✕       │
└──────────────────────────────────────────────────────────────────────┘
```

### Zones (left to right)

| Zone | Content | Width | Behavior |
|---|---|---|---|
| **Window controls** | macOS: native traffic lights. Win/Linux: custom HTML buttons (min/max/close) | macOS: ~70px, Win: ~90px, Linux: ~75px | Always visible |
| **Identity** | Project name + git badge (branch or worktree, compact) | Flexible, min ~100px | Truncated if needed |
| **Actions** | Run/Stop button, "+" (register project) | ~80px | Hidden when `launchLocked` |
| **Context** | `▶ [process: running]` — selected process name + status, or terminal title + cwd, or project base dir fallback | Flexible | Truncated if needed, hidden on <600px width |
| **Tool actions** | Terminal open button (⬡), gear menu (⚙) | ~64px | Always visible |
| **Window controls** | (Win/Linux only — renders on right side after tool actions) | ~90px | Always visible |

### Resize breakpoints

- **< 600px**: Context zone hides
- **< 450px**: Git badge collapses to just branch/worktree name (no icon), identity truncates more aggressively
- Buttons never hide

## Components

### New: `TitleBar.svelte`

Root titlebar component. Renders at the top of the page, position fixed/sticky.
Handles platform detection for window controls placement.
Props: none (reads from `runtimeStore` directly).

### New: `WindowControls.svelte`

Renders minimize/maximize/close buttons in HTML. Only rendered on Windows and
Linux (not on macOS, where controls are native).
Detects OS theme and platform variant (Win11, GNOME, KDE) to match native look.
Invokes `getCurrentWindow().minimize()`, `.toggleMaximize()`, `.close()`.

### Modified: `RunStopButton.svelte`

No code change — just moved from `sidebarHeader` snippet into `TitleBar`.

### Modified: `AppShell.svelte`

Removes `sidebarHeader` and `contentHeader` slots. The layout becomes:

```
grid: [sidebar (ProcessList)] | [content (TerminalPane / LogViewer / empty)]
```

No header rows — the titlebar handles all header content above the grid.

### Modified: `+page.svelte`

- `TitleBar` component added at top of body
- `contentHeader` and `sidebarHeader` snippets removed
- `AppShell` no longer receives those snippet props
- `windowTitle` still updates the native title (for task switcher / alt-tab)

## CSS Drag Regions

```css
.titlebar {
  app-region: drag;
  user-select: none;
}
.titlebar button,
.titlebar input,
.titlebar [data-tauri-no-drag] {
  app-region: no-drag;
}
```

The entire titlebar area is draggable except interactive elements. On macOS with
`overlay`, a left padding of ~70px prevents content from overlapping the traffic
lights.

## Tauri Configuration Changes

### `tauri.conf.json`

The static config keeps `decorations: true` (default). Platform-specific
behavior is set programmatically in Rust:

```json
"windows": [
  {
    "title": "devapp",
    "width": 800,
    "height": 600
  }
]
```

### Programmatic window creation

In `main.rs` / `lib.rs` setup, apply platform-specific window config:

```rust
.setup(|app| {
    let window = app.get_webview_window("main").unwrap();

    #[cfg(target_os = "macos")]
    {
        window.set_title_bar_style(TitleBarStyle::Overlay).unwrap();
        // decorations stay true, native traffic lights preserved
    }

    #[cfg(not(target_os = "macos"))]
    {
        window.set_decorations(false).unwrap();
        // fully custom HTML titlebar with WindowControls component
    }

    Ok(())
})
```

In `commands.rs` for secondary windows:

```rust
let mut builder = WebviewWindowBuilder::new(&app_handle, label, url)
    .title(format!("{} — devapp", project.name));

#[cfg(not(target_os = "macos"))]
{
    builder = builder.decorations(false);
}

#[cfg(target_os = "macos")]
{
    builder = builder.title_bar_style(TitleBarStyle::Overlay);
}

builder.build().map_err(...)
```

### Capabilities (`default.json`)

Add window manipulation permissions:

```json
"core:window:default",
"core:window:allow-close",
"core:window:allow-minimize",
"core:window:allow-toggle-maximize",
"core:window:allow-start-dragging",
"core:window:allow-is-maximized"
```

### Rust backend (`main.rs` or `lib.rs`)

On macOS, set the titlebar style at setup:

```rust
#[cfg(target_os = "macos")]
{
    window.title_bar_style(TitleBarStyle::Overlay);
}
```

## Data Flow

```
runtimeStore.project         ──→  TitleBar identity (project name)
runtimeStore.gitInfo         ──→  TitleBar git badge (branch/worktree)
runtimeStore.selection       ──→  TitleBar context line
runtimeStore.sessionActive   ──→  RunStopButton active state
runtimeStore.busy            ──→  RunStopButton / button disabled states
runtimeStore.launchLocked    ──→  Hide "+" button
runtimeStore.windowTitle     ──→  Native title (alt-tab / task switcher)
```

All titlebar UI reads directly from `runtimeStore` — no new props or events.

## Git Info Display

The existing `runtimeStore.gitInfo` and `windowTitle` getter are reused.
The git badge in the titlebar shows a compact version:

- Branch: icon + branch name (e.g., `⎇ main`)
- Worktree: icon + worktree name (e.g., `⊶ fix-login`)
- No git: nothing shown

Format rules from `2026-06-25-window-title-git-info-design.md` are preserved.

## Interactions with Existing Design Docs

This design **absorbs** `2026-06-25-window-title-git-info-design.md`. The git
info enrichment still updates the native window title (for task switcher), but
also renders as a badge directly in the custom titlebar. The backend git
detection module and frontend polling remain unchanged.

## Behavior Details

- **Session start**: Git polling starts, titlebar identity shows project + git
- **Session stop**: Polling stops, git badge disappears, shows project name only
- **No project**: Titlebar shows "devapp" as identity
- **No git**: Git badge hidden, rest of titlebar unchanged
- **No selection**: Context shows project base dir as fallback
- **Launch locked**: "+" button hidden, project name shown, Run/Stop hidden

## Scope

### In scope

- Custom titlebar with integrated toolbar (macOS overlay + Win/Linux custom)
- Platform-native-looking window controls on Windows and Linux
- Migration of AppShell header content into titlebar
- Tauri config and permissions for window manipulation
- Responsive layout adaptations
- Drag region CSS

### Out of scope

- Changing the git detection module (reused as-is)
- Adding new actions or buttons beyond what exists
- Persistent titlebar state across sessions
- Keyboard shortcut changes
- Window shadow, rounded corners, or OS-level theming beyond what CSS can do

## Risks and Mitigations

- **macOS overlay caveats**: Titlebar height varies (28-56px depending on
  toolbar presence). Mitigation: use `titleBarStyle: overlay` which keeps the
  standard height.
- **Linux DE detection**: GNOME and KDE have different window control layouts.
  Mitigation: detect via `XDG_CURRENT_DESKTOP` env var, fall back to
  right-aligned buttons.
- **Double titlebar during transition**: The `TitleBar` component must be hidden
  on non-macOS until the backend has set `decorations: false`, otherwise the
  app shows two titlebars temporarily. Mitigation: use a `platformReady`
  signal from the backend, or show the custom titlebar immediately since the
  backend setup runs before the webview loads.
- **Drag conflicts**: If interactive elements aren't excluded from drag region,
  buttons become unclickable. Mitigation: explicit `app-region: no-drag` on all
  interactive children of the titlebar.

## Testing Strategy

- **macOS**: Manual validation that traffic lights are visible and functional,
  content doesn't overlap, drag works
- **Windows**: Manual validation with Win11 — do custom buttons match native look?
  Do they work?
- **Linux (GNOME)**: Manual validation with GNOME — controls position correct?
- **Resize**: Shrink window to <600px, verify context hides; to <450px, verify
  badge collapses
- **Launch locked**: Open a project via `?projectId=X`, verify "+" hidden and
  Run/Stop hidden
