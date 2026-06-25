# Launch-Locked Project

## Summary

When devapp is launched with a project detected automatically (via
`devapp.yml` discovery) or passed explicitly as a CLI argument, the
window is locked to that project. The user cannot create, edit, delete,
or switch projects. Runtime operations (run/stop session, start/stop
individual processes, edit runtime config, terminals, logs) remain
fully available.

When launched without a project, the user can freely create, edit,
delete, and switch projects — the current behaviour is preserved.

Additionally, the auto-run behaviour is removed: a launch-locked
project is loaded and displayed but the session does not start
automatically. The user must click Run.

## Motivation

- A project opened via CLI or filesystem discovery represents an
  explicit intent. Allowing the user to switch away or delete it
  undermines that intent and creates confusing states.
- Auto-running on launch is surprising and can be destructive (e.g.,
  starting a dev server before the user is ready).
- The project-manager role (create/edit/delete) is separate from the
  runtime-operator role (run/stop/configure). Locking only project
  management keeps the tool operational while preventing accidental
  misconfiguration.

## Backend Changes

### 1. `LaunchProjectInfo` struct

Add a serializable struct in `src-tauri/src/tauri_api/commands.rs`:

```rust
#[derive(Serialize)]
pub struct LaunchProjectInfo {
    pub project_id: Option<ProjectId>,
    pub locked: bool,
}
```

`locked` is `true` when `AppState.launch_project_id` is `Some`.

### 2. `get_launch_project` return type

Change from `Result<Option<ProjectId>, AppError>` to
`Result<LaunchProjectInfo, AppError>`.

### 3. New error variant

In `src-tauri/src/error.rs`, add:

```rust
#[error("project is launch-locked and cannot be modified")]
LaunchLocked,
```

### 4. Guards on mutating commands

When `AppState.launch_project_id.lock().await.is_some()` (i.e.,
`locked` is `true`), the following commands return
`Err(AppError::LaunchLocked)`:

- `save_project` — cannot create or edit project records.
- `remove_project` — cannot delete the locked project.
- `open_project_window` — cannot open another project in a new window.
- `start_project` — rejects any `project_id` that does not match the
  locked project.

Not blocked:
- `stop_project`
- `save_project_config`
- `load_project_config`
- `start_process` / `stop_process` / `restart_process`
- `create_terminal` / `close_terminal` / `write_to_terminal`
- `get_session_snapshot`
- Session event listeners

## Frontend Changes

### 1. `client.ts`

Add the `LaunchProjectInfo` interface and update the return type:

```typescript
export interface LaunchProjectInfo {
  project_id: string | null;
  locked: boolean;
}

getLaunchProject: () => invoke<LaunchProjectInfo>("get_launch_project"),
```

### 2. `RuntimeStore` (`src/lib/stores/runtime.svelte.ts`)

New state field:

```typescript
launchLocked = $state<boolean>(false);
```

Modified `#applyLaunchParams()`:

- Call `getLaunchProject()` and read `{ projectId, locked }`.
- If `locked`:
  - Set `this.launchLocked = true`.
  - Set `this.projectId = projectId`.
  - Refresh the project list (read-only display).
  - Do NOT call `startCurrentProject()` (no auto-run).
- If not locked: existing behaviour unchanged.

Frontend guards:

- `saveProject()`: return early if `this.launchLocked`.
- `removeProject()`: return early if `this.launchLocked`.
- Project selection/switch: no-op if `this.launchLocked`.

### 3. UI visibility

When `runtimeStore.launchLocked` is `true`:

| Element | Behaviour |
|---|---|
| `"+"` button (sidebar header) | Hidden |
| "Edit project" in `ProjectMenu` | Hidden |
| "Remove project" in `ProjectSettingsDialog` | Hidden |
| Project list selection | Disabled (visual-only) |

`RunStopButton`, `ConfigEditor`, `ProcessList`, terminal panel, and
log viewer are unchanged.

## Data Flow

```
CLI arg or devapp.yml discovery
        │
        ▼
lib.rs: setup()
  └─ store.import_project_config_path(path)
  └─ *launch_project_id = Some(id)
        │
        ▼
Frontend init: runtimeStore.init()
  └─ getLaunchProject() → { project_id: "abc", locked: true }
  └─ #applyLaunchParams():
       ├─ launchLocked = true
       ├─ projectId = "abc"
       └─ refreshProjects() (read-only)
       └─ (no auto-start)
        │
        ▼
UI renders:
  ├─ "+" hidden
  ├─ "Edit project" hidden
  ├─ "Remove project" hidden
  ├─ Project list non-interactive
  └─ Run button enabled
        │
        ▼
User clicks Run → startProject("abc")
  └─ Backend guard: matches launch_project_id → OK
  └─ Session starts
```

## Non-Goals

- Persistent session locking (locking applies per window, per launch).
- Preventing the user from opening a second Tauri window manually.
- Locking the runtime config editor (`save_project_config`).
- Locking terminal creation/closing.

## Edge Cases

- **No CLI arg, no devapp.yml**: `locked=false`, behaviour unchanged.
- **devapp.yml present but user passes a different config path**:
  the explicit argument wins. `locked=true` for that path.
- **Window is closed and re-opened**: `launch_project_id` is set once
  at app startup. Re-opening from the OS dock/taskbar reuses the same
  state. The lock persists for the app process lifetime.
- **Second Tauri window (manual)**: if the user somehow spawns a new
  Tauri window, it will call `getLaunchProject()` and receive the same
  `{ project_id, locked: true }`. It will also become locked.
- **`open_project_window` when locked**: the backend rejects it. The
  frontend won't expose a trigger button anyway.

## Testing Strategy

### Backend (Rust)

- Unit test: `get_launch_project` returns `locked: true` when
  `launch_project_id` is `Some`.
- Unit test: `get_launch_project` returns `locked: false` when
  `launch_project_id` is `None`.
- Unit test: `save_project` returns `LaunchLocked` error when locked.
- Unit test: `remove_project` returns `LaunchLocked` error when locked.
- Unit test: `start_project` returns `LaunchLocked` for a non-matching
  project ID when locked.
- Unit test: `start_project` succeeds for the locked project ID.

### Frontend (integration / manual)

- Launch app with `devapp examples/deno-runner.yml`: verify no
  auto-run, "+" hidden, edit/remove hidden, run button works.
- Launch app without args: verify "+" visible, can create/edit/delete
  projects.
- Launch with project, stop session, verify can re-run same project.
