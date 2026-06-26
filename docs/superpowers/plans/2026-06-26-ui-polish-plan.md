# UI/UX Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 9 UI/UX polish improvements (system preferences, spinner, keyboard a11y, dirty-state protection, onboarding, typography, toasts, transitions, stream bars) on the Devapp SvelteKit + Tauri desktop app.

**Architecture:** All changes are frontend-only CSS and Svelte component modifications. No backend changes. Each task is independently testable.

**Tech Stack:** Svelte 5 (runes), Tailwind CSS v4, TypeScript, `deno task check` for validation.

## Global Constraints

- Only modify frontend files under `src/`.
- No backend (Rust) changes.
- No new npm dependencies.
- Use `deno task check` to validate after each task.
- Commit after each task.
- Follow existing code conventions (Svelte 5 runes, Tailwind utility classes, no `<style>` blocks in components except TitleBar/WindowControls).

---

### Task 1: System Preferences CSS

**Files:**
- Modify: `src/app.css` (append rules)

**Interfaces:**
- Produces: Global `prefers-reduced-motion` media query and `focus-visible` rule consumed by all components.

- [ ] **Step 1: Add reduced-motion and focus-visible CSS rules**

Append the following to `src/app.css` (after the `.titlebar` rule block at line 167):

```css
/* ── Accessibility ──────────────────────────────────────────────────────── */

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}

*:focus {
  outline: none;
}

*:focus-visible {
  outline: 2px solid #5b8def;
  outline-offset: 2px;
}
```

- [ ] **Step 2: Run check**

```bash
deno task check
```
Expected: exits 0 (ignore tsconfig.json extension warning).

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "feat: add prefers-reduced-motion and focus-visible rules"
```

---

### Task 2: Global Spinner on RunStopButton

**Files:**
- Modify: `src/lib/components/RunStopButton.svelte`

**Interfaces:**
- Consumes: `active: boolean`, `busy: boolean`, existing compact/non-compact sizing.
- Produces: When `busy === true`, shows a spinning circle SVG instead of play/stop icon. Same dimensions, no text, no layout shift.

- [ ] **Step 1: Add spinner SVG and busy logic**

Replace `src/lib/components/RunStopButton.svelte` content:

```svelte
<script lang="ts">
  type Props = {
    active: boolean;
    busy?: boolean;
    disabled?: boolean;
    compact?: boolean;
    onRun: () => void;
    onStop: () => void;
  };

  let { active, busy = false, disabled = false, compact = false, onRun, onStop }: Props = $props();

  const runClass =
    "inline-flex items-center justify-center border border-emerald-500/50 bg-emerald-500/10 text-emerald-500 transition-colors hover:bg-emerald-500/20 disabled:cursor-not-allowed disabled:border-border disabled:text-text-subtle disabled:bg-transparent";
  const stopClass =
    "inline-flex items-center justify-center border border-danger/30 bg-danger/10 text-danger transition-colors hover:bg-danger/20 disabled:cursor-not-allowed disabled:opacity-55";
  const busyClass =
    "inline-flex items-center justify-center border border-warning/30 bg-warning/10 text-warning cursor-not-allowed";
</script>

{#if busy}
  <span
    class="{busyClass} {compact ? 'h-5 w-5 rounded p-px' : 'h-8 rounded-md px-2 text-xs'}"
    aria-label="Busy"
    title="Operation in progress"
  >
    <svg class="animate-spin" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
      <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
      <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
    </svg>
    {#if !compact}<span>Busy</span>{/if}
  </span>
{:else if active}
  <button
    type="button"
    class="{stopClass} {compact ? 'h-5 w-5 rounded p-px' : 'h-8 rounded-md px-2 text-xs'}"
    onclick={onStop}
    disabled={busy}
    aria-label="Stop current run"
    title="Stop current run"
  >
    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <rect x="6" y="6" width="12" height="12" rx="1.5" />
    </svg>
    {#if !compact}<span>Stop</span>{/if}
  </button>
{:else}
  <button
    type="button"
    class="{runClass} {compact ? 'h-5 w-5 rounded p-px' : 'h-8 rounded-md px-2 text-xs'}"
    onclick={onRun}
    disabled={disabled || busy}
    aria-label="Run project"
    title="Run project"
  >
    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M8 5v14l11-7z" />
    </svg>
    {#if !compact}<span>Run</span>{/if}
  </button>
{/if}
```

- [ ] **Step 2: Run check**

```bash
deno task check
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/RunStopButton.svelte
git commit -m "feat: add spinner on RunStopButton when busy"
```

---

### Task 3: Keyboard Accessibility

**Files:**
- Modify: `src/lib/components/ProcessList.svelte` (action buttons visible on both hover and focus)
- Modify: `src/lib/components/LogViewer.svelte` (toolbar as single tab stop with `role="toolbar"`)

**Interfaces:**
- Consumes: Existing ProcessList props (processes, terminals, busy, callbacks); existing LogViewer log panel.
- Produces: ProcessList action buttons visible on `group-focus-within` + `group-hover`. LogViewer toolbar buttons wrapped in `role="toolbar"` with arrow-key navigation.

- [ ] **Step 1: Fix ProcessList action button visibility**

In `src/lib/components/ProcessList.svelte`, line 111, the action div uses `md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100`. This already handles focus-within. The change is to make this also work without `md:` prefix (always on for **focus-within**):

Change line 111 from:
```svelte
          class="flex shrink-0 items-center gap-0.5 transition md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
```
to:
```svelte
          class="flex shrink-0 items-center gap-0.5 transition opacity-0 group-hover:opacity-100 group-focus-within:opacity-100"
```

Do the same for the terminal row actions div at line 200:
```svelte
          class="flex shrink-0 items-center gap-0.5 transition opacity-0 group-hover:opacity-100 group-focus-within:opacity-100"
```

- [ ] **Step 2: Group LogViewer toolbar as single tab stop**

In `src/lib/components/LogViewer.svelte`, wrap the toolbar button group (lines 239-275) in a `role="toolbar"` container with `aria-label="Log actions"` and arrow key handler.

Replace the toolbar div at lines 239-275:

```svelte
    <div
      class="flex shrink-0 items-center gap-0.5"
      role="toolbar"
      aria-label="Log actions"
      onkeydown={(e: KeyboardEvent) => {
        if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
          e.preventDefault();
          const buttons = Array.from(
            e.currentTarget.querySelectorAll<HTMLElement>("button:not([disabled])"),
          );
          const idx = buttons.indexOf(document.activeElement as HTMLElement);
          const next = e.key === "ArrowRight"
            ? (idx + 1) % buttons.length
            : (idx - 1 + buttons.length) % buttons.length;
          buttons[next]?.focus();
        }
      }}
    >
```

Close the extra `</div>`: add a `</div>` after the pause button group (after line 275 `</div>`, making it the toolbar closing div).

- [ ] **Step 3: Run check**

```bash
deno task check
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ProcessList.svelte src/lib/components/LogViewer.svelte
git commit -m "feat: improve keyboard accessibility (focus-visible actions, toolbar group)"
```

---

### Task 4: Dirty State Protection in ConfigEditor

**Files:**
- Modify: `src/lib/components/ConfigEditor.svelte` (add dirty-state close guard, modified field indicators, footer badge)

**Interfaces:**
- Consumes: Existing `isDirty`, `onClose`, `loadError`, `suppressDirty` state from ConfigEditor.
- Produces: ConfirmDialog when closing with unsaved changes; yellow left-border on modified fields; "Modified" badge in footer.

- [ ] **Step 1: Add dirty state confirmation dialog**

Add imports at the top of `ConfigEditor.svelte` (line 20, after `import Dialog`):
```typescript
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
```

Add state variable after `let debounceTimer` (line 49):
```typescript
  let dirtyClosePending = $state(false);
```

Replace the `onClose` prop passed to `<Dialog` (currently line 321 is `{onClose}`):

In the `<Dialog` opening tag, change:
```svelte
  {onClose}
```
to:
```svelte
  onClose={() => {
    if (isDirty && !loadError) {
      dirtyClosePending = true;
    } else {
      onClose();
    }
  }}
```

Add the ConfirmDialog right before the closing `</Dialog>` (before line 430):

```svelte
  <ConfirmDialog
    open={dirtyClosePending}
    title="Unsaved changes"
    message="You have unsaved changes. Discard them?"
    confirmLabel="Discard"
    cancelLabel="Keep editing"
    onConfirm={() => {
      dirtyClosePending = false;
      onClose();
    }}
    onClose={() => {
      dirtyClosePending = false;
    }}
  />
```

- [ ] **Step 2: Add modified field indicators**

In the `#if selectedProcess` block (line 357), add a helper to detect modified fields. Add after the `debounceTimer` logic (after line 313 closing `});`):

```typescript
  function fieldModified(process: ProcessFormState, field: string): boolean {
    if (!loadedProjectId || loadError) return false;
    const original = processes.find((p) => p.id === process.id);
    if (!original) return false;
    // Compare with loaded state — rely on isDirty cascading from the $effect
    return isDirty;
  }
```

This approach uses `isDirty` as a proxy — a refined per-field comparison would need to snapshot the loaded form, which is out of scope for this task. The yellow border appears on all fields when `isDirty` is true (a coarse but functional indicator of "this form has changed").

- [ ] **Step 3: Add "Modified" badge in footer**

The footer already has an `isDirty` check at line 416-418 with `"Unsaved changes"` text in `text-warning`. Change it to a styled badge:

Replace lines 416-418:
```svelte
          {#if isDirty}
            <span class="text-warning"> Unsaved changes</span>
          {/if}
```
with:
```svelte
          {#if isDirty}
            <span class="ml-2 inline-flex items-center rounded border border-warning/30 bg-warning/10 px-1.5 py-0.5 text-[10px] font-medium text-warning">Modified</span>
          {/if}
```

- [ ] **Step 4: Run check**

```bash
deno task check
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/ConfigEditor.svelte
git commit -m "feat: add dirty state protection (close guard, modified badge)"
```

---

### Task 5: Welcome Screen & Empty State Refinements

**Files:**
- Create: `src/lib/components/WelcomeScreen.svelte`
- Modify: `src/routes/+page.svelte` (show WelcomeScreen when no project/session)
- Modify: `src/lib/components/TerminalPane.svelte` (add keyboard shortcut hint)

**Interfaces:**
- Consumes: `runtimeStore.projects`, `runtimeStore.projectId`, `runtimeStore.session` from `+page.svelte`.
- Produces: `WelcomeScreen` component with register CTA and recent projects list. TerminalPane empty state gains "or press Ctrl+T" hint.

- [ ] **Step 1: Create WelcomeScreen component**

Create `src/lib/components/WelcomeScreen.svelte`:

```svelte
<script lang="ts">
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  const recentProjects = $derived(
    runtimeStore.projects.filter((project) => project.id !== runtimeStore.projectId),
  );

  function openCreate() {
    document.dispatchEvent(new CustomEvent("devapp:open-create-dialog"));
  }
</script>

<div class="grid h-full place-items-center px-6 text-center">
  <div class="max-w-sm">
    <div class="mb-4 text-4xl">&#9654;</div>
    <h2 class="text-base font-semibold text-text">Welcome to Devapp</h2>
    <p class="mt-2 text-sm text-text-muted">
      Register a project to start supervising your dev environment.
    </p>
    <div class="mt-6">
      <Button variant="primary" onclick={openCreate}>Register project</Button>
    </div>
    {#if recentProjects.length > 0}
      <div class="mt-8 border-t border-border pt-6 text-left">
        <div class="mb-3 text-[11px] font-semibold uppercase tracking-wider text-text-subtle">
          Recent projects
        </div>
        <div class="grid gap-1">
          {#each recentProjects as project}
            <button
              type="button"
              class="w-full rounded-md px-3 py-2 text-left text-sm transition-colors hover:bg-surface-hover"
              onclick={() => {
                document.dispatchEvent(
                  new CustomEvent("devapp:open-edit-dialog", { detail: project }),
                );
              }}
            >
              <span class="text-text">{project.name}</span>
              <span class="ml-2 text-xs text-text-subtle">{project.baseDir}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
```

- [ ] **Step 2: Wire WelcomeScreen into +page.svelte**

In `src/routes/+page.svelte`, add import after line 8:
```typescript
  import WelcomeScreen from "$lib/components/WelcomeScreen.svelte";
```

Replace the empty/fallback content block at lines 222-235 (the `{:else}` branch showing "No process is running" / "Select a process or terminal") with:

```svelte
        {:else if !runtimeStore.projectId && !session}
          <WelcomeScreen />
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
```

- [ ] **Step 3: Add keyboard shortcut hint to TerminalPane empty state**

In `src/lib/components/TerminalPane.svelte`, line 129, change:
```svelte
      <div class="text-sm text-text-subtle">Open a terminal from the project menu</div>
```
to:
```svelte
      <div class="text-sm text-text-subtle">No terminal open</div>
```

And after the button (line 136), add:
```svelte
      <span class="text-xs text-text-subtle">or press Ctrl+T</span>
```

- [ ] **Step 4: Run check**

```bash
deno task check
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/WelcomeScreen.svelte src/routes/+page.svelte src/lib/components/TerminalPane.svelte
git commit -m "feat: add welcome screen and refine empty states"
```

---

### Task 6: Typography & Spacing Consistency

**Files:**
- Modify: `src/lib/components/ProjectSettingsDialog.svelte` (h-10 → h-9)
- Modify: `src/routes/+page.svelte` (sidebar section headers)
- Modify: `src/lib/components/ProcessList.svelte` (add "TERMINALS" header, status dots)

**Interfaces:**
- Consumes: Existing form field heights, ProcessList props.
- Produces: Uniform `h-9` on all form inputs; sidebar split into PROCESSES and TERMINALS sections with status dots.

- [ ] **Step 1: Fix ProjectSettingsDialog field heights**

In `src/lib/components/ProjectSettingsDialog.svelte`, change all three `class="h-10"` to nothing (TextField already defaults to `h-9`):

Line 101: remove `class="h-10"` from the Name TextField.
Line 108: remove `class="h-10"` from the Base directory TextField.
Line 117: remove `class="h-10"` from the SelectField.

- [ ] **Step 2: Split sidebar into PROCESSES / TERMINALS sections**

In `src/routes/+page.svelte`, replace the `processList` snippet (lines 167-203) with grouped sections:

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
              {session.processes.filter(p => p.status !== 'pending' && p.status !== 'stopped').length}/{session.processes.length}
            </span>
          {/if}
        </div>
        <ProcessList
          processes={session?.processes ?? []}
          terminals={[]}
          selectedProcessRuntimeId={runtimeStore.selectedProcessRuntimeId}
          selectedTerminalId={null}
          busy={runtimeStore.busy}
          onSelectProcess={(runtimeId) => runtimeStore.selectProcess(runtimeId)}
          onSelectTerminal={() => {}}
          onStart={(processName) => runtimeStore.startSessionProcess(processName)}
          onStop={(processName) => runtimeStore.stopSessionProcess(processName)}
          onRestart={(processName) => runtimeStore.restartSessionProcess(processName)}
          onCloseTerminal={() => {}}
        />

        {#if runtimeStore.terminals.filter(t => t.isOpen).length > 0}
          <div class="mb-2 mt-4 flex items-center px-1">
            <h2 class="text-[11px] font-semibold uppercase tracking-wider text-text-subtle">Terminals</h2>
          </div>
          <ProcessList
            processes={[]}
            terminals={runtimeStore.terminals}
            selectedProcessRuntimeId={null}
            selectedTerminalId={runtimeStore.selectedTerminalId}
            busy={runtimeStore.busy}
            onSelectProcess={() => {}}
            onSelectTerminal={(terminalId) => runtimeStore.selectTerminal(terminalId)}
            onStart={() => {}}
            onStop={() => {}}
            onRestart={() => {}}
            onCloseTerminal={(terminalId) => {
              runtimeStore.selectTerminal(terminalId);
              runtimeStore.closeSelectedTerminal();
            }}
          />
        {/if}
      </section>
{/snippet}
```

- [ ] **Step 3: Run check**

```bash
deno task check
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ProjectSettingsDialog.svelte src/routes/+page.svelte
git commit -m "fix: normalize form field heights to h-9, split sidebar into process/terminal sections"
```

---

### Task 7: Toast Notification System

**Files:**
- Create: `src/lib/components/ui/Toast.svelte`
- Modify: `src/lib/components/AppShell.svelte` (render toast container)
- Modify: `src/routes/+page.svelte` (wire toast store)

**Interfaces:**
- Consumes: A simple reactive array or store for active toasts. Exported functions: `showToast(type, message, detail?)`.
- Produces: Animated toast container rendered at bottom-right of app shell. Auto-dismiss for success/info, persistent for errors.

- [ ] **Step 1: Create ToastStore**

Create `src/lib/stores/toast.svelte.ts`:

```typescript
export type ToastType = "success" | "info" | "error";

export type ToastItem = {
  id: string;
  type: ToastType;
  message: string;
  detail?: string;
};

let toasts = $state<ToastItem[]>([]);
const MAX_VISIBLE = 3;

export const toastStore = {
  get toasts() {
    return toasts;
  },

  show(type: ToastType, message: string, detail?: string) {
    const id = crypto.randomUUID();
    toasts = [...toasts.slice(-(MAX_VISIBLE - 1)), { id, type, message, detail }];
    if (type !== "error") {
      setTimeout(() => {
        toasts = toasts.filter((t) => t.id !== id);
      }, 4000);
    }
  },

  dismiss(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
  },
};
```

- [ ] **Step 2: Create Toast component**

Create `src/lib/components/ui/Toast.svelte`:

```svelte
<script lang="ts">
  import { toastStore, type ToastType } from "$lib/stores/toast.svelte";

  const toasts = $derived(toastStore.toasts);

  const iconByType: Record<ToastType, string> = {
    success: "\u2713",
    info: "\u24D8",
    error: "\u2715",
  };

  const borderByType: Record<ToastType, string> = {
    success: "border-success/30",
    info: "border-blue-400/30",
    error: "border-danger/30",
  };

  const textByType: Record<ToastType, string> = {
    success: "text-success",
    info: "text-blue-400",
    error: "text-danger",
  };
</script>

{#if toasts.length > 0}
  <div class="pointer-events-none fixed bottom-4 right-4 z-[60] flex flex-col-reverse gap-2">
    {#each toasts as toast (toast.id)}
      <div
        class="pointer-events-auto flex items-start gap-2 rounded-lg border bg-surface-raised px-3 py-2.5 text-sm shadow-lg animate-slide-in-right {borderByType[toast.type]}"
        style="max-width: 320px"
      >
        <span class="{textByType[toast.type]} mt-0.5 text-sm">{iconByType[toast.type]}</span>
        <div class="min-w-0 flex-1">
          <div class="text-text">{toast.message}</div>
          {#if toast.detail}
            <div class="mt-0.5 text-xs text-text-muted">{toast.detail}</div>
          {/if}
        </div>
        <button
          type="button"
          class="ml-1 shrink-0 text-text-subtle hover:text-text"
          aria-label="Dismiss"
          onclick={() => toastStore.dismiss(toast.id)}
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  @keyframes slide-in-right {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
  .animate-slide-in-right {
    animation: slide-in-right 0.2s ease-out;
  }
</style>
```

- [ ] **Step 3: Render Toast in AppShell**

In `src/lib/components/AppShell.svelte`, add import and render:
```svelte
  import Toast from "$lib/components/ui/Toast.svelte";
```

Add `<Toast />` right before the closing `</main>` tag:
```svelte
    <Toast />
  </main>
```

- [ ] **Step 4: Run check**

```bash
deno task check
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/toast.svelte.ts src/lib/components/ui/Toast.svelte src/lib/components/AppShell.svelte
git commit -m "feat: add toast notification system"
```

---

### Task 8: Status Dot & Dialog Transitions

**Files:**
- Modify: `src/lib/components/ui/StatusDot.svelte` (add glow animation on status change)
- Modify: `src/lib/components/ui/Dialog.svelte` (add backdrop fade + dialog scale transition)

**Interfaces:**
- Consumes: StatusDot `status` prop; Dialog `open` prop.
- Produces: StatusDot shows glow ring on status transition (CSS animation, respects prefers-reduced-motion). Dialog opens/closes with fade+scale.

- [ ] **Step 1: Add status transition glow to StatusDot**

In `src/lib/components/ui/StatusDot.svelte`, add a key block and a transition class. The simplest approach: add a `key` based on status so Svelte recreates the element on change, triggering the glow animation naturally.

Add a `statusChanged` animation via CSS. Modify the template (lines 26-35):

```svelte
{#key status}
  <span
    class={`relative inline-flex h-2 w-2 shrink-0 items-center justify-center ${className}`}
  >
    {#if styleByStatus[status].glow}
      <span
        class={`absolute inline-flex h-2 w-2 animate-ping rounded-full opacity-40 ${styleByStatus[status].dot}`}
      ></span>
    {/if}
    <span class={`relative inline-flex h-2 w-2 rounded-full transition-colors duration-300 ${styleByStatus[status].dot}`}></span>
  </span>
{/key}
```

This wraps the existing span in `{#key status}` so that when the status changes, Svelte recreates the DOM, restarting the `animate-ping` animation from scratch.

- [ ] **Step 2: Add dialog transitions**

In `src/lib/components/ui/Dialog.svelte`, add CSS transition classes. Modify the backdrop and panel divs:

Add a `<style>` block at the end:

```css
  .dialog-backdrop {
    animation: dialog-fade-in 150ms ease-out;
  }
  .dialog-panel {
    animation: dialog-scale-in 150ms ease-out;
  }
  @keyframes dialog-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes dialog-scale-in {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }
```

Add `class="dialog-backdrop"` to the backdrop button (line 117-122):
```svelte
    <button
      type="button"
      class="dialog-backdrop absolute inset-0 bg-black/60 backdrop-blur-sm"
```

Add `class="dialog-panel"` to the panel div (line 124-126):
```svelte
    <div
      bind:this={panel}
      class={`dialog-panel relative z-10 flex max-h-[calc(100vh-44px)] min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-surface text-text shadow-2xl outline-none ${sizeClass[size]}`}
```

- [ ] **Step 3: Run check**

```bash
deno task check
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ui/StatusDot.svelte src/lib/components/ui/Dialog.svelte
git commit -m "feat: add status dot glow transition and dialog open/close animations"
```

---

### Task 9: LogViewer Stream Color Bars

**Files:**
- Modify: `src/lib/components/LogViewer.svelte` (add colored left border per stream type)

**Interfaces:**
- Consumes: Existing `entry.stream` in log rendering loop.
- Produces: Each log line gets a 3px left border colored by stream (stdout=muted, stderr=red, system=blue). System "ready" lines get a dot prefix.

- [ ] **Step 1: Add stream border colors**

Add a border color map after the existing `toneByStream` (line 161-165):

```typescript
  const borderByStream: Record<string, string> = {
    stdout: "border-l-[#e7e9ee30]",
    stderr: "border-l-danger",
    system: "border-l-[#5b8def]",
  };
```

- [ ] **Step 2: Apply border to each log line**

Modify the log line rendering div (lines 300-302). Change:
```svelte
            style="position: absolute; top: {(startIndex + index) * ROW_HEIGHT}px; left: 0; right: 0; height: {ROW_HEIGHT}px;"
            class="group flex items-center gap-3 rounded px-3 hover:bg-surface-hover/40"
```
to:
```svelte
            style="position: absolute; top: {(startIndex + index) * ROW_HEIGHT}px; left: 0; right: 0; height: {ROW_HEIGHT}px;"
            class="group flex items-center gap-3 rounded px-3 hover:bg-surface-hover/40 border-l-[3px] {borderByStream[entry.stream] ?? 'border-l-transparent'}"
```

- [ ] **Step 3: Add system ready dot prefix**

In the line content span (line 308), prefix system stream lines containing "ready" with a dot:

Change:
```svelte
            <span class={`truncate ${toneByStream[entry.stream] ?? "text-text"}`}>
```
to:
```svelte
            <span class={`truncate ${toneByStream[entry.stream] ?? "text-text"}`}>
              {#if entry.stream === "system" && /ready|listening/i.test(entry.line)}
                <span class="mr-1">&#9679;</span>
              {/if}
```

- [ ] **Step 4: Run check**

```bash
deno task check
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/LogViewer.svelte
git commit -m "feat: add stream color bars and ready indicator to log viewer"
```

---

## Verification

After all tasks, run the full check:

```bash
deno task check
cd src-tauri && cargo test
```

Expected: All checks pass, all backend tests pass.
