# Config Editor Refactor — Design Spec

**Date:** 2026-06-25
**Status:** Validated, ready for implementation plan
**Scope:** Frontend only — `ConfigEditor.svelte` and related components. No backend or schema changes.

## Context

The project config editor currently has two editing modes — a structured form and
a raw YAML textarea — toggled by a `SegmentedControl`. Raw mode was meant to
preserve comments and unsupported fields, but it introduces complexity (sync
issues, duplicate save paths, separate error handling) without enough value for
V1's deliberately small schema.

The goal is to simplify the editor to form-only with a read-only YAML preview,
add dirty-state protection (visual indicators, no blocking dialogs), and add
real-time inline validation.

## Decisions (from brainstorming)

1. **Remove raw mode entirely.** No more `SegmentedControl` or mode toggle.
   The YAML preview panel (collapsible, read-only, already exists) is kept.

2. **Dirty-state is visual only.** `isDirty` flag drives a highlighted Save
   button and an "Unsaved changes" badge. No confirmation dialog on close —
   the user is responsible for saving. If they close, changes are lost.

3. **Validation runs in real time.** A `$effect` with 300 ms debounce calls
   `validateConfigForm()` on every form change. Errors appear inline without
   waiting for a save attempt.

4. **Errors only appear for touched fields.** A `touchedFields: Set<string>`
   tracks fields that received and lost focus (`onblur`). Validation errors
   for untouched fields are hidden until the user interacts with them.

5. **Save button is always clickable.** Real-time errors are advisory. A final
   validation runs on save and blocks if errors still exist (current behavior).

## Architecture

### `ConfigEditor.svelte` — state changes

| Before | After |
|--------|-------|
| `mode: "form" \| "raw"` | Removed |
| `rawYaml: string` | Removed |
| `rawError: string \| null` | Removed |
| `validationIssues` (save-only) | Updated continuously via `$effect` |
| No dirty tracking | `isDirty = $state(false)` |
| No touched tracking | `touchedFields = $state(new Set<string>())` |
| `{#if mode === "raw"}...{:else}...` | Single form template, no branching |

### Dirty-state data flow

```
[Any form field change] ──► isDirty = true
    ├── Save button: variant="primary", label="Save *"
    └── Footer badge: "Unsaved changes" (text-warning)

[handleSave() succeeds] ──► isDirty = false
    ├── Save button: normal variant, label="Save"
    └── Footer badge: hidden

[Dialog closes without save] ──► no confirmation, changes lost
```

### Validation data flow

```
[Field changes] ──► 300ms debounce ──► validateConfigForm(formState)
    │
    └── validationIssues updated
        │
        └── Template filters by touchedFields ──► inline error <span>

[Field onblur] ──► touchedFields.add(fieldKey)

[handleSave()] ──► validateConfigForm(formState) [sync, always runs]
    ├── has errors? ──► show all errors (ignore touchedFields), abort save
    └── no errors? ──► buildConfig → serializeConfig → saveConfig()
```

### Touched field keys

Each form field gets a unique key for touched tracking:

```
"process.name.<processId>"
"process.cmd.<processId>"
"process.env.<processId>.<index>"
"process.dep.<processId>.<depName>"
"process.ready.<processId>.<field>"
"globalEnv.<index>"
```

The `issueFor()`, `processIssue()`, `readyIssue()`, `dependencyIssue()`, and
`envRowIssue()` helpers already produce a `ValidationIssue` with a `field`
property. The template uses `touchedFields.has(issue.field)` to gate display.

### Template simplification

The current template has two large branches (`mode === "raw"` and `mode === "form"`).
After the refactor:

```svelte
<!-- No SegmentedControl -->
<!-- No raw mode warning banner -->

<!-- Always: form UI -->
<EnvEditor bind:rows={globalEnvRows} ... />
<ProcessForm bind:name bind:cmd bind:kind ... />
<EnvEditor bind:rows={processEnvRows} ... />
<DependencyEditor ... />
<ReadyCheckEditor ... />

<!-- YAML preview (read-only, collapsible, already exists) -->

<!-- Footer -->
{#if validationIssues.length > 0}
  <span class="text-danger">X validation issue(s)</span>
{/if}
{#if isDirty}
  <span class="text-warning">Unsaved changes</span>
{/if}
```

### Files touched

| File | Change |
|------|--------|
| `src/lib/components/ConfigEditor.svelte` | Major: remove raw mode, add dirty/validation/touched state |
| `src/lib/components/ui/SegmentedControl.svelte` | No changes (may become unused; left for other consumers) |

No other files need changes. `ConfigProcessList`, `ProcessForm`, `EnvEditor`,
`DependencyEditor`, `ReadyCheckEditor`, `editorModel.ts`, `validation.ts`,
`runtime.svelte.ts`, and all backend files are untouched.

## Testing

- **Unit/manual**: Open a project, open config editor, modify fields — verify:
  - Save button highlights and shows "Save *"
  - Footer shows "Unsaved changes"
  - Save resets both indicators
  - Close dialog without saving — no confirmation, dialog closes
- **Validation**: Verify inline errors appear ~300ms after typing invalid values
  on touched fields, and that untouched fields don't show errors.
- **Regression**: Verify YAML preview updates, save persists to disk, non-config
  functionality (run/stop/logs/terminals) is unaffected.

## Out of scope

- Backend changes of any kind
- Schema changes (`devapp.yml` version/schema)
- Reordering processes, env entries, or dependencies
- Duplicate process functionality
- Frontend dependency cycle detection
- Frontend/backend validation reconciliation
- `ProjectSettingsDialog` changes
