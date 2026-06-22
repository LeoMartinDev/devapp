# UI Simplification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Simplify devapp's UI density by unifying processes and terminals into a single sidebar list, removing the Logs/Terminal toggle, and slimming every toolbar/header to its essentials.

**Architecture:** Frontend-only change. The runtime store gains a unified, discriminated selection concept (one selection → either a process or a terminal). The sidebar list renders both kinds flat. The content pane adapts to the selection kind instead of a view toggle. A new lightweight `Menu` popover primitive hosts the folded actions. No backend changes — all Tauri commands and events already exist.

**Tech Stack:** Svelte 5 (runes: `$state`, `$derived`, `$effect`, `$props`, `Snippet`), Tailwind v4 with `@theme` tokens from `src/app.css`, TypeScript. No frontend test runner is configured — validation is `deno task check` (runs svelte-check) plus manual desktop validation.

**Reference spec:** `docs/superpowers/specs/2026-06-21-ui-simplification-design.md`

---

## File Structure

**New files:**
- `src/lib/components/ui/Menu.svelte` — lightweight popover menu primitive (trigger + items + click-away close). One responsibility: show a list of actions anchored to a trigger button.

**Modified files:**
- `src/lib/stores/runtime.svelte.ts` — add unified selection; helper to open a terminal with an auto-incremented title.
- `src/lib/components/ProcessList.svelte` — render processes + terminals flat; drop accent bar, kind badge, per-row stop button; add terminal rows with close affordance. Rename conceptually to a "sidebar list" but keep the filename to minimize churn.
- `src/lib/components/LogViewer.svelte` — slim toolbar: keep search + autoscroll + pause; remove kbd hint, counter, copy, clear (those move to the contextual menu).
- `src/lib/components/TerminalPane.svelte` — remove internal header/close button; render header-less, fill the content body.
- `src/routes/+page.svelte` — remove `runtimeView` toggle and `SegmentedControl`; make the content body adaptive to the selection; reduce content header to title + status + ⚙ menu; drop path from sidebar header; wire the ⚙ menu to store actions.

**Deleted files (cleanup task):**
- `src/lib/components/ui/SegmentedControl.svelte` — becomes unused after the toggle removal. Verified: only `+page.svelte` imports it.

---

## Task 1: Add a unified selection concept to the runtime store

**Files:**
- Modify: `src/lib/stores/runtime.svelte.ts`

The store currently keeps two independent selections (`selectedProcessRuntimeId`, `selectedTerminalId`) and the view is decided by a separate `runtimeView` state in `+page.svelte`. We add a single derived accessor that tells consumers "what is currently selected" without forcing them to read two fields and a toggle. We keep the underlying fields for storage but make selection mutually exclusive: selecting one kind clears the other.

- [ ] **Step 1: Add the unified selection types and accessor**

At the top of `src/lib/stores/runtime.svelte.ts`, after the existing imports, add a discriminated union and export it:

```ts
export type Selection =
  | { kind: "process"; runtimeId: ProcessRuntimeId }
  | { kind: "terminal"; terminalId: TerminalSessionId }
  | null;
```

Inside the `RuntimeStore` class, after the existing `get selectedTerminal()` getter (around line 94), add:

```ts
  get selection(): Selection {
    if (this.selectedProcessRuntimeId) {
      return { kind: "process", runtimeId: this.selectedProcessRuntimeId };
    }
    if (this.selectedTerminalId) {
      return { kind: "terminal", terminalId: this.selectedTerminalId };
    }
    return null;
  }
```

- [ ] **Step 2: Make selection mutually exclusive**

Modify the existing `selectProcess` and `selectTerminal` methods so selecting one kind clears the other. Replace:

```ts
  selectProcess(runtimeId: ProcessRuntimeId) {
    this.selectedProcessRuntimeId = runtimeId;
  }

  selectTerminal(terminalId: TerminalSessionId) {
    this.selectedTerminalId = terminalId;
  }
```

with:

```ts
  selectProcess(runtimeId: ProcessRuntimeId) {
    this.selectedProcessRuntimeId = runtimeId;
    this.selectedTerminalId = null;
  }

  selectTerminal(terminalId: TerminalSessionId) {
    this.selectedTerminalId = terminalId;
    this.selectedProcessRuntimeId = null;
  }
```

- [ ] **Step 3: Add a titled terminal opener**

The current `openProjectTerminal` opens a terminal with no title. The sidebar needs to show a readable title (`bash`, `bash 2`, …). Add a helper that derives the next title from the existing terminals. After the existing `openProjectTerminal` method, add:

```ts
  async openTitledTerminal(projectId = this.projectId) {
    if (!projectId) {
      return null;
    }
    const openCount = this.terminals.filter((t) => t.isOpen).length;
    const title = openCount === 0 ? "bash" : `bash ${openCount + 1}`;
    return this.openProjectTerminal(projectId, title);
  }
```

(`openProjectTerminal` already accepts an optional `title` and forwards it to the Tauri `open_terminal` command.)

- [ ] **Step 4: Verify it type-checks**

Run: `deno task check`
Expected: completes successfully (the known non-blocking SvelteKit `tsconfig.json` extension warning is fine — only treat a non-zero exit as failure).

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/runtime.svelte.ts
git commit -m "feat(runtime): add unified selection concept and titled terminal opener"
```

---

## Task 2: Create the Menu popover primitive

**Files:**
- Create: `src/lib/components/ui/Menu.svelte`

A minimal popover: a trigger slot, a list of items, click-away to close. It mirrors the styling of the existing UI primitives (hairline borders, `surface-raised` background, `text-muted` items). It uses Svelte 5 runes and a `Snippet` for items so callers control the markup.

- [ ] **Step 1: Create the component**

Create `src/lib/components/ui/Menu.svelte` with this content:

```svelte
<script lang="ts" module>
  export type MenuItem = {
    label: string;
    onSelect: () => void;
    disabled?: boolean;
    danger?: boolean;
    dividerAfter?: boolean;
  };
</script>

<script lang="ts">
  import { onMount } from "svelte";

  type Props = {
    label: string;
    items: MenuItem[];
    disabled?: boolean;
  };

  let { label, items, disabled = false }: Props = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement | null>(null);

  function onPointerDown(event: MouseEvent) {
    if (!root) {
      return;
    }
    if (!root.contains(event.target as Node)) {
      open = false;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (open && event.key === "Escape") {
      open = false;
    }
  }

  onMount(() => {
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("keydown", onKeydown);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("keydown", onKeydown);
    };
  });

  function choose(item: MenuItem) {
    if (item.disabled) {
      return;
    }
    open = false;
    item.onSelect();
  }
</script>

<div bind:this={root} class="relative">
  <button
    type="button"
    {disabled}
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label={label}
    title={label}
    class="grid h-8 w-8 place-items-center rounded-md text-text-muted transition-colors hover:bg-surface-hover hover:text-text disabled:cursor-not-allowed disabled:opacity-55"
    onclick={() => (open = !open)}
  >
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  </button>

  {#if open}
    <div
      role="menu"
      aria-label={label}
      class="absolute right-0 top-full z-30 mt-1 min-w-[180px] overflow-hidden rounded-md border border-border bg-surface-raised py-1 shadow-2xl"
    >
      {#each items as item, i (i)}
        <button
          type="button"
          role="menuitem"
          disabled={item.disabled}
          class={`flex w-full items-center px-3 py-1.5 text-left text-[13px] transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${
            item.disabled
              ? ""
              : item.danger
                ? "text-danger hover:bg-danger/10"
                : "text-text-muted hover:bg-surface-hover hover:text-text"
          }`}
          onclick={() => choose(item)}
        >
          {item.label}
        </button>
        {#if item.dividerAfter}
          <div class="my-1 h-px bg-border"></div>
        {/if}
      {/each}
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Verify it type-checks**

Run: `deno task check`
Expected: completes successfully.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/Menu.svelte
git commit -m "feat(ui): add Menu popover primitive"
```

---

## Task 3: Slim the LogViewer toolbar

**Files:**
- Modify: `src/lib/components/LogViewer.svelte`

Keep search + autoscroll + pause. Remove the kbd `/` hint, the `X/Y` counter, the copy button, and the clear button. The `copyLogs`, `clearLogs`, `query`, `autoScroll`, `paused` logic stays — only the toolbar chrome changes. Copy and clear remain invokable from `+page.svelte` via the contextual ⚙ menu in a later task, so we must keep `filteredLogs`, `copyLogs`, `clearLogs`, and the log count accessible. We expose them through props callbacks.

- [ ] **Step 1: Expose copy/clear via props so the menu can call them**

In `src/lib/components/LogViewer.svelte`, change the `Props` type and destructuring. Replace the existing `Props` and `let { ... }: Props = $props();`:

```ts
  type Props = {
    logs: ProcessLogPayload[];
    processName: string | null;
    truncatedCount: number;
    onClear: () => void;
  };

  let { logs, processName, truncatedCount, onClear }: Props = $props();
```

with:

```ts
  type Props = {
    logs: ProcessLogPayload[];
    processName: string | null;
    truncatedCount: number;
    onClear: () => void;
    /** Register externally-callable actions (copy/clear) so a parent menu can trigger them. */
    onActions?: (actions: { copy: () => void; clear: () => void }) => void;
  };

  let { logs, processName, truncatedCount, onClear, onActions }: Props = $props();
```

- [ ] **Step 2: Register the actions upward**

Inside the existing `<script>` block, just after the `clearLogs` and `copyLogs` function definitions (keep both functions as-is), add an effect that registers them with the parent:

```ts
  $effect(() => {
    onActions?.({ copy: copyLogs, clear: clearLogs });
  });
```

- [ ] **Step 3: Replace the toolbar markup**

Replace the entire `<div class="flex items-center gap-2 border-b border-border px-3 py-2">...</div>` toolbar block (the one containing the search input, the kbd hint, the counter, and the four action buttons) with this slimmed version:

```svelte
  <div class="flex items-center gap-2 border-b border-border px-3 py-2">
    <div class="relative min-w-0 flex-1">
      <svg
        class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-text-subtle"
        width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor"
        stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input
        bind:this={searchInput}
        bind:value={query}
        type="text"
        placeholder="Search logs"
        spellcheck="false"
        class="h-8 w-full rounded-md border border-border bg-surface-raised pl-8 pr-3 text-[13px] text-text outline-none transition-colors placeholder:text-text-subtle focus:border-accent"
      />
    </div>

    <div class="flex shrink-0 items-center gap-0.5">
      <button
        type="button"
        class={`grid h-8 w-8 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-hover hover:text-text ${
          autoScroll ? "text-accent hover:text-accent" : ""
        }`}
        onclick={() => (autoScroll = !autoScroll)}
        aria-pressed={autoScroll}
        aria-label={autoScroll ? "Auto-scroll on" : "Auto-scroll off"}
        title={autoScroll ? "Auto-scroll: on" : "Auto-scroll: off"}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>

      <button
        type="button"
        class="grid h-8 w-8 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-hover hover:text-text"
        onclick={togglePaused}
        aria-pressed={paused}
        aria-label={paused ? "Resume live log view" : "Pause live log view"}
        title={paused ? "Resume" : "Pause"}
      >
        {#if paused}
          <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M8 5v14l11-7z" />
          </svg>
        {:else}
          <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="5" width="4" height="14" rx="1" />
            <rect x="14" y="5" width="4" height="14" rx="1" />
          </svg>
        {/if}
      </button>
    </div>
  </div>
```

This keeps the `/` keyboard shortcut behavior (the `handleKeydown` function is untouched) but drops the visual `<kbd>` hint, the `X/Y` counter, and the copy/clear buttons.

- [ ] **Step 4: Verify it type-checks**

Run: `deno task check`
Expected: completes successfully.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/LogViewer.svelte
git commit -m "feat(logs): slim toolbar to search + autoscroll + pause"
```

---

## Task 4: Unify the sidebar list (processes + terminals)

**Files:**
- Modify: `src/lib/components/ProcessList.svelte`

This file currently renders only processes, with an accent bar, a kind badge, and restart+stop hover buttons. It becomes a flat list of processes then terminals. Process rows keep dot + name + status text + restart (hover/selected). Terminal rows use a terminal icon + title + close (hover). Accent bar and kind badge are removed. Stop is removed from the row (moves to the ⚙ menu).

- [ ] **Step 1: Update the Props and imports**

Replace the entire `<script lang="ts">...</script>` block of `src/lib/components/ProcessList.svelte` with:

```svelte
<script lang="ts">
  import StatusDot from "$lib/components/ui/StatusDot.svelte";
  import type {
    ProcessRuntimeId,
    ProcessSnapshot,
    ProcessStatus,
    TerminalSnapshot,
  } from "$lib/types";

  type Props = {
    processes: ProcessSnapshot[];
    terminals: TerminalSnapshot[];
    selectedProcessRuntimeId: ProcessRuntimeId | null;
    selectedTerminalId: string | null;
    busy?: boolean;
    onSelectProcess: (runtimeId: ProcessRuntimeId) => void;
    onSelectTerminal: (terminalId: string) => void;
    onRestart: (processName: string) => void;
    onCloseTerminal: (terminalId: string) => void;
  };

  let {
    processes,
    terminals,
    selectedProcessRuntimeId,
    selectedTerminalId,
    busy = false,
    onSelectProcess,
    onSelectTerminal,
    onRestart,
    onCloseTerminal,
  }: Props = $props();

  const restartDisabledStatuses = new Set<ProcessStatus>([
    "pending",
    "blocked",
    "starting",
    "stopping",
  ]);

  function canRestart(status: ProcessStatus) {
    return !busy && !restartDisabledStatuses.has(status);
  }

  function statusLabel(process: ProcessSnapshot) {
    return process.exitCode !== undefined && process.exitCode !== null
      ? `${process.status} · exit ${process.exitCode}`
      : process.status;
  }
</script>
```

- [ ] **Step 2: Replace the markup with the unified list**

Replace the entire `<div class="grid gap-0.5">...</div>` block with:

```svelte
<div class="grid gap-0.5">
  {#if processes.length === 0 && terminals.length === 0}
    <div
      class="rounded-lg border border-dashed border-border px-3 py-6 text-center text-xs leading-5 text-text-subtle"
    >
      No process loaded
    </div>
  {:else}
    {#each processes as process (process.runtimeId)}
      {@const selected = process.runtimeId === selectedProcessRuntimeId}
      <div
        class={`group relative flex items-center gap-2.5 rounded-md px-3 py-1.5 transition-colors ${
          selected ? "bg-surface-raised" : "hover:bg-surface-hover"
        }`}
      >
        <button
          type="button"
          class="grid min-w-0 flex-1 grid-cols-[auto_minmax(0,1fr)] items-center gap-2.5 text-left"
          aria-current={selected ? "true" : undefined}
          onclick={() => onSelectProcess(process.runtimeId)}
        >
          <StatusDot status={process.status} />
          <span class="min-w-0">
            <span class="block truncate text-[13px] font-medium text-text">{process.name}</span>
            <span class="block truncate text-[11px] text-text-subtle">
              {statusLabel(process)}
            </span>
          </span>
        </button>

        <div
          class="flex shrink-0 items-center gap-0.5 transition md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
        >
          <button
            type="button"
            class="grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
            disabled={!canRestart(process.status)}
            aria-label={`Restart ${process.name}`}
            title={`Restart ${process.name}`}
            onclick={(event) => {
              event.stopPropagation();
              onRestart(process.name);
            }}
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
              <path d="M3 3v5h5" />
            </svg>
          </button>
        </div>
      </div>
    {/each}

    {#each terminals.filter((t) => t.isOpen) as terminal (terminal.terminalId)}
      {@const selected = terminal.terminalId === selectedTerminalId}
      <div
        class={`group relative flex items-center gap-2.5 rounded-md px-3 py-1.5 transition-colors ${
          selected ? "bg-surface-raised" : "hover:bg-surface-hover"
        }`}
      >
        <button
          type="button"
          class="grid min-w-0 flex-1 grid-cols-[auto_minmax(0,1fr)] items-center gap-2.5 text-left"
          aria-current={selected ? "true" : undefined}
          onclick={() => onSelectTerminal(terminal.terminalId)}
        >
          <svg class="h-3.5 w-3.5 shrink-0 text-text-subtle" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" y1="19" x2="20" y2="19" />
          </svg>
          <span class="min-w-0">
            <span class="block truncate text-[13px] font-medium text-text">{terminal.title}</span>
            <span class="block truncate text-[11px] text-text-subtle">{terminal.cwd}</span>
          </span>
        </button>

        <div
          class="flex shrink-0 items-center gap-0.5 transition md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
        >
          <button
            type="button"
            class="grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-danger/10 hover:text-danger disabled:cursor-not-allowed disabled:opacity-40"
            aria-label={`Close ${terminal.title}`}
            title={`Close ${terminal.title}`}
            onclick={(event) => {
              event.stopPropagation();
              onCloseTerminal(terminal.terminalId);
            }}
          >
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>
    {/each}
  {/if}
</div>
```

Note: the terminal row uses a chevron-terminal SVG icon (matching the existing "open terminal" icon used elsewhere) in place of a status dot, and shows `terminal.cwd` as the discreet secondary line.

- [ ] **Step 3: Verify it type-checks**

Run: `deno task check`
Expected: completes successfully. (`+page.svelte` will have errors here because its props to `ProcessList` have not been updated yet — that is expected and fixed in Task 6. If `deno task check` errors ONLY on `+page.svelte` `ProcessList` props, proceed.)

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ProcessList.svelte
git commit -m "feat(sidebar): unify processes and terminals in a flat list"
```

---

## Task 5: Make TerminalPane header-less

**Files:**
- Modify: `src/lib/components/TerminalPane.svelte`

The pane currently has its own header (a "Terminal" label, the title, and a Close button). With the content header now showing the terminal title and close living in the sidebar + ⚙ menu, the pane's internal header is redundant. Remove it; the xterm host fills the whole pane.

- [ ] **Step 1: Remove the internal header markup**

In `src/lib/components/TerminalPane.svelte`, replace the entire `<section>...</section>` markup (from `<section class="flex h-full min-h-0 flex-col bg-canvas">` to the closing `</section>`) with:

```svelte
<section class="flex h-full min-h-0 flex-col bg-canvas">
  <div
    bind:this={host}
    class={`min-h-0 flex-1 overflow-hidden px-2 py-2 ${terminalId ? "" : "hidden"}`}
  ></div>
  {#if !terminalId}
    <div class="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 px-4 text-center">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor"
        stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-text-subtle" aria-hidden="true">
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </svg>
      <div class="text-sm text-text-subtle">Open a terminal from the project menu.</div>
    </div>
  {/if}
</section>
```

- [ ] **Step 2: Drop the now-unused `onClose` prop**

The `onClose` prop is no longer referenced in the markup. Update the `Props` type and destructuring to remove it. Replace:

```ts
  type Props = {
    terminalId: string | null;
    title: string | null;
    output: string;
    onInput: (data: string) => void;
    onResize: (cols: number, rows: number) => void;
    onClose: () => void;
  };

  let { terminalId, title, output, onInput, onResize, onClose }: Props = $props();
```

with:

```ts
  type Props = {
    terminalId: string | null;
    output: string;
    onInput: (data: string) => void;
    onResize: (cols: number, rows: number) => void;
  };

  let { terminalId, output, onInput, onResize }: Props = $props();
```

(`title` is also removed since it was only used by the deleted header.)

- [ ] **Step 3: Verify it type-checks**

Run: `deno task check`
Expected: completes successfully (errors only in `+page.svelte` passing the removed props are expected; fixed in Task 6).

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/TerminalPane.svelte
git commit -m "feat(terminal): make pane header-less, fill content body"
```

---

## Task 6: Rewrite +page.svelte — adaptive content, slim headers, ⚙ menu

**Files:**
- Modify: `src/routes/+page.svelte`

This is the integration task. Remove `runtimeView` and `SegmentedControl`. Make the content body render based on `runtimeStore.selection.kind`. Reduce the content header to title + status + ⚙ menu. Drop the path subtitle from the sidebar header. Wire the ⚙ menu to project/process/terminal actions.

- [ ] **Step 1: Replace the `<script>` block**

Replace the entire `<script lang="ts">...</script>` block with:

```svelte
<script lang="ts">
  import { onMount } from "svelte";

  import AppShell from "$lib/components/AppShell.svelte";
  import ConfigEditor from "$lib/components/ConfigEditor.svelte";
  import LogViewer from "$lib/components/LogViewer.svelte";
  import ProcessList from "$lib/components/ProcessList.svelte";
  import ProjectSettingsDialog from "$lib/components/ProjectSettingsDialog.svelte";
  import TerminalPane from "$lib/components/TerminalPane.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Menu from "$lib/components/ui/Menu.svelte";
  import type { MenuItem } from "$lib/components/ui/Menu.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import type { ProjectRecord } from "$lib/types";

  let detailsOpen = $state(false);
  let configOpen = $state(false);
  let editingProject = $state<ProjectRecord | null>(null);

  // Externally-callable log actions, registered by LogViewer.
  let logActions = $state<{ copy: () => void; clear: () => void } | null>(null);

  const project = $derived(runtimeStore.project);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);
  const selection = $derived(runtimeStore.selection);

  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const selectedTerminal = $derived(runtimeStore.selectedTerminal);

  onMount(() => {
    void runtimeStore.init();
    return () => {
      void runtimeStore.teardown();
    };
  });

  function openCreateDialog() {
    editingProject = null;
    detailsOpen = true;
  }

  function openEditDialog(openedProject: ProjectRecord) {
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

  // The ⚙ menu is contextual: project actions always; process actions when a
  // process is selected; terminal actions when a terminal is selected.
  const menuItems = $derived<MenuItem[]>(
    buildMenuItems(),
  );

  function buildMenuItems(): MenuItem[] {
    const items: MenuItem[] = [];

    // Project-scoped actions (always available when a project is registered).
    if (project) {
      items.push({ label: "Edit project", onSelect: () => openEditDialog(project) });
      items.push({ label: "Runtime config", onSelect: openConfigDialog, dividerAfter: true });
    }

    if (selection?.kind === "process" && selectedProcess) {
      const name = selectedProcess.name;
      items.push({ label: `Restart ${name}`, onSelect: () => runtimeStore.restartSessionProcess(name) });
      items.push({ label: `Stop ${name}`, onSelect: () => runtimeStore.stopSessionProcess(name), danger: true, dividerAfter: true });
      items.push({ label: "Copy logs", onSelect: () => logActions?.copy(), disabled: !logActions });
      items.push({ label: "Clear logs", onSelect: () => logActions?.clear(), disabled: !logActions, danger: true });
    }

    if (selection?.kind === "terminal" && selectedTerminal) {
      items.push({ label: `Close ${selectedTerminal.title}`, onSelect: () => runtimeStore.closeSelectedTerminal(), danger: true });
    }

    items.push({ label: "Open terminal", onSelect: openTerminal, disabled: !runtimeStore.projectId || runtimeStore.busy });

    return items;
  }
</script>
```

- [ ] **Step 2: Replace the `sidebarHeader` snippet**

Replace the entire `{#snippet sidebarHeader()} ... {/snippet}` block with the slimmed version (no baseDir subtitle):

```svelte
{#snippet sidebarHeader()}
      <div class="border-b border-border px-4 py-3">
        <div class="flex items-center justify-between gap-2">
          <div class="min-w-0">
            <div class="truncate text-sm font-semibold text-text">{project?.name ?? "devapp"}</div>
          </div>
          <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
            +
          </IconButton>
        </div>

        <div class="mt-2.5 flex items-center gap-1.5">
          {#if sessionActive}
            <button
              type="button"
              class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-md border border-danger/30 bg-danger/10 px-2 text-xs font-medium text-danger transition-colors hover:bg-danger/20 disabled:cursor-not-allowed disabled:opacity-55"
              onclick={() => runtimeStore.stopCurrentProject()}
              disabled={runtimeStore.busy}
              aria-label="Stop current run"
              title="Stop current run"
            >
              <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <rect x="6" y="6" width="12" height="12" rx="1.5" />
              </svg>
              Stop
            </button>
          {:else}
            <button
              type="button"
              class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-md bg-accent px-2 text-xs font-medium text-canvas transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:bg-surface-hover disabled:text-text-subtle"
              onclick={() => runtimeStore.startCurrentProject()}
              disabled={!runtimeStore.projectId || runtimeStore.busy}
              aria-label="Run project"
              title="Run project"
            >
              <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M8 5v14l11-7z" />
              </svg>
              Run
            </button>
          {/if}
        </div>
      </div>
{/snippet}
```

- [ ] **Step 3: Replace the `processList` snippet**

Replace the entire `{#snippet processList()} ... {/snippet}` block to pass the unified props:

```svelte
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
          onRestart={(processName) => runtimeStore.restartSessionProcess(processName)}
          onCloseTerminal={(terminalId) => {
            runtimeStore.selectTerminal(terminalId);
            runtimeStore.closeSelectedTerminal();
          }}
        />
      </section>
{/snippet}
```

- [ ] **Step 4: Replace the `contentHeader` snippet**

Replace the entire `{#snippet contentHeader()} ... {/snippet}` block with the slimmed title + status + ⚙ menu:

```svelte
{#snippet contentHeader()}
      <header class="flex h-12 min-w-0 items-center justify-between gap-4 border-b border-border px-4">
        <div class="min-w-0">
          {#if selection?.kind === "terminal" && selectedTerminal}
            <div class="truncate text-sm font-semibold text-text">{selectedTerminal.title}</div>
            <div class="truncate text-[11px] text-text-subtle">{selectedTerminal.cwd}</div>
          {:else if selectedProcess}
            <div class="truncate text-sm font-semibold text-text">{selectedProcess.name}</div>
            <div class="truncate text-[11px] text-text-subtle">{selectedProcess.status}</div>
          {:else}
            <div class="truncate text-sm font-semibold text-text">
              {project?.name ?? "devapp"}
            </div>
            <div class="truncate text-[11px] text-text-subtle">
              {project?.baseDir ?? "No project registered"}
            </div>
          {/if}
        </div>

        <div class="flex shrink-0 items-center gap-1">
          <Menu label="Project menu" items={menuItems} />
        </div>
      </header>
{/snippet}
```

- [ ] **Step 5: Replace the AppShell children (adaptive content body)**

Replace the `<AppShell ...> ... </AppShell>` block's children (the part between the opening `<AppShell` and closing `</AppShell>`, i.e. the `{#if runtimeView === "logs"} ... {/if}` block) with adaptive content driven by `selection`:

```svelte
<AppShell
  {sidebarHeader}
  {processList}
  {contentHeader}
>
        {#if selection?.kind === "terminal" && selectedTerminal}
          <TerminalPane
            terminalId={selectedTerminal.terminalId}
            output={runtimeStore.terminalOutput[selectedTerminal.terminalId] ?? ""}
            onInput={(data) => runtimeStore.writeToTerminal(data)}
            onResize={(cols, rows) => runtimeStore.resizeSelectedTerminal(cols, rows)}
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
```

- [ ] **Step 6: Verify it type-checks**

Run: `deno task check`
Expected: completes successfully with no errors. (If `IconButton` is now unused in this file, that's fine — the `+` register button still uses it. If `SegmentedControl` import was already removed in Step 1, there should be no leftover references.)

- [ ] **Step 7: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(app): adaptive content, unified sidebar, contextual menu"
```

---

## Task 7: Delete the now-unused SegmentedControl

**Files:**
- Delete: `src/lib/components/ui/SegmentedControl.svelte`

After Task 6, `SegmentedControl` has no importers. Remove it.

- [ ] **Step 1: Confirm no remaining references**

Run: `grep -rn "SegmentedControl" src/` (or the project's search tool)
Expected: no matches.

- [ ] **Step 2: Delete the file**

Delete `src/lib/components/ui/SegmentedControl.svelte`.

- [ ] **Step 3: Verify the build still type-checks**

Run: `deno task check`
Expected: completes successfully.

- [ ] **Step 4: Commit**

```bash
git add -A src/lib/components/ui/SegmentedControl.svelte
git commit -m "chore(ui): remove unused SegmentedControl"
```

---

## Task 8: Manual validation + final check

**Files:** none (validation only)

- [ ] **Step 1: Run the full check**

Run: `deno task check`
Expected: completes successfully.

- [ ] **Step 2: Launch the app against the example config**

Run: `deno task app examples/deno-runner.yml`
Expected: app window opens.

- [ ] **Step 3: Validate the sidebar**

Verify against the spec's success criteria:
- Sidebar shows processes (after starting) with dot + name + status text, no accent bar, no kind badge.
- Opening a terminal via the ⚙ menu adds it to the sidebar (below processes), with a terminal icon and a `bash` / `bash 2` title.
- Selecting a process highlights it with a raised background (no accent bar).
- Selecting a terminal highlights it; the previously selected process de-selects (mutual exclusion).

- [ ] **Step 4: Validate the content pane**

- With a process selected: logs render; toolbar shows only search + autoscroll + pause (no kbd hint, no counter, no copy/clear buttons).
- With a terminal selected: xterm renders full-height, no internal header.
- The `/` keyboard shortcut still focuses the log search when a process is selected.
- Switching between a process and a terminal swaps the content pane (no toggle control anywhere).

- [ ] **Step 5: Validate the ⚙ menu**

Open the ⚙ menu and confirm it is contextual:
- Always shows "Edit project" + "Runtime config" (when a project is registered).
- With a process selected: shows Restart / Stop / Copy logs / Clear logs / Open terminal.
- With a terminal selected: shows Close + Open terminal.
- "Copy logs" and "Clear logs" actually copy/clear (verify clipboard and that the log area empties).
- "Open terminal" creates a new terminal and selects it.
- Click-away closes the menu; Escape closes it.

- [ ] **Step 6: Validate stop/run and error states**

- The sidebar Run button starts the session; Stop stops it.
- Triggering a UI error (e.g. stop with no session, or a failing process) shows the dismissible error banner in the sidebar.
- The empty state (no session, no selection) shows the correct message.

- [ ] **Step 7: Commit any cleanup**

If manual validation surfaced small fixes, commit them:

```bash
git add -A
git commit -m "fix(ui): validation tweaks from manual testing"
```

If no fixes were needed, no commit — you're done.

---

## Notes for the implementer

- **No frontend tests exist.** Do not invent a test runner. Validation is `deno task check` (svelte-check) plus the manual steps in Task 8.
- **Svelte 5 runes everywhere.** Use `$state`, `$derived`, `$effect`, `$props`. Match the existing style in the files you touch.
- **Tailwind tokens** (`bg-canvas`, `text-text`, `border-border`, `bg-surface-raised`, etc.) are defined in `src/app.css` under `@theme`. Do not introduce new tokens.
- **AGENTS.md**: this project is **not** a git repo in this environment, so the `git add` / `git commit` steps are instructions for a real environment; if git is unavailable, skip commits — the file edits are what matter.
- **The `tsconfig.json` extension warning** from SvelteKit is expected and non-blocking; only a non-zero exit counts as failure.
