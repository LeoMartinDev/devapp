# Settings A1 Minimal Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the full-page settings editor around the approved A1 continuous inspector layout: quieter navigation, a single vertical process sheet, compact secondary actions, neutral icon-only remove controls, and a stable `Save` CTA.

**Architecture:** This is a frontend-only polish pass on the existing settings flow. The work keeps the current data model, touched-field validation, and save behavior, but redistributes emphasis across the page: lighter wrappers, smaller controls, and shared icon-only remove affordances. The implementation is split into small UI tasks so each visual slice can be tested and verified independently.

**Tech Stack:** Svelte 5 runes, SvelteKit, TypeScript, Vitest, Testing Library, Tailwind CSS v4, `deno task test`, `deno task check`

---

## File Map

- `src/lib/components/ui/IconButton.svelte`
  Shared icon-only button primitive. Extend it so settings remove actions can be neutral, compact, and borderless at rest.

- `src/lib/components/ui/IconButton.test.ts`
  New focused regression tests for compact ghost icon buttons.

- `src/lib/components/ProcessForm.svelte`
  Selected-process identity block. This becomes the top of the continuous process sheet and swaps the loud remove CTA for an icon-only action.

- `src/lib/components/ProcessForm.test.ts`
  New regression test proving the selected-process remove control is icon-only and still accessible.

- `src/lib/components/EnvEditor.svelte`
  Environment row editor. Quiet header, compact add button, neutral icon-only row removal.

- `src/lib/components/EnvEditor.test.ts`
  New regression test covering the environment row removal control label and the absence of visible `Remove` copy.

- `src/lib/components/DependencyEditor.svelte`
  Dependency row editor. Same visual language as environment rows.

- `src/lib/components/DependencyEditor.test.ts`
  New regression test covering the dependency row removal control label and icon-only presentation.

- `src/lib/components/ReadyCheckEditor.svelte`
  Readiness subsection. Remove the nested-card chrome so it reads like another subsection in the same vertical sheet.

- `src/lib/components/ReadyCheckEditor.test.ts`
  New regression test locking in the flattened shell.

- `src/lib/components/ConfigEditor.svelte`
  Owns the page shell, left rail, global section wrappers, footer status, and the selected process composition.

- `src/lib/components/ConfigEditor.test.ts`
  Existing page-level test file. Extend it with the `Save`-label regression for the redesigned footer.

## Global Constraints

- Frontend only. No Rust, schema, or runtime-store behavior changes.
- Preserve the full-page settings route and the existing left-nav section behavior.
- Preserve current save semantics, touched-field validation rules, and dirty-state logic; only the visual treatment changes.
- Prefer the existing `IconButton` primitive over inventing a new settings-only control.
- Keep add actions compact and secondary. Keep remove actions icon-only and neutral at rest.
- Run targeted tests after each task, then `deno task check`.
- Treat the known SvelteKit `tsconfig.json` extension warning as non-blocking if the command exits successfully.
- Commit after each task.

---

### Task 1: Shared Quiet Icon Button Primitive

**Files:**
- Create: `src/lib/components/ui/IconButton.test.ts`
- Modify: `src/lib/components/ui/IconButton.svelte`

**Interfaces:**
- Consumes: existing `label`, `variant`, `children` props.
- Produces: `size?: "sm" | "md"` and a ghost icon button that stays neutral and borderless at rest, suitable for settings remove actions.

- [ ] **Step 1: Write the failing test for compact ghost icon buttons**

Create `src/lib/components/ui/IconButton.test.ts` with:

```ts
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";

import IconButton from "./IconButton.svelte";

describe("IconButton", () => {
  it("supports a compact ghost remove control without a resting border", () => {
    const { getByRole } = render(IconButton, {
      props: {
        label: "Remove variable EXAMPLE_ENV",
        variant: "ghost",
        size: "sm",
      },
    });

    const button = getByRole("button", { name: "Remove variable EXAMPLE_ENV" });

    expect(button.className).toContain("h-7");
    expect(button.className).toContain("w-7");
    expect(button.className).not.toContain("border");
  });
});
```

- [ ] **Step 2: Run the focused test and confirm it fails**

Run:

```bash
deno task test -- src/lib/components/ui/IconButton.test.ts
```

Expected: FAIL because `IconButton.svelte` does not yet support `size="sm"`, so the rendered class list still contains `h-8 w-8`.

- [ ] **Step 3: Implement the compact ghost icon-button primitive**

Replace `src/lib/components/ui/IconButton.svelte` with:

```svelte
<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  type Variant = "primary" | "secondary" | "danger" | "ghost";
  type Size = "sm" | "md";

  type Props = HTMLButtonAttributes & {
    label: string;
    variant?: Variant;
    size?: Size;
    children?: Snippet;
  };

  let {
    label,
    variant = "secondary",
    size = "md",
    disabled = false,
    type = "button",
    title,
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const variantClass: Record<Variant, string> = {
    primary: "bg-accent text-canvas hover:bg-accent-hover disabled:bg-surface-hover",
    secondary:
      "border border-border bg-surface-raised text-text-muted hover:bg-surface-hover hover:text-text disabled:text-text-subtle",
    danger: "border border-danger/30 text-danger hover:bg-danger/10 disabled:text-text-subtle",
    ghost: "text-text-subtle hover:bg-surface-hover hover:text-text disabled:text-text-subtle",
  };

  const sizeClass: Record<Size, string> = {
    sm: "h-7 w-7 rounded-[8px]",
    md: "h-8 w-8 rounded-md",
  };
</script>

<button
  {...rest}
  {type}
  {disabled}
  aria-label={label}
  title={title ?? label}
  class={`grid place-items-center transition-colors duration-75 disabled:cursor-not-allowed ${variantClass[variant]} ${sizeClass[size]} ${className}`}
>
  {@render children?.()}
</button>
```

- [ ] **Step 4: Re-run the focused test and the frontend check**

Run:

```bash
deno task test -- src/lib/components/ui/IconButton.test.ts
deno task check
```

Expected: both commands exit successfully.

- [ ] **Step 5: Commit the primitive change**

```bash
git add src/lib/components/ui/IconButton.svelte src/lib/components/ui/IconButton.test.ts
git commit -m "feat: add compact ghost icon button variant for settings controls"
```

---

### Task 2: Flatten the Selected Process Identity Block

**Files:**
- Create: `src/lib/components/ProcessForm.test.ts`
- Modify: `src/lib/components/ProcessForm.svelte`

**Interfaces:**
- Consumes: `process`, `processCount`, `processIssue`, `onRemove`, `onFieldBlur`.
- Produces: a quieter process header, icon-only remove action, and default compact field heights in a vertical-reading layout.

- [ ] **Step 1: Write the failing ProcessForm regression test**

Create `src/lib/components/ProcessForm.test.ts` with:

```ts
import { describe, expect, it, vi } from "vitest";
import { render } from "@testing-library/svelte";

import ProcessForm from "./ProcessForm.svelte";
import type { ProcessForm as ProcessFormState } from "$lib/config/editorModel";

const process: ProcessFormState = {
  id: "process-1",
  name: "api",
  kind: "service",
  cmd: "deno task dev",
  envRows: [],
  dependencies: [],
  readyEnabled: false,
  readyType: "http",
  httpUrl: "http://localhost:3000",
  logPattern: "ready",
  logRegex: false,
  delayDurationMs: 1000,
  commandCmd: "",
  intervalMs: null,
  timeoutMs: 60000,
};

describe("ProcessForm", () => {
  it("uses an icon-only remove action for the selected process", () => {
    const { getByRole, queryByText } = render(ProcessForm, {
      props: {
        process,
        processCount: 2,
        processIssue: () => null,
        onRemove: vi.fn(),
      },
    });

    expect(getByRole("button", { name: "Remove process" })).toBeInTheDocument();
    expect(queryByText("Remove process")).toBeNull();
  });
});
```

- [ ] **Step 2: Run the focused test and confirm it fails**

Run:

```bash
deno task test -- src/lib/components/ProcessForm.test.ts
```

Expected: FAIL because `ProcessForm.svelte` still renders a visible `Remove process` button label.

- [ ] **Step 3: Replace the loud process header with a quiet vertical header**

Replace `src/lib/components/ProcessForm.svelte` with:

```svelte
<script lang="ts">
  import type { ProcessForm as ProcessFormState } from "$lib/config/editorModel";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import SelectField from "$lib/components/ui/SelectField.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";

  type Props = {
    process: ProcessFormState;
    processCount: number;
    processIssue: (process: ProcessFormState, field: string) => string | null;
    onRemove: (id: string) => void;
    onFieldBlur?: (key: string) => void;
  };

  let { process, processCount, processIssue, onRemove, onFieldBlur }: Props = $props();
</script>

<section class="grid gap-4 border-t border-border/70 pt-5">
  <div class="flex items-start justify-between gap-3">
    <div>
      <h3 class="text-sm font-semibold text-text">Process</h3>
      <p class="mt-1 text-xs leading-5 text-text-subtle">Name, command, and kind for the selected runtime node.</p>
    </div>
    <IconButton
      label="Remove process"
      variant="ghost"
      size="sm"
      onclick={() => onRemove(process.id)}
      disabled={processCount <= 1}
    >
      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
        <path d="M6 2.75h4M3.75 4.5h8.5M6.5 6.5v5M9.5 6.5v5M5.5 2.75l.35-.6A.75.75 0 0 1 6.5 1.75h3a.75.75 0 0 1 .65.4l.35.6m-6 1.75V12a1.25 1.25 0 0 0 1.25 1.25h4.5A1.25 1.25 0 0 0 11.5 12V4.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </IconButton>
  </div>

  <div class="grid grid-cols-1 gap-3 md:grid-cols-[220px_minmax(0,1fr)]">
    <TextField
      label="Name"
      error={processIssue(process, "name")}
      bind:value={process.name}
      onblur={() => onFieldBlur?.(`process.${process.id}.name`)}
    />
    <TextField
      label="Command"
      placeholder="deno task dev"
      monospace
      error={processIssue(process, "cmd")}
      bind:value={process.cmd}
      onblur={() => onFieldBlur?.(`process.${process.id}.cmd`)}
    />
  </div>

  <div class="grid grid-cols-1 gap-3 md:grid-cols-[220px_minmax(0,1fr)]">
    <SelectField
      label="Kind"
      options={[
        { value: "service", label: "Service" },
        { value: "task", label: "Task" },
      ]}
      bind:value={process.kind}
      onblur={() => onFieldBlur?.(`process.${process.id}.kind`)}
    />
  </div>
</section>
```

- [ ] **Step 4: Re-run the focused test and the frontend check**

Run:

```bash
deno task test -- src/lib/components/ProcessForm.test.ts
deno task check
```

Expected: both commands exit successfully.

- [ ] **Step 5: Commit the process-form slice**

```bash
git add src/lib/components/ProcessForm.svelte src/lib/components/ProcessForm.test.ts
git commit -m "feat: flatten selected process header in settings"
```

---

### Task 3: Convert Environment and Dependency Rows to Quiet Inline Editors

**Files:**
- Create: `src/lib/components/EnvEditor.test.ts`
- Create: `src/lib/components/DependencyEditor.test.ts`
- Modify: `src/lib/components/EnvEditor.svelte`
- Modify: `src/lib/components/DependencyEditor.svelte`

**Interfaces:**
- Consumes: existing editor props and validation callbacks.
- Produces: compact add controls, icon-only remove actions, quieter empty states, and row labels that are still accessible.

- [ ] **Step 1: Write the failing EnvEditor regression test**

Create `src/lib/components/EnvEditor.test.ts` with:

```ts
import { describe, expect, it, vi } from "vitest";
import { render } from "@testing-library/svelte";

import EnvEditor from "./EnvEditor.svelte";

describe("EnvEditor", () => {
  it("uses an icon-only remove action for variable rows", () => {
    const { getByRole, queryByText } = render(EnvEditor, {
      props: {
        rows: [{ id: "env-1", key: "EXAMPLE_ENV", value: "hello-from-devapp" }],
        issueFor: () => null,
        onAdd: vi.fn(),
        onRemove: vi.fn(),
      },
    });

    expect(getByRole("button", { name: "Remove variable EXAMPLE_ENV" })).toBeInTheDocument();
    expect(queryByText("Remove")).toBeNull();
  });
});
```

- [ ] **Step 2: Write the failing DependencyEditor regression test**

Create `src/lib/components/DependencyEditor.test.ts` with:

```ts
import { describe, expect, it, vi } from "vitest";
import { render } from "@testing-library/svelte";

import DependencyEditor from "./DependencyEditor.svelte";
import type { ProcessForm } from "$lib/config/editorModel";

const process: ProcessForm = {
  id: "process-1",
  name: "worker",
  kind: "service",
  cmd: "deno task worker",
  envRows: [],
  dependencies: [{ id: "dependency-1", processName: "api", condition: "ready" }],
  readyEnabled: false,
  readyType: "http",
  httpUrl: "http://localhost:3000",
  logPattern: "ready",
  logRegex: false,
  delayDurationMs: 1000,
  commandCmd: "",
  intervalMs: null,
  timeoutMs: 60000,
};

describe("DependencyEditor", () => {
  it("uses an icon-only remove action for dependency rows", () => {
    const { getByRole, queryByText } = render(DependencyEditor, {
      props: {
        process,
        processes: [
          process,
          { ...process, id: "process-2", name: "api" },
        ],
        dependencyIssue: () => null,
        onAdd: vi.fn(),
        onRemove: vi.fn(),
      },
    });

    expect(getByRole("button", { name: "Remove dependency on api" })).toBeInTheDocument();
    expect(queryByText("Remove")).toBeNull();
  });
});
```

- [ ] **Step 3: Run the focused tests and confirm they fail**

Run:

```bash
deno task test -- src/lib/components/EnvEditor.test.ts src/lib/components/DependencyEditor.test.ts
```

Expected: FAIL because both components still render visible `Remove` button text instead of icon-only controls with specific labels.

- [ ] **Step 4: Flatten EnvEditor and replace row removal with IconButton**

Replace `src/lib/components/EnvEditor.svelte` with:

```svelte
<script lang="ts">
  import type { EnvRow } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";

  type Props = {
    rows: EnvRow[];
    processId?: string;
    issueFor: (key: string) => string | null;
    onAdd: () => void;
    onRemove: (id: string) => void;
    onFieldBlur?: (key: string) => void;
  };

  let { rows = $bindable(), processId, issueFor, onAdd, onRemove, onFieldBlur }: Props = $props();
</script>

<section class="grid gap-3 border-t border-border/70 pt-5">
  <div class="flex items-start justify-between gap-3">
    <div>
      <h3 class="text-sm font-semibold text-text">Environment variables</h3>
      {#if processId}
        <p class="mt-1 text-xs leading-5 text-text-subtle">Injected into this process.</p>
      {/if}
    </div>
    <Button size="sm" onclick={onAdd}>Add variable</Button>
  </div>

  <div class="grid gap-2.5">
    {#if rows.length === 0}
      <div class="px-1 text-sm text-text-subtle">
        {processId ? "No process variables." : "No global variables."}
      </div>
    {:else}
      {#each rows as row (row.id)}
        {@const envIssueKey = processId ? `process.${processId}.env.${row.id}.key` : `env.${row.id}.key`}
        {@const keyError = issueFor(envIssueKey)}
        <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)_auto] md:items-start">
          <label class="grid gap-1">
            <input
              class={`h-9 rounded-md border px-3 text-sm outline-none transition-colors duration-75 ${
                keyError
                  ? "border-danger focus:border-danger"
                  : "border-border bg-surface-raised focus:border-accent"
              }`}
              placeholder="KEY"
              bind:value={row.key}
              onblur={() => onFieldBlur?.(processId ? `process.${processId}.env.${row.id}.key` : `env.${row.id}.key`)}
            />
            {#if keyError}
              <span class="text-xs text-danger">{keyError}</span>
            {/if}
          </label>
          <input
            class="h-9 rounded-md border border-border bg-surface-raised px-3 text-sm outline-none transition-colors duration-75 focus:border-accent"
            placeholder="value"
            bind:value={row.value}
            onblur={() => onFieldBlur?.(processId ? `process.${processId}.env.${row.id}.value` : `env.${row.id}.value`)}
          />
          <IconButton
            label={`Remove variable ${row.key || "row"}`}
            variant="ghost"
            size="sm"
            onclick={() => onRemove(row.id)}
            class="mt-1"
          >
            <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
              <path d="M6 2.75h4M3.75 4.5h8.5M6.5 6.5v5M9.5 6.5v5M5.5 2.75l.35-.6A.75.75 0 0 1 6.5 1.75h3a.75.75 0 0 1 .65.4l.35.6m-6 1.75V12a1.25 1.25 0 0 0 1.25 1.25h4.5A1.25 1.25 0 0 0 11.5 12V4.5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </IconButton>
        </div>
      {/each}
    {/if}
  </div>
</section>
```

- [ ] **Step 5: Flatten DependencyEditor and replace row removal with IconButton**

Replace `src/lib/components/DependencyEditor.svelte` with:

```svelte
<script lang="ts">
  import type { DependencyCondition } from "$lib/types";
  import type { ProcessForm } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";

  type Props = {
    process: ProcessForm;
    processes: ProcessForm[];
    dependencyIssue: (process: ProcessForm, dependencyId: string) => string | null;
    onAdd: (process: ProcessForm) => void;
    onRemove: (process: ProcessForm, dependencyId: string) => void;
    onFieldBlur?: (key: string) => void;
  };

  let {
    process,
    processes,
    dependencyIssue,
    onAdd,
    onRemove,
    onFieldBlur,
  }: Props = $props();
</script>

<section class="grid gap-3 border-t border-border/70 pt-5">
  <div class="flex items-start justify-between gap-3">
    <div>
      <h3 class="text-sm font-semibold text-text">Dependencies</h3>
      <p class="mt-1 text-xs leading-5 text-text-subtle">Process launch order for the selected node.</p>
    </div>
    <Button size="sm" onclick={() => onAdd(process)}>Add dependency</Button>
  </div>

  {#if process.dependencies.length === 0}
    <div class="px-1 text-sm text-text-subtle">
      Starts without dependencies.
    </div>
  {:else}
    <div class="grid gap-2.5">
      {#each process.dependencies as dependency (dependency.id)}
        {@const depError = dependencyIssue(process, dependency.id)}
        <div class="grid gap-1">
          <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_150px_auto] md:items-start">
            <select
              class={`h-9 rounded-md border px-3 text-sm outline-none transition-colors duration-75 ${
                depError
                  ? "border-danger focus:border-danger"
                  : "border-border bg-surface-raised focus:border-accent"
              }`}
              bind:value={dependency.processName}
              onblur={() => onFieldBlur?.(`process.${process.id}.dependency.${dependency.id}`)}
            >
              <option value="">Select process</option>
              {#each processes.filter((candidate) => candidate.id !== process.id) as candidate}
                <option value={candidate.name}>{candidate.name}</option>
              {/each}
            </select>
            <select
              class="h-9 rounded-md border border-border bg-surface-raised px-3 text-sm outline-none transition-colors duration-75 focus:border-accent"
              bind:value={dependency.condition}
              onblur={() => onFieldBlur?.(`process.${process.id}.dependency.${dependency.id}`)}
            >
              <option value={"ready" satisfies DependencyCondition}>Ready</option>
              <option value={"success" satisfies DependencyCondition}>Success</option>
            </select>
            <IconButton
              label={`Remove dependency on ${dependency.processName || "selected process"}`}
              variant="ghost"
              size="sm"
              onclick={() => onRemove(process, dependency.id)}
              class="mt-1"
            >
              <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
                <path d="M6 2.75h4M3.75 4.5h8.5M6.5 6.5v5M9.5 6.5v5M5.5 2.75l.35-.6A.75.75 0 0 1 6.5 1.75h3a.75.75 0 0 1 .65.4l.35.6m-6 1.75V12a1.25 1.25 0 0 0 1.25 1.25h4.5A1.25 1.25 0 0 0 11.5 12V4.5"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </IconButton>
          </div>
          {#if depError}
            <span class="text-xs text-danger">{depError}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</section>
```

- [ ] **Step 6: Re-run the focused tests and the frontend check**

Run:

```bash
deno task test -- src/lib/components/EnvEditor.test.ts src/lib/components/DependencyEditor.test.ts
deno task check
```

Expected: both commands exit successfully.

- [ ] **Step 7: Commit the row-editor slice**

```bash
git add src/lib/components/EnvEditor.svelte src/lib/components/EnvEditor.test.ts src/lib/components/DependencyEditor.svelte src/lib/components/DependencyEditor.test.ts
git commit -m "feat: quiet settings row editors and icon-only removal controls"
```

---

### Task 4: Flatten the Readiness Subsection

**Files:**
- Create: `src/lib/components/ReadyCheckEditor.test.ts`
- Modify: `src/lib/components/ReadyCheckEditor.svelte`

**Interfaces:**
- Consumes: `process`, `readyIssue`, `onFieldBlur`.
- Produces: readiness inputs that keep existing behavior but lose the nested-card shell and oversized field overrides.

- [ ] **Step 1: Write the failing readiness-shell regression test**

Create `src/lib/components/ReadyCheckEditor.test.ts` with:

```ts
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";

import ReadyCheckEditor from "./ReadyCheckEditor.svelte";
import type { ProcessForm } from "$lib/config/editorModel";

const process: ProcessForm = {
  id: "process-1",
  name: "api",
  kind: "service",
  cmd: "deno task dev",
  envRows: [],
  dependencies: [],
  readyEnabled: true,
  readyType: "log",
  httpUrl: "http://localhost:3000",
  logPattern: "ready",
  logRegex: false,
  delayDurationMs: 1000,
  commandCmd: "",
  intervalMs: null,
  timeoutMs: 60000,
};

describe("ReadyCheckEditor", () => {
  it("drops the nested card chrome while keeping readiness fields visible", () => {
    const { container, getByLabelText } = render(ReadyCheckEditor, {
      props: {
        process,
        readyIssue: () => null,
      },
    });

    const section = container.querySelector("section") as HTMLElement;

    expect(section.className).not.toContain("rounded-md");
    expect(section.className).not.toContain("bg-surface-raised/40");
    expect(getByLabelText("Pattern")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run the focused test and confirm it fails**

Run:

```bash
deno task test -- src/lib/components/ReadyCheckEditor.test.ts
```

Expected: FAIL because `ReadyCheckEditor.svelte` still renders the rounded bordered shell.

- [ ] **Step 3: Replace the nested readiness card with a flat subsection**

Replace `src/lib/components/ReadyCheckEditor.svelte` with:

```svelte
<script lang="ts">
  import type { ProcessForm } from "$lib/config/editorModel";
  import CheckboxField from "$lib/components/ui/CheckboxField.svelte";
  import SelectField from "$lib/components/ui/SelectField.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";

  type Props = {
    process: ProcessForm;
    readyIssue: (process: ProcessForm, field: string) => string | null;
    onFieldBlur?: (key: string) => void;
  };

  let { process, readyIssue, onFieldBlur }: Props = $props();
</script>

<section class="grid gap-3 border-t border-border/70 pt-5">
  <div>
    <h3 class="text-sm font-semibold text-text">Readiness</h3>
    <p class="mt-1 text-xs leading-5 text-text-subtle">Optional startup probe for services that need an explicit ready signal.</p>
  </div>

  <CheckboxField
    label="Enable readiness check"
    class="text-xs text-text-muted"
    bind:checked={process.readyEnabled}
    onblur={() => onFieldBlur?.(`process.${process.id}.ready.enabled`)}
  />

  {#if process.readyEnabled}
    <div class="grid grid-cols-1 gap-3 md:grid-cols-[180px_minmax(0,1fr)]">
      <SelectField
        label="Type"
        options={[
          { value: "http", label: "HTTP" },
          { value: "log", label: "Log" },
          { value: "delay", label: "Delay" },
          { value: "command", label: "Command" },
        ]}
        bind:value={process.readyType}
        onblur={() => onFieldBlur?.(`process.${process.id}.ready.type`)}
      />

      {#if process.readyType === "http"}
        <TextField
          label="URL"
          error={readyIssue(process, "httpUrl")}
          bind:value={process.httpUrl}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.httpUrl`)}
        />
      {:else if process.readyType === "log"}
        <div class="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,1fr)_120px]">
          <TextField
            label="Pattern"
            error={readyIssue(process, "logPattern")}
            bind:value={process.logPattern}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.logPattern`)}
          />
          <CheckboxField
            label="Regex"
            class="mt-7 text-xs text-text-muted"
            bind:checked={process.logRegex}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.logRegex`)}
          />
        </div>
      {:else if process.readyType === "delay"}
        <TextField
          label="Duration ms"
          type="number"
          min="0"
          error={readyIssue(process, "delayDurationMs")}
          bind:value={process.delayDurationMs}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.delayDurationMs`)}
        />
      {:else}
        <TextField
          label="Command"
          monospace
          error={readyIssue(process, "commandCmd")}
          bind:value={process.commandCmd}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.commandCmd`)}
        />
      {/if}
    </div>

    {#if process.readyType !== "delay"}
      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        {#if process.readyType === "http" || process.readyType === "command"}
          <TextField
            label="Interval ms"
            type="number"
            min="0"
            error={readyIssue(process, "intervalMs")}
            bind:value={process.intervalMs}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.intervalMs`)}
          />
        {/if}
        <TextField
          label="Timeout ms"
          type="number"
          min="0"
          error={readyIssue(process, "timeoutMs")}
          bind:value={process.timeoutMs}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.timeoutMs`)}
        />
      </div>
    {/if}
  {/if}
</section>
```

- [ ] **Step 4: Re-run the focused test and the frontend check**

Run:

```bash
deno task test -- src/lib/components/ReadyCheckEditor.test.ts
deno task check
```

Expected: both commands exit successfully.

- [ ] **Step 5: Commit the readiness slice**

```bash
git add src/lib/components/ReadyCheckEditor.svelte src/lib/components/ReadyCheckEditor.test.ts
git commit -m "feat: flatten readiness subsection in settings"
```

---

### Task 5: Rebuild the ConfigEditor Page Shell Around the A1 Layout

**Files:**
- Modify: `src/lib/components/ConfigEditor.svelte`
- Modify: `src/lib/components/ConfigEditor.test.ts`

**Interfaces:**
- Consumes: existing ConfigEditor load/save/validation state and the flattened child editors from Tasks 2-4.
- Produces: a calmer left rail, a controlled-width main column, lighter section wrappers, and a footer that always uses the stable `Save` label.

- [ ] **Step 1: Add the failing page-level Save-label regression test**

Append this test to `src/lib/components/ConfigEditor.test.ts`:

```ts
  it("keeps the primary action label stable in page mode", () => {
    const { getByRole, queryByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose: vi.fn(),
        mode: "page",
      },
    });

    expect(getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(queryByRole("button", { name: "Save settings" })).toBeNull();
  });
```

- [ ] **Step 2: Run the focused page test and confirm it fails**

Run:

```bash
deno task test -- src/lib/components/ConfigEditor.test.ts
```

Expected: FAIL because the current footer button still renders `Save settings` when the editor is clean.

- [ ] **Step 3: Replace the footer label logic and soften the navigation shell**

In `src/lib/components/ConfigEditor.svelte`, update `settingsNavClass()` to reduce active/hover weight:

```ts
  function settingsNavClass(sectionId: SectionId) {
    return activeSection === sectionId
      ? "bg-surface-raised/70 text-text"
      : "text-text-subtle hover:bg-surface-hover/70 hover:text-text";
  }
```

Then replace `editorFooter()` with:

```svelte
{#snippet editorFooter()}
  <div class="flex items-center justify-between gap-3">
    <div class="text-xs text-text-subtle">
      {#if formIssueCount > 0}
        <span class="text-danger">{formIssueCount} validation issue{formIssueCount === 1 ? "" : "s"} must be fixed before saving.</span>
      {:else if isDirty}
        Unsaved changes in project YAML
      {:else}
        {status ?? "Changes are saved to the project YAML."}
      {/if}
    </div>
    <div class="flex gap-2">
      <Button onclick={requestClose} disabled={saving}>
        Cancel
      </Button>
      <Button variant="primary" onclick={save} disabled={!project || saving || loadError !== null}>
        Save
      </Button>
    </div>
  </div>
{/snippet}
```

- [ ] **Step 4: Flatten the page body into a continuous inspector document**

In `src/lib/components/ConfigEditor.svelte`, make the following focused replacements:

Replace the page wrapper line:

```svelte
      <div class="mx-auto flex w-full max-w-245 flex-col gap-6 px-5 py-6 lg:px-8 lg:py-8">
```

Replace the section-nav links from `py-2 text-sm` to `py-1.5 text-[13px]`, for example:

```svelte
              class={`rounded-md px-3 py-1.5 text-[13px] transition-colors duration-75 ${settingsNavClass("settings-general")}`}
```

Do the same for each section link and the process selection buttons in the left rail:

```svelte
                class={`flex min-w-44 items-center justify-between gap-2 rounded-md px-3 py-1.5 text-left text-[13px] transition-colors duration-75 lg:mb-1 lg:w-full lg:min-w-0 ${
                  process.id === selectedProcess?.id
                    ? "bg-surface-raised/70 text-text"
                    : "text-text-subtle hover:bg-surface-hover/70 hover:text-text"
                }`}
```

Replace the `General` section wrapper with a flat block:

```svelte
          <section bind:this={generalSection} id="settings-general" class="grid gap-4">
            <div>
              <h2 class="text-base font-semibold text-text">General</h2>
              <p class="mt-1 text-sm leading-6 text-text-subtle">Project-scoped runtime settings for the current workspace.</p>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
              <div class="rounded-lg border border-border/70 bg-surface/40 px-4 py-3">
                <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-subtle">Project</div>
                <div class="mt-2 text-sm font-medium text-text">{project?.name ?? "No project selected"}</div>
                <div class="mt-1 wrap-break-word text-xs leading-5 text-text-subtle">{project?.baseDir ?? "Select a project to load its runtime config."}</div>
              </div>

              <div class="rounded-lg border border-border/70 bg-surface/40 px-4 py-3">
                <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-subtle">Config source</div>
                <div class="mt-2 text-sm font-medium text-text">{projectSourceLabel}</div>
                <div class="mt-1 wrap-break-word text-xs leading-5 text-text-subtle">{project?.configPath ?? "The backend will resolve devapp.yml when a project is selected."}</div>
              </div>
            </div>
          </section>
```

Replace the `Environment`, `Processes`, and `YAML preview` section wrappers so they use dividers instead of rounded cards:

```svelte
          <section bind:this={environmentSection} id="settings-environment" class="grid gap-4 border-t border-border/70 pt-6">
            <div>
              <h2 class="text-base font-semibold text-text">Environment</h2>
              <p class="mt-1 text-sm leading-6 text-text-subtle">Shared variables are injected into every configured process.</p>
            </div>

            <EnvEditor
              bind:rows={globalEnvRows}
              {issueFor}
              onAdd={addGlobalEnvRow}
              onRemove={removeGlobalEnvRow}
              onFieldBlur={markTouched}
            />
          </section>

          <section bind:this={processesSection} id="settings-processes" class="grid gap-5 border-t border-border/70 pt-6">
            <div class="flex items-start justify-between gap-3">
              <div>
                <h2 class="text-base font-semibold text-text">Processes</h2>
                <p class="mt-1 text-sm leading-6 text-text-subtle">Select a process in the left rail to edit command, env, dependencies, and readiness.</p>
              </div>
              <Button size="sm" onclick={addProcess} disabled={loading || loadError !== null}>Add process</Button>
            </div>

            {#if selectedProcess}
              <div class="grid gap-0">
                <ProcessForm
                  process={selectedProcess}
                  processCount={processes.length}
                  {processIssue}
                  onRemove={removeProcess}
                  onFieldBlur={markTouched}
                />

                <EnvEditor
                  bind:rows={selectedProcess.envRows}
                  processId={selectedProcess.id}
                  {issueFor}
                  onAdd={() => addEnvRow(selectedProcess)}
                  onRemove={(id) => removeEnvRow(selectedProcess, id)}
                  onFieldBlur={markTouched}
                />

                <DependencyEditor
                  process={selectedProcess}
                  {processes}
                  {dependencyIssue}
                  onAdd={addDependency}
                  onRemove={removeDependency}
                  onFieldBlur={markTouched}
                />

                <ReadyCheckEditor process={selectedProcess} {readyIssue} onFieldBlur={markTouched} />
              </div>
            {:else}
              <div class="px-1 text-sm text-text-subtle">
                Add a process to start building the runtime graph.
              </div>
            {/if}
          </section>

          <section bind:this={previewSection} id="settings-preview" class="grid gap-4 border-t border-border/70 pt-6">
            <div class="flex items-center justify-between gap-3">
              <div>
                <h2 class="text-base font-semibold text-text">YAML preview</h2>
                <p class="mt-1 text-sm leading-6 text-text-subtle">Preview the generated devapp.yml before saving it back to the project.</p>
              </div>
              <Button size="sm" onclick={() => (showPreview = !showPreview)}>
                {showPreview ? "Hide preview" : "Preview YAML"}
              </Button>
            </div>
            {#if showPreview}
              <pre data-native-selectable="yaml-preview" class="max-h-72 overflow-auto rounded-md border border-border/70 bg-surface/40 p-3 font-mono text-xs leading-5 text-text-muted">{previewYaml}</pre>
            {/if}
          </section>
```

This step also resolves the existing `deno task check` suggestions by replacing `max-w-[980px]` with `max-w-245` and `break-words` with `wrap-break-word` while touching the same lines.

- [ ] **Step 5: Re-run focused tests, the full component subset, and the frontend check**

Run:

```bash
deno task test -- src/lib/components/ui/IconButton.test.ts src/lib/components/ProcessForm.test.ts src/lib/components/EnvEditor.test.ts src/lib/components/DependencyEditor.test.ts src/lib/components/ReadyCheckEditor.test.ts src/lib/components/ConfigEditor.test.ts
deno task check
```

Expected: both commands exit successfully.

- [ ] **Step 6: Launch the app and perform manual QA on the settings page**

Run:

```bash
deno task app examples/deno-runner.yml
```

Expected: the Tauri window opens. In the running app, open Runtime configuration and verify all of the following:

- the left rail reads as text-first navigation, not stacked mini-cards;
- the selected process reads as a single vertical sheet;
- add buttons feel secondary and compact;
- remove actions are discoverable on hover/focus but not loud at rest;
- the footer always says `Save`, never `Save *` or `Save settings`;
- dirty state remains obvious enough through the footer copy;
- validation messages still appear inline and close to the fields they belong to.

- [ ] **Step 7: Commit the page-shell redesign**

```bash
git add src/lib/components/ConfigEditor.svelte src/lib/components/ConfigEditor.test.ts
git commit -m "feat: apply A1 continuous inspector redesign to ConfigEditor"
```
