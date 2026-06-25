# Config Editor Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove raw YAML editing mode, add dirty-state visual indicators, and add real-time debounced inline validation with touched-field gating.

**Architecture:** Three sequential changes to `ConfigEditor.svelte`, plus minor callback additions to ProcessForm, EnvEditor, DependencyEditor, and ReadyCheckEditor. No backend changes. Core file is `src/lib/components/ConfigEditor.svelte` (~426 lines → ~320 lines).

**Tech Stack:** Svelte 5 runes (`$state`, `$derived`, `$effect`, `$bindable`), SvelteKit, TypeScript

## Global Constraints

- Form-only editing mode (no SegmentedControl, no raw YAML textarea)
- YAML preview panel (collapsible, read-only) preserved
- `isDirty` visual: Save button label appends ` *`, footer shows "Unsaved changes" badge
- `isDirty` resets on successful save
- Closing dialog without saving loses changes with no confirmation
- Debounced validation at 300ms via `$effect` + `setTimeout`
- `touchedFields: Set<string>` populated via `onblur`; errors hidden for untouched fields
- Save button always clickable; save-time validation runs synchronously
- No backend or schema changes

---

### Task 1: Remove raw YAML editing mode

**Files:**
- Modify: `src/lib/components/ConfigEditor.svelte`

**Produces:** Form-only ConfigEditor. SegmentedControl, raw textarea, `mode`/`rawYaml`/`rawError` state, and all mode-branching removed. YAML preview kept.

- [ ] **Step 1: Remove SegmentedControl import (line 22)**

```diff
- import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
```

- [ ] **Step 2: Remove raw-mode state variables (lines 44-52)**

```diff
- let mode = $state<"form" | "raw">("form");
- let rawYaml = $state("");
- let rawError = $state<string | null>(null);
- const modeOptions: { value: "form" | "raw"; label: string }[] = [
-   { value: "form", label: "Form" },
-   { value: "raw", label: "Raw YAML" },
- ];
```

- [ ] **Step 3: Remove `rawInputClass` function (lines 266-269)**

```diff
- function rawInputClass(error: string | null, extra = "") { ... }
```

- [ ] **Step 4: Patch `resetEmpty()` — remove `rawYaml` assignment (line 85)**

```diff
  function resetEmpty() {
    version = 1;
    globalEnvRows = [];
    processes = [newProcess("api")];
    selectedProcessId = processes[0].id;
-   rawYaml = serializeConfig(buildConfigFromForm({ version, globalEnvRows, processes }));
  }
```

- [ ] **Step 5: Patch `resetUnloaded()` — remove `rawYaml` assignment (line 93)**

```diff
  function resetUnloaded() {
    version = 1;
    globalEnvRows = [];
    processes = [];
    selectedProcessId = null;
-   rawYaml = "";
  }
```

- [ ] **Step 6: Patch `load()` — remove `rawError` and `rawYaml` references (lines 110, 134)**

```diff
  async function load(projectId: string) {
    loading = true;
    status = null;
    loadError = null;
-   rawError = null;
    validationIssues = [];
    showPreview = false;
    try {
      // ...
      selectedProcessId = processes[0].id;
-     rawYaml = document.yaml;
      loadedProjectId = projectId;
```

- [ ] **Step 7: Simplify `save()` — remove raw-mode branch (lines 215-223, 236-237)**

Replace the entire `save()` function:

```ts
async function save() {
  if (!project || loadError) {
    return;
  }
  saving = true;
  status = null;
  try {
    const validation = validateConfigForm(formState);
    validationIssues = validation.issues;
    if (!validation.valid) {
      return;
    }

    const yaml = previewYaml;
    await runtimeStore.saveConfig(yaml, project.id);
    status = "Settings saved";
    onClose();
  } catch (error) {
    status = errorMessage(error);
  } finally {
    saving = false;
  }
}
```

- [ ] **Step 8: Simplify template — remove SegmentedControl and raw-mode branches (lines 309-351)**

Remove the SegmentedControl wrapper (lines 309-315):
```diff
- <div class="flex justify-end border-b border-border px-5 py-3">
-   <SegmentedControl ... />
- </div>
```

Remove the raw-mode template branch (lines 328-346) and the form-mode warning (lines 349-351). The branching changes from:
```
{#if loading}...{:else if loadError}...{:else if mode === "raw"}...{:else}...
```
to:
```
{#if loading}...{:else if loadError}...{:else}...
```

Remove the form-mode warning banner:
```diff
- <div class="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-warning/90">
-   Saving rewrites the generated YAML and may remove comments or unsupported fields.
- </div>
```

- [ ] **Step 9: Simplify footer — remove `mode === "form"` guard (line 410)**

```diff
- {#if formIssueCount > 0 && mode === "form"}
+ {#if formIssueCount > 0}
```

- [ ] **Step 10: Run check and test**

```sh
deno task check
```

Expected: clean exit (the known SvelteKit `tsconfig.json` warning is non-blocking).

- [ ] **Step 11: Commit**

```sh
git add src/lib/components/ConfigEditor.svelte
git commit -m "refactor: remove raw YAML editing mode from ConfigEditor"
```

---

### Task 2: Add dirty-state tracking

**Files:**
- Modify: `src/lib/components/ConfigEditor.svelte`

**Consumes:** Form-only ConfigEditor from Task 1
**Produces:** `isDirty` state trackable by form-field changes; visual indicators on save button and footer.

- [ ] **Step 1: Add `isDirty` and `suppressDirty` state variables**

After the existing state declarations (after line 47 `validationIssues`):

```ts
let isDirty = $state(false);
let suppressDirty = $state(false);
```

- [ ] **Step 2: Add `$effect` to track form changes for dirty flag**

After the existing `$effect` for loading (after line 286):

```ts
$effect(() => {
  void version;
  void globalEnvRows;
  void processes;
  if (suppressDirty) return;
  isDirty = true;
});
```

- [ ] **Step 3: Patch `load()` — suppress dirty during loading, reset after**

In the `load()` function, wrap the form-state-setting code:

```ts
async function load(projectId: string) {
  loading = true;
  status = null;
  loadError = null;
  validationIssues = [];
  showPreview = false;
  try {
    const document = await runtimeStore.loadConfig(projectId);
    const config = document?.config;
    if (!config) {
      suppressDirty = true;
      resetEmpty();
      suppressDirty = false;
      isDirty = false;
      loadedProjectId = projectId;
      return;
    }
    suppressDirty = true;
    version = config.version;
    globalEnvRows = Object.entries(config.env ?? {}).map(([key, value]) => ({
      id: nextId("env"),
      key,
      value,
    }));
    processes = Object.entries(config.processes ?? {}).map(([name, process]) =>
      toProcessForm(name, process, nextId),
    );
    if (processes.length === 0) {
      processes = [newProcess("api")];
    }
    selectedProcessId = processes[0].id;
    suppressDirty = false;
    isDirty = false;
    loadedProjectId = projectId;
  } catch (error) {
    loadError = errorMessage(error);
    loadedProjectId = null;
    resetUnloaded();
  } finally {
    loading = false;
  }
}
```

- [ ] **Step 4: Patch `save()` — reset dirty after successful save**

In the `save()` function, after the successful save path:

```ts
try {
  const validation = validateConfigForm(formState);
  validationIssues = validation.issues;
  if (!validation.valid) {
    return;
  }

  const yaml = previewYaml;
  await runtimeStore.saveConfig(yaml, project.id);
  suppressDirty = true;
  isDirty = false;
  suppressDirty = false;
  status = "Settings saved";
  onClose();
} catch (error) {
  status = errorMessage(error);
} finally {
  saving = false;
}
```

- [ ] **Step 5: Modify footer — add "Unsaved changes" badge and dynamic save button label**

Replace the footer snippet:

```svelte
{#snippet footer()}
  <div class="flex items-center justify-between gap-3">
    <div class="text-xs text-text-subtle">
      {#if formIssueCount > 0}
        <span class="text-danger">{formIssueCount} validation issue{formIssueCount === 1 ? "" : "s"} must be fixed before saving.</span>
      {:else}
        {status ?? "Changes are saved to the project YAML."}
      {/if}
      {#if isDirty}
        <span class="text-warning"> Unsaved changes</span>
      {/if}
    </div>
    <div class="flex gap-2">
      <Button onclick={onClose} disabled={saving}>
        Cancel
      </Button>
      <Button variant="primary" onclick={save} disabled={!project || saving || loadError !== null}>
        Save{isDirty ? " *" : " settings"}
      </Button>
    </div>
  </div>
{/snippet}
```

- [ ] **Step 6: Run check**

```sh
deno task check
```

- [ ] **Step 7: Commit**

```sh
git add src/lib/components/ConfigEditor.svelte
git commit -m "feat: add dirty-state tracking with visual indicators in ConfigEditor"
```

---

### Task 3: Add real-time debounced validation with touched-field gating

**Files:**
- Modify: `src/lib/components/ConfigEditor.svelte`
- Modify: `src/lib/components/ProcessForm.svelte`
- Modify: `src/lib/components/EnvEditor.svelte`
- Modify: `src/lib/components/DependencyEditor.svelte`
- Modify: `src/lib/components/ReadyCheckEditor.svelte`

**Consumes:** Dirty-state tracking from Task 2; form-mode-only ConfigEditor from Task 1
**Produces:** Debounced inline validation errors displayed after field interaction (blur); untouched fields stay clean.

- [ ] **Step 1: Add `touchedFields` and debounce timer state in ConfigEditor.svelte**

After `let isDirty = $state(false);` from Task 2:

```ts
let touchedFields = $state(new Set<string>());
let debounceTimer: ReturnType<typeof setTimeout> | null = null;
```

- [ ] **Step 2: Add `markTouched` helper and debounced validation `$effect`**

After the `$effect` for dirty tracking from Task 2:

```ts
function markTouched(key: string) {
  touchedFields.add(key);
}

$effect(() => {
  void version;
  void globalEnvRows;
  void processes;
  if (!loadedProjectId) return;

  if (debounceTimer !== null) {
    clearTimeout(debounceTimer);
  }
  debounceTimer = setTimeout(() => {
    validationIssues = validateConfigForm(formState).issues;
  }, 300);
});
```

- [ ] **Step 3: Add field blur handlers in ProcessForm.svelte**

First, add `onFieldBlur` prop:

```diff
  type Props = {
    process: ProcessFormState;
    processCount: number;
    processIssue: (process: ProcessFormState, field: string) => string | null;
    onRemove: (id: string) => void;
+   onFieldBlur?: (key: string) => void;
  };

- let { process, processCount, processIssue, onRemove }: Props = $props();
+ let { process, processCount, processIssue, onRemove, onFieldBlur }: Props = $props();
```

Then add `onblur` to each TextField and SelectField:

```diff
  <TextField
    label="Name"
    class="h-10"
    error={processIssue(process, "name")}
    bind:value={process.name}
+   onblur={() => onFieldBlur?.(`process.${process.id}.name`)}
  />
  <TextField
    label="Command"
    class="h-10"
    placeholder="deno task dev"
    monospace
    error={processIssue(process, "cmd")}
    bind:value={process.cmd}
+   onblur={() => onFieldBlur?.(`process.${process.id}.cmd`)}
  />
```

```diff
  <SelectField
    label="Kind"
    class="h-10"
    options={[...]}
    bind:value={process.kind}
+   onblur={() => onFieldBlur?.(`process.${process.id}.kind`)}
  />
```

- [ ] **Step 4: Add field blur handlers in EnvEditor.svelte**

Add `onFieldBlur` prop:

```diff
  type Props = {
    rows: EnvRow[];
    processId?: string;
    issueFor: (key: string) => string | null;
    onAdd: () => void;
    onRemove: (id: string) => void;
+   onFieldBlur?: (key: string) => void;
  };

- let { rows = $bindable(), processId, issueFor, onAdd, onRemove }: Props = $props();
+ let { rows = $bindable(), processId, issueFor, onAdd, onRemove, onFieldBlur }: Props = $props();
```

Add `onblur` to the key input and value input:

```diff
  <input
    class={`h-9 rounded-md border px-3 text-sm ...`}
    placeholder="KEY"
    bind:value={row.key}
+   onblur={() => onFieldBlur?.(processId ? `process.${processId}.env.${row.id}.key` : `env.${row.id}.key`)}
  />
```

```diff
  <input
    class="h-9 rounded-md border border-border ..."
    placeholder="value"
    bind:value={row.value}
+   onblur={() => onFieldBlur?.(processId ? `process.${processId}.env.${row.id}.value` : `env.${row.id}.value`)}
  />
```

- [ ] **Step 5: Add field blur handlers in DependencyEditor.svelte**

Add `onFieldBlur` prop:

```diff
  type Props = {
    process: ProcessForm;
    processes: ProcessForm[];
    dependencyIssue: (process: ProcessForm, dependencyId: string) => string | null;
    onAdd: (process: ProcessForm) => void;
    onRemove: (process: ProcessForm, dependencyId: string) => void;
+   onFieldBlur?: (key: string) => void;
  };

  let {
    process,
    processes,
    dependencyIssue,
    onAdd,
    onRemove,
+   onFieldBlur,
  }: Props = $props();
```

Add `onblur` to the process-name select and condition select:

```diff
  <select ... bind:value={dependency.processName}
+   onblur={() => onFieldBlur?.(`process.${process.id}.dependency.${dependency.id}`)}
  >
```

```diff
  <select ... bind:value={dependency.condition}
+   onblur={() => onFieldBlur?.(`process.${process.id}.dependency.${dependency.id}`)}
  >
```

- [ ] **Step 6: Add field blur handlers in ReadyCheckEditor.svelte**

Add `onFieldBlur` prop:

```diff
  type Props = {
    process: ProcessForm;
    readyIssue: (process: ProcessForm, field: string) => string | null;
+   onFieldBlur?: (key: string) => void;
  };

- let { process, readyIssue }: Props = $props();
+ let { process, readyIssue, onFieldBlur }: Props = $props();
```

Add `onblur` to each field. Use consistent key format (`process.${process.id}.ready.<field>`):

On CheckboxField (readyEnabled):
```diff
  <CheckboxField
    label="Enable readiness check"
    class="text-sm text-text-muted"
    bind:checked={process.readyEnabled}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.enabled`)}
  />
```

On SelectField (readyType):
```diff
  <SelectField ... bind:value={process.readyType}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.type`)}
  />
```

On TextField (httpUrl):
```diff
  <TextField ... bind:value={process.httpUrl}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.httpUrl`)}
  />
```

On TextField (logPattern):
```diff
  <TextField ... bind:value={process.logPattern}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.logPattern`)}
  />
```

On CheckboxField (logRegex):
```diff
  <CheckboxField ... bind:checked={process.logRegex}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.logRegex`)}
  />
```

On TextField (delayDurationMs):
```diff
  <TextField ... bind:value={process.delayDurationMs}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.delayDurationMs`)}
  />
```

On TextField (commandCmd):
```diff
  <TextField ... bind:value={process.commandCmd}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.commandCmd`)}
  />
```

On TextField (intervalMs):
```diff
  <TextField ... bind:value={process.intervalMs}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.intervalMs`)}
  />
```

On TextField (timeoutMs):
```diff
  <TextField ... bind:value={process.timeoutMs}
+   onblur={() => onFieldBlur?.(`process.${process.id}.ready.timeoutMs`)}
  />
```

- [ ] **Step 7: Wire `onFieldBlur` callbacks in ConfigEditor.svelte template**

Pass `onFieldBlur={markTouched}` to each form child component:

```diff
  <EnvEditor
    bind:rows={globalEnvRows}
    {issueFor}
    onAdd={addGlobalEnvRow}
    onRemove={removeGlobalEnvRow}
+   onFieldBlur={markTouched}
  />

  <ProcessForm
    process={selectedProcess}
    processCount={processes.length}
    {processIssue}
    onRemove={removeProcess}
+   onFieldBlur={markTouched}
  />

  <EnvEditor
    bind:rows={selectedProcess.envRows}
    processId={selectedProcess.id}
    {issueFor}
    onAdd={() => addEnvRow(selectedProcess)}
    onRemove={(id) => removeEnvRow(selectedProcess, id)}
+   onFieldBlur={markTouched}
  />

  <DependencyEditor
    process={selectedProcess}
    {processes}
    {dependencyIssue}
    onAdd={addDependency}
    onRemove={removeDependency}
+   onFieldBlur={markTouched}
  />

  <ReadyCheckEditor
    process={selectedProcess}
    {readyIssue}
+   onFieldBlur={markTouched}
  />
```

- [ ] **Step 8: Gate error display by touched fields in ConfigEditor.svelte**

Update the error-display helpers (`issueFor`, `processIssue`, etc.) to filter by touched fields. Replace the existing helpers (lines 246-264):

```ts
function isTouched(key: string) {
  return touchedFields.has(key);
}

function issueFor(key: string) {
  const issue = validationIssues.find((issue) => issue.key === key);
  if (!issue) return null;
  if (!isTouched(key)) return null;
  return issue.message;
}

function processIssue(process: ProcessFormState, field: string) {
  return issueFor(`process.${process.id}.${field}`);
}

function readyIssue(process: ProcessFormState, field: string) {
  return issueFor(`process.${process.id}.ready.${field}`);
}

function dependencyIssue(process: ProcessFormState, dependencyId: string) {
  return issueFor(`process.${process.id}.dependency.${dependencyId}`);
}

function envRowIssue(process: ProcessFormState, rowId: string) {
  return issueFor(`process.${process.id}.env.${rowId}.key`);
}
```

- [ ] **Step 9: Run check**

```sh
deno task check
```

- [ ] **Step 10: Run backend tests**

```sh
cd src-tauri && cargo test
```

- [ ] **Step 11: Commit**

```sh
git add src/lib/components/ConfigEditor.svelte \
        src/lib/components/ProcessForm.svelte \
        src/lib/components/EnvEditor.svelte \
        src/lib/components/DependencyEditor.svelte \
        src/lib/components/ReadyCheckEditor.svelte
git commit -m "feat: add real-time debounced validation with touched-field gating"
```
