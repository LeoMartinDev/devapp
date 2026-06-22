# Sidebar Contextual Process Action — Design Spec

**Date:** 2026-06-21
**Status:** Validated, ready for implementation plan
**Scope:** Frontend (Svelte) + Rust/Tauri backend. Adds a new backend command.

## Context

The sidebar `ProcessList` (`src/lib/components/ProcessList.svelte`) currently
exposes a single action per process row: **Restart** (↻). There is no **Stop**
action in the sidebar — to stop a process the user must select it and use the
⚙ menu in the content header. This asymmetry is surprising for an operational
desktop tool: the user can restart from the sidebar but cannot stop there.

Additionally, restarting a previously stopped/failed process is the only way to
bring it back, and the current `restart_process` backend path performs an
explicit `stop` step that is wasted work (and marks `terminating=true`) for a
process that is no longer running.

## Goal

Give each sidebar process row a single, dense, state-aware action button that
lets the user either **Stop** a running process or **Start** a stopped/failed
one, directly from the sidebar. Restart remains available via the ⚙ menu on the
selected process.

## Decisions (from brainstorming)

1. **Single contextual button.** One button per row whose icon, action, and
   label change with the process state — instead of two always-visible buttons.
   Keeps the sidebar compact per the project's "dense and operational" UI rule.
2. **No explicit Restart in the sidebar.** Restart is removed from the sidebar.
   Restart stays available through the ⚙ menu on the selected process (the
   `ProjectMenu` already exposes both Stop and Restart there).
3. **Visibility: on hover/focus (unchanged).** The action button keeps the
   existing `md:opacity-0 → group-hover:opacity-100 / group-focus-within`
   behavior. No always-visible change.
4. **New backend command `start_process`.** Reset→spawn without the stop step,
   so re-launching a failed/stopped process is clean and does not flip
   `terminating`. `stop_process` and `restart_process` are unchanged.

## Backend changes

### `src-tauri/src/application/orchestrator.rs`

Add a `start_process` method modeled on `restart_process`, minus the stop step:

```rust
pub async fn start_process(
    &self,
    app_handle: AppHandle,
    window_key: &str,
    process_name: &str,
) -> Result<Option<RunSessionSnapshot>, AppError> {
    self.reset_process(window_key, process_name).await?;
    self.spawn_runnable_processes(app_handle.clone(), window_key)
        .await?;
    self.snapshot(window_key).await
}
```

`reset_process` already clears `child`, sets `terminating=false`, resets status
to `Pending`, and zeroes timing/exit fields. `spawn_runnable_processes` already
re-spawns processes whose status is `Pending | Blocked | Stopped`, so a
stopped/failed process will be relaunched. `stop_process` and `restart_process`
are left unchanged.

### `src-tauri/src/tauri_api/commands.rs`

Add a Tauri command mirroring `restart_process`, reusing `ProcessActionRequest`:

```rust
#[tauri::command]
pub async fn start_process(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    request: ProcessActionRequest,
) -> Result<Option<RunSessionSnapshot>, String> {
    state
        .orchestrator
        .start_process(app_handle, &window_key(&window), &request.process_name)
        .await
        .map_err(String::from)
}
```

### Command registration

Register `start_process` in the `tauri::generate_handler!` list in
`src-tauri/src/lib.rs`, next to `restart_process` and `stop_process` (currently
lines 44–45).

### Tests (`cargo test` in `src-tauri`)

Add a unit/integration test (following the style of existing
`restart_process`/`stop_process` tests) verifying that:

- A process in `stopped` or `failed` state returns to a running/ready state
  after `start_process`.
- Other processes in the same session are unaffected by `start_process`.

## Frontend changes

### `src/lib/tauri/client.ts`

Add a typed wrapper next to `restartProcess`/`stopProcess`:

```ts
export async function startProcess(
  processName: string,
): Promise<RunSessionSnapshot | null> {
  return invoke<RunSessionSnapshot | null>("start_process", {
    request: { processName },
  });
}
```

### `src/lib/stores/runtime.svelte.ts`

Add `startSessionProcess` modeled exactly on `restartSessionProcess`:

```ts
async startSessionProcess(processName: string) {
  this.busy = true;
  try {
    this.session = await startProcess(processName);
    this.syncProcessSelection();
  } catch (error) {
    this.setError(error);
    throw error;
  } finally {
    this.busy = false;
  }
}
```

### `src/lib/components/ProcessList.svelte`

Replace the `onRestart` prop with two new props and render a single
**contextual** action button whose behavior depends on the process status.

**Props change:**

- Remove `onRestart`.
- Add `onStop: (processName: string) => void`.
- Add `onStart: (processName: string) => void`.

**State → button mapping:**

| Process status                         | Icon   | Action       | `title`/`aria-label` |
| -------------------------------------- | ------ | ------------ | -------------------- |
| `running` / `ready` / `starting`       | ■      | `onStop`     | `Stop {name}`        |
| `stopped` / `failed` / `succeeded`     | ▶      | `onStart`    | `Start {name}`       |
| `pending` / `blocked` / `stopping`     | —      | disabled     | —                    |

**Button behavior:**

- Disabled when `busy` is true, or when the status is `pending`, `blocked`, or
  `stopping` (transitional/non-actionable states).
- `event.stopPropagation()` on click (unchanged) so the row's select handler
  does not fire.
- Hover/focus visibility unchanged (`md:opacity-0 → group-hover:opacity-100 /
  group-focus-within:opacity-100`).
- Stop variant uses `hover:text-danger` styling (consistent with the sibling
  terminal close button and the ⚙ menu's danger `Stop` entry); the Start
  variant uses the existing neutral `text-text-subtle`.
- Inline SVG icons: filled square (Stop) and triangle (Start). The existing
  restart (↻) SVG is removed from this component.

The terminal-row action (close button) is unchanged.

### `src/routes/+page.svelte`

Update the `<ProcessList>` invocation:

- Replace `onRestart={(name) => runtimeStore.restartSessionProcess(name)}` with
  `onStart={(name) => runtimeStore.startSessionProcess(name)}` and
  `onStop={(name) => runtimeStore.stopSessionProcess(name)}`.

`runtimeStore.restartSessionProcess` and `restartProcess` are **kept** — the ⚙
menu (`ProjectMenu.svelte`) still calls `restartSessionProcess` on the selected
process. No change to `ProjectMenu.svelte`.

## Out of scope

- No change to `stop_process` or `restart_process` behavior.
- No change to the ⚙ menu (`ProjectMenu.svelte`).
- No change to `RunStopButton` (session-level Run/Stop).
- No persistent/exportable logs, no per-process autorestart policy, no UI for
  manually starting a process that is still `pending`/`blocked` in the normal
  flow.
- No new sidebar actions beyond the single contextual Stop/Start button.

## Risks / notes

- `start_process` on a process that is already running is a no-op for the user's
  intent but the UI never offers Start on a running process (the button is Stop
  there), so this path is not reachable from the sidebar. The ⚙ menu does not
  expose `start_process` either.
- `spawn_runnable_processes` already short-circuits when `stop_requested` is
  true for the session, so `start_process` during a session-wide stop will not
  race the shutdown.
