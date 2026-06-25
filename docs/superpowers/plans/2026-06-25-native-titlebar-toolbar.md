# Native Titlebar as Integrated Toolbar — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the native OS titlebar and internal AppShell header with a single integrated titlebar embedding project identity, git context, Run/Stop, terminal, and gear menu, with platform-native window controls on Windows/Linux.

**Architecture:** macOS uses `TitleBarStyle::Overlay` (native traffic lights preserved). Windows/Linux use `decorations: false` + a custom HTML `WindowControls` component. The `TitleBar` component replaces `sidebarHeader` and `contentHeader` snippets in `AppShell`. Backend changes are minimal: platform-conditional window setup in `lib.rs` and `commands.rs`, plus capability permissions.

**Tech Stack:** Tauri v2 (Rust), Svelte 5 (runes), Tailwind v4, TypeScript

## Global Constraints

- Cross-platform: macOS, Windows, Linux
- macOS: native traffic lights preserved via `TitleBarStyle::Overlay`
- Windows: custom HTML buttons styled like Win11
- Linux: custom HTML buttons, right-aligned (GNOME/KDE)
- Titlebar height: 38px
- `data-tauri-drag-region` for window dragging on all platforms
- Existing git info module and `runtimeStore.windowTitle` unchanged
- `deno task check` must pass
- `deno task build` must succeed
- Backend tests: `cd src-tauri && cargo test`

---

### Task 1: Backend — Platform-specific window setup

**Files:**
- Modify: `src-tauri/src/lib.rs:16-34`
- Modify: `src-tauri/src/tauri_api/commands.rs:1-5, 378-408`

**Interfaces:**
- Produces: macOS windows have `TitleBarStyle::Overlay`; Win/Linux windows have `decorations(false)`. Frontend `WindowControls` component renders on non-macOS only.

- [ ] **Step 1: Add platform-conditional window config in lib.rs `setup()`**

In `src-tauri/src/lib.rs`, after the `manage(app_state)` line and before the existing `.setup()` closure content, add window decoration logic:

```rust
// --- file: src-tauri/src/lib.rs ---
// Insert after `.plugin(tauri_plugin_opener::init())` and before `.setup(|app| {`

.setup(|app| {
    // Platform-specific window decorations
    #[cfg(target_os = "macos")]
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_title_bar_style(tauri::TitleBarStyle::Overlay)
            .expect("failed to set titlebar overlay style on macOS");
    }

    #[cfg(not(target_os = "macos"))]
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_decorations(false)
            .expect("failed to disable window decorations");
    }

    // --- existing setup code below ---
    let state = app.state::<AppState>().inner().clone();
    tauri::async_runtime::block_on(async move {
        // ... keep existing code unchanged ...
    })
    .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
    Ok(())
})
```

- [ ] **Step 2: Add platform-conditional decorations to secondary window creation in commands.rs**

Add `TitleBarStyle` import at top of `src-tauri/src/tauri_api/commands.rs`:

```rust
// line 4 — add TitleBarStyle to tauri imports
use tauri::{AppHandle, Manager, State, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};
```

Replace the `open_project_window` window builder (lines 402-405):

```rust
// --- replace lines 402-405 in src-tauri/src/tauri_api/commands.rs ---
    let url = WebviewUrl::App(format!("?projectId={project_id}&autorun=1").into());
    let win_builder = WebviewWindowBuilder::new(&app_handle, label, url)
        .title(format!("{} — devapp", project.name));

    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Overlay);

    #[cfg(not(target_os = "macos"))]
    let win_builder = win_builder.decorations(false);

    let window = win_builder.build().map_err(|error| error.to_string())?;
```

- [ ] **Step 3: Verify backend compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Run backend tests**

Run: `cd src-tauri && cargo test`
Expected: All existing tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/tauri_api/commands.rs
git commit -m "feat: platform-specific window decorations (macOS overlay, Win/Linux custom)"
```

---

### Task 2: Backend — Window permissions

**Files:**
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Produces: Frontend can call `getCurrentWindow().close()`, `.minimize()`, `.toggleMaximize()`, `.startDragging()`, `.isMaximized()`.

- [ ] **Step 1: Add window manipulation permissions**

Replace `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main", "project-*"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:window:default",
    "core:window:allow-close",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-start-dragging",
    "core:window:allow-is-maximized",
    "opener:default"
  ]
}
```

- [ ] **Step 2: Verify build**

Run: `deno task build`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/capabilities/default.json
git commit -m "feat: add window manipulation permissions for custom titlebar"
```

---

### Task 3: Frontend — WindowControls component

**Files:**
- Create: `src/lib/components/WindowControls.svelte`

**Interfaces:**
- Consumes: `getCurrentWindow` from `@tauri-apps/api/window`
- Produces: Renders min/max/close buttons styled per platform. Only used on non-macOS (conditionally rendered by `TitleBar`).

- [ ] **Step 1: Write the component**

Create `src/lib/components/WindowControls.svelte`:

```svelte
<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let isMaximized = $state(false);
  let platform = $state<"win32" | "linux">("linux");

  const appWindow = getCurrentWindow();

  $effect(() => {
    // Detect platform from navigator
    const p = navigator.platform.toLowerCase();
    platform = p.includes("win") ? "win32" : "linux";

    // Check initial maximized state
    void appWindow.isMaximized().then((v) => (isMaximized = v));
  });
</script>

<div class="flex items-center" data-tauri-no-drag>
  <button
    aria-label="Minimize"
    title="Minimize"
    class="window-control"
    onclick={() => appWindow.minimize()}
  >
    {#if platform === "win32"}
      <svg width="10" height="10" viewBox="0 0 10 10">
        <rect x="0" y="4.5" width="10" height="1" fill="currentColor" />
      </svg>
    {:else}
      <svg width="10" height="10" viewBox="0 0 10 10">
        <rect x="1" y="5" width="8" height="1" fill="currentColor" />
      </svg>
    {/if}
  </button>

  <button
    aria-label={isMaximized ? "Restore" : "Maximize"}
    title={isMaximized ? "Restore" : "Maximize"}
    class="window-control"
    onclick={() => {
      void appWindow.toggleMaximize();
      isMaximized = !isMaximized;
    }}
  >
    {#if platform === "win32"}
      {#if isMaximized}
        <!-- Win11 restore icon -->
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="2" y="0.5" width="7" height="7" rx="0.5" fill="none" stroke="currentColor" stroke-width="1" />
          <rect x="0.5" y="2" width="7" height="7" rx="0.5" fill="#0e0f12" stroke="currentColor" stroke-width="1" />
        </svg>
      {:else}
        <!-- Win11 maximize icon -->
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="0.5" y="0.5" width="9" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
      {/if}
    {:else}
      {#if isMaximized}
        <!-- Linux restore icon -->
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="2" y="0.5" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1" />
          <rect x="0.5" y="2" width="7" height="7" rx="1" fill="#0e0f12" stroke="currentColor" stroke-width="1" />
        </svg>
      {:else}
        <!-- Linux maximize icon -->
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="1" y="1" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
      {/if}
    {/if}
  </button>

  <button
    aria-label="Close"
    title="Close"
    class="window-control window-control-close"
    onclick={() => appWindow.close()}
  >
    <svg width="10" height="10" viewBox="0 0 10 10">
      <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  </button>
</div>

<style>
  .window-control {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 38px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--color-text-muted, #9aa0aa);
    cursor: default;
    -webkit-app-region: no-drag;
  }
  .window-control:hover {
    background: var(--color-surface-hover, #1b1d22);
    color: var(--color-text, #e7e9ee);
  }
  .window-control-close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `deno task check`
Expected: No new errors related to this file.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/WindowControls.svelte
git commit -m "feat: add WindowControls component for custom titlebar on Win/Linux"
```

---

### Task 4: Frontend — TitleBar component

**Files:**
- Create: `src/lib/components/TitleBar.svelte`

**Interfaces:**
- Consumes: `runtimeStore` (project, gitInfo, selection, selectedProcess, selectedTerminal, sessionActive, busy, launchLocked, openTitledTerminal, startCurrentProject, stopCurrentProject, clearError), `getCurrentWindow` for drag support.
- Produces: Rendered by `+page.svelte` at the top of the viewport.

- [ ] **Step 1: Write the component**

Create `src/lib/components/TitleBar.svelte`:

```svelte
<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import ProjectMenu from "$lib/components/ProjectMenu.svelte";
  import RunStopButton from "$lib/components/RunStopButton.svelte";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";

  const isMac = navigator.platform.toLowerCase().includes("mac");

  const project = $derived(runtimeStore.project);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);
  const selection = $derived(runtimeStore.selection);
  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const selectedTerminal = $derived(runtimeStore.selectedTerminal);
  const gitInfo = $derived(runtimeStore.gitInfo);

  let logActions = $state<{ copy: () => void; clear: () => void } | null>(null);

  function onDragStart(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-tauri-no-drag]")) return;
    void getCurrentWindow().startDragging();
  }

  async function openTerminal() {
    const terminal = await runtimeStore.openTitledTerminal(
      session && !session.stoppedAt ? session.projectId : undefined,
    );
    if (terminal) {
      runtimeStore.selectTerminal(terminal.terminalId);
    }
  }

  function openCreateDialog() {
    document.dispatchEvent(new CustomEvent("devapp:open-create-dialog"));
  }

  function openEditDialog(p: typeof project) {
    if (!p) return;
    document.dispatchEvent(new CustomEvent("devapp:open-edit-dialog", { detail: p }));
  }

  function openConfigDialog() {
    document.dispatchEvent(new CustomEvent("devapp:open-config-dialog"));
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header
  class="titlebar"
  style:padding-left={isMac ? "74px" : "12px"}
  style:padding-right={isMac ? "8px" : "0"}
  onmousedown={onDragStart}
>
  <!-- Left: Identity + Actions -->
  <div class="titlebar-left" data-tauri-no-drag>
    <div class="min-w-0">
      <div class="truncate text-sm font-semibold text-text">{project?.name ?? "devapp"}</div>
    </div>
    {#if gitInfo?.branch || gitInfo?.worktree}
      <span class="git-badge">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3" />
          <path d="M6 6v12M18 6v12" />
        </svg>
        {gitInfo.worktree ?? gitInfo.branch}
      </span>
    {/if}

    {#if !runtimeStore.launchLocked}
      <div class="titlebar-actions">
        <RunStopButton
          active={sessionActive}
          busy={runtimeStore.busy}
          disabled={!runtimeStore.projectId}
          onRun={() => runtimeStore.startCurrentProject()}
          onStop={() => runtimeStore.stopCurrentProject()}
        />
        <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
          +
        </IconButton>
      </div>
    {/if}
  </div>

  <!-- Center: Context -->
  <div class="titlebar-center" data-tauri-no-drag>
    {#if selection?.kind === "terminal" && selectedTerminal}
      <div class="truncate text-sm text-text-muted">{selectedTerminal.title}</div>
      <div class="truncate text-[10px] text-text-subtle">{selectedTerminal.cwd}</div>
    {:else if selectedProcess}
      <div class="truncate text-sm text-text-muted">▶ {selectedProcess.name}</div>
      <div class="truncate text-[10px] text-text-subtle">{selectedProcess.status}</div>
    {:else if project}
      <div class="truncate text-[10px] text-text-subtle">{project.baseDir}</div>
    {/if}
  </div>

  <!-- Right: Tool actions -->
  <div class="titlebar-right" data-tauri-no-drag>
    <IconButton
      label="Open terminal"
      disabled={!runtimeStore.projectId || runtimeStore.busy}
      onclick={openTerminal}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor"
        stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </svg>
    </IconButton>
    <ProjectMenu
      {project}
      {selection}
      {selectedProcess}
      {selectedTerminal}
      busy={runtimeStore.busy}
      {logActions}
      launchLocked={runtimeStore.launchLocked}
      onEditProject={(p) => openEditDialog(p)}
      onOpenConfig={openConfigDialog}
      onRestartProcess={(name) => runtimeStore.restartSessionProcess(name)}
      onStopProcess={(name) => runtimeStore.stopSessionProcess(name)}
      onCloseTerminal={() => runtimeStore.closeSelectedTerminal()}
      onOpenTerminal={openTerminal}
    />

    {#if !isMac}
      <WindowControls />
    {/if}
  </div>
</header>

<style>
  .titlebar {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    height: 38px;
    min-height: 38px;
    background: var(--color-surface, #0e0f12);
    user-select: none;
    -webkit-user-select: none;
    gap: 8px;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex-shrink: 1;
  }

  .titlebar-center {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-width: 0;
    overflow: hidden;
  }

  @media (max-width: 600px) {
    .titlebar-center {
      display: none;
    }
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .git-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--color-surface-raised, #15171b);
    font-size: 11px;
    color: var(--color-text-muted, #9aa0aa);
    flex-shrink: 0;
  }

  @media (max-width: 450px) {
    .git-badge {
      display: none;
    }
  }

  .titlebar-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }
</style>
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `deno task check`
Expected: No errors related to TitleBar.svelte.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/TitleBar.svelte
git commit -m "feat: add TitleBar component with integrated toolbar"
```

---

### Task 5: Frontend — Simplify AppShell (remove header slots)

**Files:**
- Modify: `src/lib/components/AppShell.svelte`

**Interfaces:**
- Consumes: (none — slots removed)
- Produces: Simpler layout: sidebar + content, no header rows.

- [ ] **Step 1: Remove sidebarHeader and contentHeader from AppShell**

Replace `src/lib/components/AppShell.svelte`:

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    processList: Snippet;
    children: Snippet;
  };

  let {
    processList,
    children,
  }: Props = $props();
</script>

<main class="h-screen min-h-0 overflow-hidden bg-canvas text-text" style="padding-top: 38px">
  <div
    class="grid h-full min-h-0 grid-rows-[minmax(260px,42vh)_minmax(0,1fr)] overflow-hidden lg:grid-cols-[260px_minmax(0,1fr)] lg:grid-rows-none"
  >
    <aside
      class="grid min-h-0 grid-rows-[minmax(0,1fr)] border-b border-border bg-surface lg:border-b-0 lg:border-r"
    >
      {@render processList()}
    </aside>

    <section class="grid min-h-0 grid-rows-[minmax(0,1fr)] bg-canvas">
      <div class="min-h-0">
        {@render children()}
      </div>
    </section>
  </div>
</main>
```

Key changes:
- Removed `sidebarHeader` and `contentHeader` props/slots
- Added `padding-top: 38px` to `<main>` to offset the fixed titlebar height
- Sidebar `<aside>` now uses `grid-rows-[minmax(0,1fr)]` (no `auto` row for header)
- Content `<section>` now uses `grid-rows-[minmax(0,1fr)]` (no `auto` row for header)

- [ ] **Step 2: Verify TypeScript compilation**

Run: `deno task check`
Expected: Errors in `+page.svelte` (props removed — expected, fixed in next task). No errors in AppShell itself.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/AppShell.svelte
git commit -m "refactor: remove header slots from AppShell for titlebar integration"
```

---

### Task 6: Frontend — Wire TitleBar in +page.svelte

**Files:**
- Modify: `src/routes/+page.svelte`

**Interfaces:**
- Consumes: `TitleBar` component, updated `AppShell` props (no header snippets)
- Produces: Full page with titlebar + simplified layout

- [ ] **Step 1: Replace header snippets with TitleBar and simplified AppShell**

Replace `src/routes/+page.svelte`:

Key changes:
1. Import `TitleBar` (line 4 area)
2. Remove `sidebarHeader` and `contentHeader` snippets (lines 155-266)
3. Replace AppShell props: remove `sidebarHeader`, `contentHeader`; keep `processList` and `children`

Here is the diff showing the changes:

**Add import** (after line 10):
```svelte
import TitleBar from "$lib/components/TitleBar.svelte";
```

**Remove** the two snippet blocks: `{#snippet sidebarHeader()}...{/snippet}` (lines 155-178) and `{#snippet contentHeader()}...{/snippet}` (lines 218-266).

**Replace AppShell usage** (lines 268-303):

```svelte
<!-- After the snippets, replace: -->
<TitleBar />

<AppShell
  {processList}
>
  <!-- ... children content unchanged (lines 273-301) ... -->
</AppShell>
```

**Remove** `setWindowTitle` from imports (line 15) — the titlebar handles the native title now. Actually keep it — the `$effect` still calls `setWindowTitle` for the task switcher. Keep as-is.

**Replace `openCreateDialog`, `openEditDialog`, `openConfigDialog`** with CustomEvent listeners since TitleBar communicates via DOM events:

At the end of the `<script>` block, after the `openTerminal` function (around line 146), add event listeners in `onMount`:

```typescript
// Add to onMount callback (around line 118-121):
onMount(() => {
  void runtimeStore.init();

  // Listen for dialog open events from TitleBar
  const onOpenCreate = () => openCreateDialog();
  const onOpenEdit = (e: Event) => {
    const detail = (e as CustomEvent).detail;
    if (detail) openEditDialog(detail);
  };
  const onOpenConfig = () => openConfigDialog();

  document.addEventListener("devapp:open-create-dialog", onOpenCreate);
  document.addEventListener("devapp:open-edit-dialog", onOpenEdit);
  document.addEventListener("devapp:open-config-dialog", onOpenConfig);

  return () => {
    void runtimeStore.teardown();
    document.removeEventListener("devapp:open-create-dialog", onOpenCreate);
    document.removeEventListener("devapp:open-edit-dialog", onOpenEdit);
    document.removeEventListener("devapp:open-config-dialog", onOpenConfig);
  };
});
```

Full file after changes — write the complete `src/routes/+page.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";

  import AppShell from "$lib/components/AppShell.svelte";
  import ConfigEditor from "$lib/components/ConfigEditor.svelte";
  import LogViewer from "$lib/components/LogViewer.svelte";
  import ProcessList from "$lib/components/ProcessList.svelte";
  import ProjectSettingsDialog from "$lib/components/ProjectSettingsDialog.svelte";
  import TerminalPane from "$lib/components/TerminalPane.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import { setWindowTitle } from "$lib/tauri/client";
  import { createShortcutRegistry } from "$lib/shortcuts/registry";
  import type { ProjectRecord } from "$lib/types";

  let detailsOpen = $state(false);
  let configOpen = $state(false);
  let editingProject = $state<ProjectRecord | null>(null);

  let logActions = $state<{ copy: () => void; clear: () => void } | null>(null);

  const project = $derived(runtimeStore.project);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);
  const selection = $derived(runtimeStore.selection);

  $effect(() => {
    const title = runtimeStore.windowTitle;
    document.title = title;
    setWindowTitle(title);
  });

  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const selectedTerminal = $derived(runtimeStore.selectedTerminal);

  const navigableItems = $derived.by(() => {
    const items: Array<
      | { kind: "process"; runtimeId: string }
      | { kind: "terminal"; terminalId: string }
    > = [];
    if (session) {
      for (const p of session.processes) {
        items.push({ kind: "process", runtimeId: p.runtimeId });
      }
    }
    for (const t of runtimeStore.terminals.filter((t) => t.isOpen)) {
      items.push({ kind: "terminal", terminalId: t.terminalId });
    }
    return items;
  });

  function navigateList(direction: 1 | -1) {
    const items = navigableItems;
    if (items.length === 0) return;

    let currentIndex = -1;
    if (selection) {
      currentIndex = items.findIndex((item) => {
        if (item.kind === "process" && selection.kind === "process") {
          return item.runtimeId === selection.runtimeId;
        }
        if (item.kind === "terminal" && selection.kind === "terminal") {
          return item.terminalId === selection.terminalId;
        }
        return false;
      });
    }

    let nextIndex = currentIndex + direction;
    if (nextIndex < 0) nextIndex = items.length - 1;
    if (nextIndex >= items.length) nextIndex = 0;

    const item = items[nextIndex];
    if (item.kind === "process") {
      runtimeStore.selectProcess(item.runtimeId);
    } else {
      runtimeStore.selectTerminal(item.terminalId);
    }
  }

  const shortcutHandler = createShortcutRegistry([
    {
      key: "Mod+Enter",
      description: "Start the current project",
      handler: () => { void runtimeStore.startCurrentProject(); },
      guard: () => !!runtimeStore.projectId && !sessionActive && !runtimeStore.busy,
    },
    {
      key: "Mod+.",
      description: "Stop the current project",
      handler: () => { void runtimeStore.stopCurrentProject(); },
      guard: () => sessionActive && !runtimeStore.busy,
    },
    {
      key: "Mod+T",
      description: "Open a new terminal",
      handler: () => { void openTerminal(); },
      guard: () => !!runtimeStore.projectId && !runtimeStore.busy,
    },
    {
      key: "Mod+J",
      description: "Select next sidebar item",
      handler: () => navigateList(1),
    },
    {
      key: "Mod+K",
      description: "Select previous sidebar item",
      handler: () => navigateList(-1),
    },
  ]);

  onMount(() => {
    void runtimeStore.init();

    const onOpenCreate = () => openCreateDialog();
    const onOpenEdit = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (detail) openEditDialog(detail);
    };
    const onOpenConfig = () => openConfigDialog();

    document.addEventListener("devapp:open-create-dialog", onOpenCreate);
    document.addEventListener("devapp:open-edit-dialog", onOpenEdit);
    document.addEventListener("devapp:open-config-dialog", onOpenConfig);

    return () => {
      void runtimeStore.teardown();
      document.removeEventListener("devapp:open-create-dialog", onOpenCreate);
      document.removeEventListener("devapp:open-edit-dialog", onOpenEdit);
      document.removeEventListener("devapp:open-config-dialog", onOpenConfig);
    };
  });

  function openCreateDialog() {
    if (runtimeStore.launchLocked) return;
    editingProject = null;
    detailsOpen = true;
  }

  function openEditDialog(openedProject: ProjectRecord) {
    if (runtimeStore.launchLocked) return;
    editingProject = openedProject;
    detailsOpen = true;
  }

  function openConfigDialog() {
    configOpen = true;
  }

  async function openTerminal() {
    const terminal = await runtimeStore.openTitledTerminal(
      session && !session.stoppedAt ? session.projectId : undefined,
    );
    if (terminal) {
      runtimeStore.selectTerminal(terminal.terminalId);
    }
  }
</script>

<svelte:head>
  <title>{runtimeStore.windowTitle}</title>
</svelte:head>

<svelte:window onkeydown={shortcutHandler} />

<TitleBar />

{#snippet processList()}
      <section class="min-h-0 overflow-y-auto px-3 pb-4 pt-4">
        {#if runtimeStore.uiError}
          <div class="mb-3 rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
            <div class="break-words">{runtimeStore.uiError}</div>
            <Button variant="ghost" size="sm" class="mt-2 h-auto px-0 text-danger/70 hover:bg-transparent hover:text-danger" onclick={() => runtimeStore.clearError()}>
              Dismiss
            </Button>
          </div>
        {/if}

        <div class="mb-2 flex items-center justify-between px-1">
          <h2 class="text-[11px] font-semibold uppercase tracking-wider text-text-subtle">Processes</h2>
          {#if session}
            <span class="text-[11px] text-text-subtle">
              since {new Date(session.startedAt).toLocaleTimeString()}
            </span>
          {/if}
        </div>
        <ProcessList
          processes={session?.processes ?? []}
          terminals={runtimeStore.terminals}
          selectedProcessRuntimeId={runtimeStore.selectedProcessRuntimeId}
          selectedTerminalId={runtimeStore.selectedTerminalId}
          busy={runtimeStore.busy}
          onSelectProcess={(runtimeId) => runtimeStore.selectProcess(runtimeId)}
          onSelectTerminal={(terminalId) => runtimeStore.selectTerminal(terminalId)}
          onStart={(processName) => runtimeStore.startSessionProcess(processName)}
          onStop={(processName) => runtimeStore.stopSessionProcess(processName)}
          onRestart={(processName) => runtimeStore.restartSessionProcess(processName)}
          onCloseTerminal={(terminalId) => {
            runtimeStore.selectTerminal(terminalId);
            runtimeStore.closeSelectedTerminal();
          }}
        />
      </section>
{/snippet}

<AppShell {processList}>
          {#if selection?.kind === "terminal" && selectedTerminal}
            <TerminalPane
              terminalId={selectedTerminal.terminalId}
              output={runtimeStore.terminalOutput[selectedTerminal.terminalId] ?? ""}
              onInput={(data) => runtimeStore.writeToTerminal(data)}
              onResize={(cols, rows) => runtimeStore.resizeSelectedTerminal(cols, rows)}
              onOpenTerminal={openTerminal}
            />
        {:else if selection?.kind === "process" && session}
          <LogViewer
            logs={runtimeStore.logsForSelectedProcess()}
            processName={selectedProcess?.name ?? null}
            truncatedCount={runtimeStore.truncatedLogCountForSelectedProcess()}
            onClear={() => runtimeStore.clearSelectedProcessLogs()}
            onActions={(actions) => (logActions = actions)}
          />
        {:else}
          <div class="grid h-full place-items-center px-6 text-center">
            <div class="max-w-sm">
              <div class="text-sm font-semibold text-text">
                {session ? "Select a process or terminal" : "No process is running"}
              </div>
              <p class="mt-2 text-sm leading-6 text-text-subtle">
                {session
                  ? "Choose an item in the sidebar to view its logs or terminal."
                  : "Start the current project to populate the process list, or open a terminal from the project menu."}
              </p>
            </div>
          </div>
        {/if}
</AppShell>

<ProjectSettingsDialog
  open={detailsOpen}
  project={editingProject}
  onClose={() => {
    detailsOpen = false;
    editingProject = null;
  }}
  onSave={async (input) => {
    await runtimeStore.saveProject(input);
  }}
  onRemove={async (project) => {
    await runtimeStore.removeProject(project.id);
  }}
/>

<ConfigEditor
  open={configOpen}
  {project}
  onClose={() => {
    configOpen = false;
  }}
/>
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `deno task check`
Expected: No errors.

- [ ] **Step 3: Verify build**

Run: `deno task build`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: wire TitleBar into +page.svelte, remove header snippets"
```

---

### Task 7: CSS — Global titlebar styles

**Files:**
- Modify: `src/app.css`

**Interfaces:**
- Produces: `[data-tauri-drag-region]` CSS for window dragging, `app-region: drag` for Windows.

- [ ] **Step 1: Add drag-region CSS at the end of app.css**

Append to `src/app.css` (after the `::selection` block at line 142):

```css
/* ── Titlebar drag region ──────────────────────────────────────────────── */

.titlebar {
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar button,
.titlebar input,
.titlebar select,
.titlebar [data-tauri-no-drag] {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

/* macOS overlay: prevent content from hiding behind the traffic lights */
@supports (-webkit-app-region: drag) {
  .titlebar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 50;
  }
}
```

- [ ] **Step 2: Verify build**

Run: `deno task build`
Expected: Build succeeds, CSS is bundled.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "feat: add titlebar drag-region CSS"
```

---

### Task 8: Frontend — LogActions forwarded via TitleBar

**Files:**
- Modify: `src/lib/components/TitleBar.svelte` (lines for logActions)

**Interfaces:**
- Consumes: `logActions` from `LogViewer` (copy/clear logs) — needs to reach `ProjectMenu` in the titlebar.
- Produces: `ProjectMenu` in titlebar has access to logActions.

**Note:** Currently `logActions` is set by `LogViewer` via `onActions` in `+page.svelte` and passed to `ProjectMenu` in the `contentHeader` snippet. With the titlebar, `ProjectMenu` is in `TitleBar.svelte` which doesn't have access to `logActions`. We need to move `logActions` state and the forwarding mechanism.

- [ ] **Step 1: Move logActions forwarding to +page.svelte and pass via CustomEvent**

The `LogViewer`'s `onActions` callback sets `logActions` in `+page.svelte`. The titlebar needs this value for `ProjectMenu`. Add a second CustomEvent dispatch in the `$effect` or modify the store.

Simplest approach: store `logActions` on the `runtimeStore` itself so it's accessible from anywhere.

Add to `src/lib/stores/runtime.svelte.ts` in the `RuntimeStore` class, after existing fields:

```typescript
// Add after line 64 (gitInfo field):
logActions = $state<{ copy: () => void; clear: () => void } | null>(null);
```

In `+page.svelte`, change the `LogViewer` onActions binding:

```svelte
<!-- replace: -->
onActions={(actions) => (logActions = actions)}

<!-- with: -->
onActions={(actions) => (runtimeStore.logActions = actions)}
```

Remove the local `logActions` state variable from `+page.svelte` (line 24).

In `TitleBar.svelte`, replace the local `logActions` with:

```typescript
const logActions = $derived(runtimeStore.logActions);
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `deno task check`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/runtime.svelte.ts src/lib/components/TitleBar.svelte src/routes/+page.svelte
git commit -m "fix: forward logActions through runtimeStore for TitleBar ProjectMenu"
```

---

### Task 9: Final verification

- [ ] **Step 1: Run full check**

```bash
deno task check
```

Expected: No errors (ignore pre-existing non-blocking SvelteKit `tsconfig.json` warning).

- [ ] **Step 2: Run full build**

```bash
deno task build
```

Expected: Build succeeds.

- [ ] **Step 3: Run backend tests**

```bash
cd src-tauri && cargo test
```

Expected: All tests pass.

- [ ] **Step 4: Visual validation on Linux**

Run `deno task app examples/deno-runner.yml` and verify:
- [ ] Custom titlebar visible at top (38px height)
- [ ] Project name displayed left
- [ ] Run/Stop button visible
- [ ] "+" button visible
- [ ] Terminal icon visible right
- [ ] Gear menu icon visible right
- [ ] Window controls (min/max/close) on the far right
- [ ] Window is draggable from the titlebar
- [ ] Min/max/close buttons work
- [ ] Buttons in titlebar are clickable (not blocked by drag)
- [ ] Sidebar + content area fill the remaining space below titlebar

- [ ] **Step 5: Commit verification**

```bash
git status
```

Expected: No uncommitted changes.
