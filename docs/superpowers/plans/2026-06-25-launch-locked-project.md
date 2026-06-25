# Launch-Locked Project Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lock project management (create, edit, delete, switch) when devapp is launched with a project via CLI argument or `devapp.yml` discovery. Remove auto-run behaviour.

**Architecture:** The backend stores `launch_project_id` in `AppState` and exposes a `locked` flag through an enriched `get_launch_project` command. Mutating commands (`save_project`, `remove_project`, `start_project`, `open_project_window`) reject when locked. The frontend derives `launchLocked` from the command response and conditionally hides project management UI while keeping runtime operations available.

**Tech Stack:** Rust (Tauri backend), TypeScript/Svelte 5 (frontend)

## Global Constraints

- Backend is source of truth for locking decision
- `launchLocked` applies per app process (all windows share the same `AppState`)
- Runtime operations (run/stop session, start/stop processes, terminals, config editing, logs) are never locked
- `save_project_config` and `load_project_config` are not blocked

---

### Task 1: Backend — `LaunchLocked` error variant and `LaunchProjectInfo` struct

**Files:**
- Modify: `src-tauri/src/error.rs:5-19`
- Modify: `src-tauri/src/tauri_api/commands.rs:104-108`
- Modify: `src-tauri/src/domain/project.rs:1-34`

**Interfaces:**
- Produces: `AppError::LaunchLocked` error variant, `LaunchProjectInfo` struct with fields `project_id: Option<ProjectId>` and `locked: bool`

---

- [ ] **Step 1: Add `LaunchLocked` variant to `AppError`**

In `src-tauri/src/error.rs`, add after line 18:

```rust
#[error("project is launch-locked and cannot be modified")]
LaunchLocked,
```

Expected diff around lines 17-19:

```rust
    #[error("terminal error: {0}")]
    Terminal(String),
    #[error("project is launch-locked and cannot be modified")]
    LaunchLocked,
}
```

- [ ] **Step 2: Add `LaunchProjectInfo` struct in `commands.rs`**

In `src-tauri/src/tauri_api/commands.rs`, after line 46 (`SaveProjectConfigRequest` struct), add:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchProjectInfo {
    pub project_id: Option<ProjectId>,
    pub locked: bool,
}
```

- [ ] **Step 3: Modify `get_launch_project` to return `LaunchProjectInfo`**

In `src-tauri/src/tauri_api/commands.rs`, replace lines 104-108:

```rust
#[tauri::command]
pub async fn get_launch_project(state: State<'_, AppState>) -> Result<Option<ProjectId>, String> {
    let launch_project_id = state.launch_project_id.lock().await;
    Ok(launch_project_id.clone())
}
```

With:

```rust
#[tauri::command]
pub async fn get_launch_project(
    state: State<'_, AppState>,
) -> Result<LaunchProjectInfo, String> {
    let launch_project_id = state.launch_project_id.lock().await;
    Ok(LaunchProjectInfo {
        project_id: launch_project_id.clone(),
        locked: launch_project_id.is_some(),
    })
}
```

- [ ] **Step 4: Add `ProjectId` import if not already imported**

Check line 3 of `commands.rs` — `ProjectId` is already imported via `use crate::domain::project::{ProjectId, ProjectRecord, ProjectSource};`. No change needed.

- [ ] **Step 5: Build check**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compiles successfully.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/tauri_api/commands.rs
git commit -m "feat: add LaunchLocked error and LaunchProjectInfo struct"
```

---

### Task 2: Backend — Guards on mutating commands when locked

**Files:**
- Modify: `src-tauri/src/tauri_api/commands.rs:110-125` (`save_project`)
- Modify: `src-tauri/src/tauri_api/commands.rs:127-134` (`remove_project`)
- Modify: `src-tauri/src/tauri_api/commands.rs:184-205` (`start_project`)
- Modify: `src-tauri/src/tauri_api/commands.rs:348-376` (`open_project_window`)

**Interfaces:**
- Consumes: `AppError::LaunchLocked` from Task 1

---

- [ ] **Step 1: Add `launch_locked` helper function**

In `src-tauri/src/tauri_api/commands.rs`, after the `window_key` function (line 96), add:

```rust
async fn check_launch_locked(state: &AppState) -> Result<(), String> {
    if state.launch_project_id.lock().await.is_some() {
        return Err(AppError::LaunchLocked.to_string());
    }
    Ok(())
}
```

- [ ] **Step 2: Guard `save_project`**

In `src-tauri/src/tauri_api/commands.rs`, at line 115 (after the function body opens), add the check:

```rust
#[tauri::command]
pub async fn save_project(
    state: State<'_, AppState>,
    request: SaveProjectRequest,
) -> Result<ProjectRecord, String> {
    check_launch_locked(&state).await?;
    let store = state.project_store.lock().await;
    // ... rest unchanged
```

- [ ] **Step 3: Guard `remove_project`**

At line 132 in `remove_project`, add the check:

```rust
#[tauri::command]
pub async fn remove_project(
    state: State<'_, AppState>,
    project_id: ProjectId,
) -> Result<(), String> {
    check_launch_locked(&state).await?;
    let store = state.project_store.lock().await;
    // ... rest unchanged
```

- [ ] **Step 4: Guard `start_project` — reject mismatched project ID**

In `start_project` (line 185), add a check that verifies the project_id matches the locked project:

```rust
#[tauri::command]
pub async fn start_project(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    request: ProjectActionRequest,
) -> Result<RunSessionSnapshot, String> {
    {
        let locked_id = state.launch_project_id.lock().await;
        if let Some(locked) = locked_id.as_ref() {
            if locked != &request.project_id {
                return Err(AppError::LaunchLocked.to_string());
            }
        }
    }

    let project = {
        // ... rest unchanged
```

The extra block `{}` ensures the lock is dropped before the rest of the function continues.

- [ ] **Step 5: Guard `open_project_window`**

In `open_project_window` (line 349), add the lock guard:

```rust
#[tauri::command]
pub async fn open_project_window(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: ProjectActionRequest,
) -> Result<(), String> {
    check_launch_locked(&state).await?;

    let project = {
        // ... rest unchanged
```

- [ ] **Step 6: Build check**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compiles successfully.

- [ ] **Step 7: Run backend tests**

Run: `cd src-tauri && cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/tauri_api/commands.rs
git commit -m "feat: guard mutating commands with launch-lock check"
```

---

### Task 3: Frontend — Update `client.ts` type and return type

**Files:**
- Modify: `src/lib/tauri/client.ts:29-31`

**Interfaces:**
- Consumes: `LaunchProjectInfo` from Task 1
- Produces: `LaunchProjectInfo` interface, updated `getLaunchProject` return type

---

- [ ] **Step 1: Add `LaunchProjectInfo` interface and update `getLaunchProject`**

In `src/lib/tauri/client.ts`, replace lines 29-31:

```typescript
export async function getLaunchProject(): Promise<ProjectId | null> {
  return invoke<ProjectId | null>("get_launch_project");
}
```

With:

```typescript
export interface LaunchProjectInfo {
  project_id: string | null;
  locked: boolean;
}

export async function getLaunchProject(): Promise<LaunchProjectInfo> {
  return invoke<LaunchProjectInfo>("get_launch_project");
}
```

- [ ] **Step 2: Typecheck**

Run: `deno task check`
Expected: Passes (or only shows the known non-blocking SvelteKit tsconfig warning).

- [ ] **Step 3: Commit**

```bash
git add src/lib/tauri/client.ts
git commit -m "feat: add LaunchProjectInfo type and update getLaunchProject"
```

---

### Task 4: Frontend — `RuntimeStore` changes (launchLocked state, no auto-run, guards)

**Files:**
- Modify: `src/lib/stores/runtime.svelte.ts` (multiple sections)

**Interfaces:**
- Consumes: `LaunchProjectInfo` from Task 3
- Produces: `launchLocked` state field, modified `#applyLaunchParams`, guarded `saveProject` and `removeProject`

---

- [ ] **Step 1: Import `LaunchProjectInfo` type**

In `src/lib/stores/runtime.svelte.ts`, add `LaunchProjectInfo` to the import from `$lib/tauri/client` on line 24:

Current line 24:
```typescript
  type SaveProjectInput,
```

Change to:
```typescript
  type SaveProjectInput,
  type LaunchProjectInfo,
```

- [ ] **Step 2: Add `launchLocked` state field**

In the `RuntimeStore` class, after line 53 (`projectId` field), add:

```typescript
  launchLocked = $state<boolean>(false);
```

- [ ] **Step 3: Modify `#applyLaunchParams()` to not auto-run when locked**

Replace lines 502-517:

```typescript
  async #applyLaunchParams() {
    if (typeof window === "undefined") {
      return;
    }
    const params = new URLSearchParams(window.location.search);
    const launchProjectId = await getLaunchProject();
    const projectId = params.get("projectId") ?? launchProjectId;
    const autorun = params.get("autorun") === "1";
    if (!projectId || !this.projects.some((project) => project.id === projectId)) {
      return;
    }
    this.projectId = projectId;
    if ((autorun || launchProjectId === projectId) && !this.session) {
      await this.startCurrentProject();
    }
  }
```

With:

```typescript
  async #applyLaunchParams() {
    if (typeof window === "undefined") {
      return;
    }
    const params = new URLSearchParams(window.location.search);
    const urlProjectId = params.get("projectId");
    const launchInfo = await getLaunchProject();
    this.launchLocked = launchInfo.locked;

    if (launchInfo.locked) {
      if (
        launchInfo.project_id &&
        this.projects.some((p) => p.id === launchInfo.project_id)
      ) {
        this.projectId = launchInfo.project_id;
      }
      return;
    }

    const projectId = urlProjectId ?? launchInfo.project_id;
    const autorun = params.get("autorun") === "1";
    if (!projectId || !this.projects.some((project) => project.id === projectId)) {
      return;
    }
    this.projectId = projectId;
    if ((autorun || launchInfo.project_id === projectId) && !this.session) {
      await this.startCurrentProject();
    }
  }
```

- [ ] **Step 4: Guard `saveProject()`**

In `saveProject()` (line 191), add early return when locked:

```typescript
  async saveProject(input: SaveProjectInput) {
    if (this.launchLocked) {
      this.setError("Project is launch-locked and cannot be modified.");
      return;
    }
    this.busy = true;
    // ... rest unchanged
```

- [ ] **Step 5: Guard `removeProject()`**

In `removeProject()` (line 206), add early return when locked:

```typescript
  async removeProject(projectId: ProjectId) {
    if (this.launchLocked) {
      this.setError("Project is launch-locked and cannot be modified.");
      return;
    }
    this.busy = true;
    // ... rest unchanged
```

- [ ] **Step 6: Guard `refreshProjects()` fallback when locked**

In `refreshProjects()` (line 174), the method has a fallback that sets `this.projectId = this.projects[0].id` when the current projectId is not found. When locked, we should not switch to a different project.

Replace lines 174-189:

```typescript
  async refreshProjects() {
    try {
      this.projects = await listProjects();
      if (!this.projectId && this.projects.length > 0) {
        this.projectId = this.projects[0].id;
      }
      if (
        this.projectId &&
        !this.projects.some((project) => project.id === this.projectId)
      ) {
        this.projectId = this.projects[0]?.id ?? null;
      }
    } catch (error) {
      this.setError(error);
    }
  }
```

With:

```typescript
  async refreshProjects() {
    try {
      this.projects = await listProjects();
      if (this.launchLocked) {
        return;
      }
      if (!this.projectId && this.projects.length > 0) {
        this.projectId = this.projects[0].id;
      }
      if (
        this.projectId &&
        !this.projects.some((project) => project.id === this.projectId)
      ) {
        this.projectId = this.projects[0]?.id ?? null;
      }
    } catch (error) {
      this.setError(error);
    }
  }
```

- [ ] **Step 7: Typecheck**

Run: `deno task check`
Expected: Passes (or only the known non-blocking SvelteKit tsconfig warning).

- [ ] **Step 8: Commit**

```bash
git add src/lib/stores/runtime.svelte.ts
git commit -m "feat: add launchLocked state, remove auto-run, guard project ops"
```

---

### Task 5: Frontend — UI changes (hide project management elements)

**Files:**
- Modify: `src/routes/+page.svelte:123-126,153-173` (openCreateDialog guard, sidebarHeader)
- Modify: `src/lib/components/ProjectMenu.svelte:1-87` (hide Edit project when locked)
- Modify: `src/lib/components/ProjectSettingsDialog.svelte:1-160` (optional: guard openCreateDialog from within)

**Interfaces:**
- Consumes: `runtimeStore.launchLocked` from Task 4

---

- [ ] **Step 1: Guard `openCreateDialog` in `+page.svelte`**

In `src/routes/+page.svelte`, modify `openCreateDialog` (line 123):

```typescript
  function openCreateDialog() {
    if (runtimeStore.launchLocked) return;
    editingProject = null;
    detailsOpen = true;
  }
```

- [ ] **Step 2: Guard `openEditDialog` in `+page.svelte`**

In `src/routes/+page.svelte`, modify `openEditDialog` (line 128):

```typescript
  function openEditDialog(openedProject: ProjectRecord) {
    if (runtimeStore.launchLocked) return;
    editingProject = openedProject;
    detailsOpen = true;
  }
```

- [ ] **Step 3: Hide "+" button when locked**

In `src/routes/+page.svelte`, the `sidebarHeader` snippet (lines 153-173). Hide the "+" IconButton when locked — wrap it in an `{#if}`:

Replace lines 158-161:

```svelte
          <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
            +
          </IconButton>
```

With:

```svelte
          {#if !runtimeStore.launchLocked}
            <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
              +
            </IconButton>
          {/if}
```

- [ ] **Step 4: Hide "Edit project" in `ProjectMenu.svelte`**

The `ProjectMenu` needs access to `launchLocked`. Add a new prop and conditionally exclude the "Edit project" item.

Add `launchLocked` prop declaration after existing props in `ProjectMenu.svelte`:

In the `Props` type (lines 13-26), add:

```typescript
    launchLocked: boolean;
```

In the destructured `let` block (lines 28-41), add:

```typescript
    launchLocked,
```

Now modify `buildItems()` to skip "Edit project" when locked. Replace lines 50-53:

```typescript
    if (project) {
      built.push({ label: "Edit project", onSelect: () => onEditProject(project) });
      built.push({ label: "Runtime config", onSelect: onOpenConfig, dividerAfter: true });
    }
```

With:

```typescript
    if (project) {
      if (!launchLocked) {
        built.push({ label: "Edit project", onSelect: () => onEditProject(project) });
      }
      built.push({ label: "Runtime config", onSelect: onOpenConfig, dividerAfter: true });
    }
```

- [ ] **Step 5: Pass `launchLocked` to `ProjectMenu` in `+page.svelte`**

In `src/routes/+page.svelte`, add the prop to the `ProjectMenu` call (around line 245):

```svelte
          <ProjectMenu
            {project}
            {selection}
            {selectedProcess}
            {selectedTerminal}
            busy={runtimeStore.busy}
            {logActions}
            launchLocked={runtimeStore.launchLocked}
            onEditProject={openEditDialog}
            onOpenConfig={openConfigDialog}
            onRestartProcess={(name) => runtimeStore.restartSessionProcess(name)}
            onStopProcess={(name) => runtimeStore.stopSessionProcess(name)}
            onCloseTerminal={() => runtimeStore.closeSelectedTerminal()}
            onOpenTerminal={openTerminal}
          />
```

- [ ] **Step 6: Typecheck**

Run: `deno task check`
Expected: Passes (or only the known non-blocking SvelteKit tsconfig warning).

- [ ] **Step 7: Commit**

```bash
git add src/routes/+page.svelte src/lib/components/ProjectMenu.svelte
git commit -m "feat: hide project management UI when launch-locked"
```

---

### Task 6: Integration test — launch with project, verify lock

Manual verification steps. No automated test changes needed (there is no existing E2E test infrastructure).

- [ ] **Step 1: Launch with a project config**

Run: `deno task app examples/deno-runner.yml`
Expected:
  - Window opens, project name shown (not auto-run)
  - "+" button is not visible
  - Gear menu: "Runtime config" is present, "Edit project" is not present
  - Clicking Run starts the session

- [ ] **Step 2: Verify cannot open a second project window via backend**

The `open_project_window` command has no frontend wrapper, so this is a backend-only guard. Verified by build.

- [ ] **Step 3: Launch without a project**

Run: `deno task app` (no arguments, and ensure no `devapp.yml` auto-detected)
Expected:
  - Window opens to empty state
  - "+" button is visible
  - Can create a new project via the dialog
  - "Edit project" is visible in the gear menu

- [ ] **Step 4: Commit (if any doc/readme updates needed)**

No commit needed for verification step.
