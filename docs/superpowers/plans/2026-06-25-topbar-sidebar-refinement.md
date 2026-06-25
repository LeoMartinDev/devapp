# Top Bar & Sidebar Refinement — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Simplify the titlebar into a clean macOS/Zed-style identity bar and move Run/Stop to the sidebar.

**Architecture:** Three file edits: TitleBar drops RunStopButton, terminal button, and center section. AppShell shrinks padding-top from 38px to 36px. +page.svelte adds RunStopButton + divider at top of sidebar snippet.

**Tech Stack:** Svelte 5, Tailwind v4, Tauri v2

## Global Constraints

- Titlebar height: 36px
- Titlebar background: `bg-canvas` (`#08090b`)
- Sidebar Run/Stop: full-width, above divider, above PROCESSES header
- Window controls stay on right (no position change)
- Drag region behavior unchanged
- macOS padding-left stays at 74px

---

### Task 1: Simplify TitleBar

**Files:**
- Modify: `src/lib/components/TitleBar.svelte`

**Interfaces:**
- Produces: `TitleBar` component renders with grid `1fr auto`, 36px height, bg-canvas
- RunStopButton import removed; titlebar-actions div only contains `+` IconButton

- [ ] **Step 1: Remove RunStopButton import, unused derived state, and terminal open handler**

In `<script lang="ts">` section, apply these changes:

Remove `RunStopButton` import (line 5):
```svelte
import RunStopButton from "$lib/components/RunStopButton.svelte";
```

Remove unused derived state (lines 12-16) — `session`, `sessionActive`, `selection`, `selectedProcess`, `selectedTerminal`, `logActions` are still used by ProjectMenu props and `openTerminal`:
Keep: `session`, `sessionActive`, `selection`, `selectedProcess`, `selectedTerminal`, `logActions`

Remove the `openTerminal` function (lines 26-33). Actually keep it — ProjectMenu still uses it via `onOpenTerminal` prop (line 117). Keep it.

Actually, re-checking: `sessionActive` is only used by `RunStopButton` (line 74) and `openTerminal` (line 28). `openTerminal` is called by ProjectMenu (line 117). So keep `sessionActive` and `openTerminal`.

Remove nothing from `<script>` — all derived state is still used by ProjectMenu.

Wait — let me re-verify. After removing RunStopButton from the template:
- `sessionActive` — used by `openTerminal` (kept)
- `selection`, `selectedProcess`, `selectedTerminal` — used by ProjectMenu props (kept)
- `logActions` — used by ProjectMenu (kept)
- `session` — used by `sessionActive` (kept)
- `project`, `gitInfo` — used in template (kept)

So nothing to remove from script. Only template and style changes.

- [ ] **Step 2: Update template — remove RunStopButton, terminal button, center spacer**

Replace the entire `<header>` body (lines 57-123) with:

```svelte
<!-- svelte-ignore a11y_no_static_element_interactions -->
<header
  class="titlebar"
  data-tauri-drag-region
  style:padding-left={isMac ? "74px" : "16px"}
  style:padding-right={isMac ? "8px" : "8px"}
  onmousedown={onDragStart}
>
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
  </div>

  <div class="titlebar-right" data-tauri-no-drag>
    {#if !runtimeStore.launchLocked}
      <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
        +
      </IconButton>
    {/if}
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
```

Key changes:
- Padding-left changed from `"12px"` to `"16px"` (non-Mac)
- Padding-right changed from `"0"` to `"8px"` (non-Mac, symmetric)
- Removed RunStopButton from titlebar-left
- Removed center spacer div
- Removed terminal IconButton from titlebar-right
- Moved `+` IconButton (with `launchLocked` guard) into titlebar-right, before ProjectMenu

- [ ] **Step 3: Update CSS — new grid, height, background**

Replace the `<style>` block (lines 126-178) with:

```css
<style>
  .titlebar {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    height: 36px;
    min-height: 36px;
    background: var(--color-canvas, #08090b);
    user-select: none;
    -webkit-user-select: none;
    gap: 8px;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
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
</style>
```

Key changes:
- `grid-template-columns`: `auto 1fr auto` → `1fr auto`
- `height`/`min-height`: `38px` → `36px`
- `background`: `var(--color-surface, #0e0f12)` → `var(--color-canvas, #08090b)`
- Removed `.titlebar-actions` rule (no longer used)
- Removed `flex-shrink: 1` from `.titlebar-left` (no longer needed, only identity items)

- [ ] **Step 4: Run check**

```bash
deno task check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/TitleBar.svelte
git commit -m "feat: simplify titlebar to identity bar — remove Run/Stop, terminal button, center section"
```

---

### Task 2: Update AppShell padding

**Files:**
- Modify: `src/lib/components/AppShell.svelte:15`

**Interfaces:**
- Consumes: titlebar height from Task 1 (36px)
- Produces: unchanged — same `main` element, just different `padding-top`

- [ ] **Step 1: Change padding-top from 38px to 36px**

Line 15, change:
```svelte
<main class="h-screen min-h-0 overflow-hidden bg-canvas text-text" style="padding-top: 38px">
```
to:
```svelte
<main class="h-screen min-h-0 overflow-hidden bg-canvas text-text" style="padding-top: 36px">
```

- [ ] **Step 2: Run check**

```bash
deno task check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/AppShell.svelte
git commit -m "fix: match AppShell padding-top to new titlebar height (36px)"
```

---

### Task 3: Add RunStopButton to sidebar

**Files:**
- Modify: `src/routes/+page.svelte:168-204`

**Interfaces:**
- Consumes: `RunStopButton` component (unchanged), `sessionActive` derived from runtimeStore
- Produces: `processList` snippet with Run/Stop at top, divider, then PROCESSES header + list

- [ ] **Step 1: Add RunStopButton import**

Line 1-15 imports section — add after `ProcessList` import (line 7):
```svelte
import RunStopButton from "$lib/components/RunStopButton.svelte";
```

- [ ] **Step 2: Update processList snippet**

Replace the `processList` snippet (lines 168-204) with:

```svelte
{#snippet processList()}
      <section class="flex min-h-0 flex-col">
        {#if !runtimeStore.launchLocked}
          <div class="flex gap-2 px-3 pt-4 pb-3">
            <RunStopButton
              active={sessionActive}
              busy={runtimeStore.busy}
              disabled={!runtimeStore.projectId}
              onRun={() => runtimeStore.startCurrentProject()}
              onStop={() => runtimeStore.stopCurrentProject()}
            />
          </div>
          <div class="mx-3 border-t border-border"></div>
        {/if}

        <div class="min-h-0 overflow-y-auto px-3 pb-4 {runtimeStore.launchLocked ? 'pt-4' : 'pt-3'}">
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
        </div>
      </section>
{/snippet}
```

Key layout change: the outer `<section>` now uses `flex min-h-0 flex-col` to stack:
1. Run/Stop row (hidden when `launchLocked`)
2. Divider (hidden when `launchLocked`)
3. Scrollable area (error banner + PROCESSES header + ProcessList)

The scrolling is isolated to the bottom div so Run/Stop stays fixed at top.

- [ ] **Step 3: Run check**

```bash
deno task check
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: move RunStopButton from titlebar to sidebar top"
```
