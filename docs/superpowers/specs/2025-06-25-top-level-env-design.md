# Design: Top-Level `env:` in Configuration

Date: 2025-06-25

## Problem

The `DevappConfig` Rust struct has no `env` field, but the README, AGENTS.md,
and `docs/configuration.md` all document a top-level `env:` block that should
be merged into every configured process and readiness check command. Serde
silently ignores unknown YAML fields during deserialization, so any `env:`
block in a `devapp.yml` is dropped with no error or warning.

## Design Decisions

| Decision | Choice |
|----------|--------|
| Merge strategy | Process-level env overrides top-level env on key conflict |
| Scope | Top-level env flows to process commands **and** readiness check commands |
| Merge timing | At spawn time in orchestrator (config stays pure, not baked into `ProcessConfig`) |
| UI placement | New "Global Environment" section at the top of ConfigEditor, above the process list |

## Changes

### 1. Rust Domain Model (`src-tauri/src/domain/config.rs`)

Add `env` field to `DevappConfig`:

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DevappConfig {
    pub version: u32,
    #[serde(default)]
    pub env: IndexMap<String, String>,
    pub processes: IndexMap<String, ProcessConfig>,
}
```

Add `env` to `RawDevappConfig` inside the custom `Deserialize` impl so the
field is captured during deserialization:

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDevappConfig {
    version: u32,
    #[serde(default)]
    env: IndexMap<String, String>,
    processes: IndexMap<String, ProcessConfig>,
}
```

The existing `validate_version` check is unaffected. No new validation rules
are needed for `env` — empty keys in a YAML map are rejected by the parser
naturally.

### 2. Orchestrator (`src-tauri/src/application/orchestrator.rs`)

In `spawn_named_process`, the `env` for `spawn_process` is currently built
from only `process.config.env` (lines 330–335). Change the block to merge
top-level env first, then overlay per-process env:

```rust
let mut env: HashMap<_, _> = active.loaded_config.config.env.iter()
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect();
for (key, value) in &process.config.env {
    env.insert(key.clone(), value.clone());
}
```

The merged `env` is already passed to `wait_until_ready` on line 412, so
readiness checks automatically receive the merged environment. No changes
to `command_runner.rs` or `readiness.rs`.

### 3. Frontend Types (`src/lib/types.ts`)

Add optional `env` to `DevappConfig`:

```ts
export type DevappConfig = {
  version: number;
  env?: Record<string, string>;
  processes: Record<string, ProcessConfig>;
};
```

### 4. Frontend Form Model (`src/lib/config/editorModel.ts`)

**a) Add `globalEnvRows` to `ConfigFormState`:**

```ts
export type ConfigFormState = {
  version: number;
  globalEnvRows: EnvRow[];
  processes: ProcessForm[];
};
```

**b) Load path** in `ConfigEditor.svelte`: populate `globalEnvRows` from the
loaded config's top-level `env`, alongside populating processes. No new
function is needed — expand the existing load logic inline.

**c) `buildConfig()`** emits top-level `env` from `globalEnvRows`:

```ts
export function buildConfig(form: ConfigFormState): DevappConfig {
  const env = Object.fromEntries(
    form.globalEnvRows
      .map((row) => [row.key.trim(), row.value] as const)
      .filter(([key]) => key.length > 0),
  );
  return {
    version: form.version,
    env: Object.keys(env).length > 0 ? env : undefined,
    processes: /* existing */,
  };
}
```

**d) `serializeConfig()`** emits a top-level `env:` block before
`processes:` when global env is non-empty.

**e) `buildProcessConfig`** is unchanged — per-process env remains
independent.

### 5. ConfigEditor UI (`src/lib/components/ConfigEditor.svelte`)

Add a `globalEnvRows = $state<EnvRow[]>([])` and render a new `EnvEditor`
at the top of the form mode area, above the process list. The existing
`EnvEditor` component already supports an optional `processId` prop that
scopes validation issue keys; when used without `processId` it shows
"global" copy and keys un-prefixed issue keys.

The load path populates `globalEnvRows` from the loaded config's `env`.
The form mode layout becomes:

```
[Warning banner]
[Global Environment section — EnvEditor]
[Process list sidebar / Process form area]
```

### 6. Tests

- **Backend config_loader test**: Add a test that `env:` at top level parses
  successfully and the values are accessible on `DevappConfig.env`.
- **Backend orchestrator test**: Add a test verifying that top-level env
  is merged (and overridden by process-level env) in the env map passed to
  `spawn_process`.
- **Frontend editorModel test**: Verify `buildConfig` and `serializeConfig`
  correctly handle global env rows.

### 7. Unchanged

- `LoadProjectConfig` — the `config: DevappConfig` field already carries
  the full struct, including the new `env` field.
- `command_runner.rs` — receives a `HashMap<String, String>`; no change.
- `readiness.rs` — already receives env; merged env flows through.
- `ProcessConfig` — per-process `env` stays as-is.
- `EnvEditor.svelte` — reused as-is for global env (it already supports
  optional `processId` for key scoping).
- `validation.ts` — top-level env keys have no special validation beyond
  what the form already does (non-empty keys), but a mild uniqueness check
  can be added if needed.

## Files Touched

- `src-tauri/src/domain/config.rs`
- `src-tauri/src/application/orchestrator.rs`
- `src/lib/types.ts`
- `src/lib/config/editorModel.ts`
- `src/lib/components/ConfigEditor.svelte`
- `src-tauri/src/infrastructure/config_loader.rs` (tests only)
