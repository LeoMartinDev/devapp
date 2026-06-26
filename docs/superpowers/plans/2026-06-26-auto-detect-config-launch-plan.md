# Auto-Detect Config on Launch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When devapp is launched without a CLI config path, automatically discover and load the nearest `devapp.yml` by walking up from the current directory, falling back to the existing WelcomeScreen when none is found or if the discovered config is invalid.

**Architecture:** Add a small config-discovery helper to the backend, wire it into the existing launch setup path, propagate any launch error through the existing `get_launch_project` command, and display it via the existing frontend error banner.

**Tech Stack:** Rust (Tauri), Svelte 5 / TypeScript, cargo, deno.

## Global Constraints

- Backend is the source of truth for runtime behavior.
- Only `devapp.yml` is searched; no alternate filenames.
- Only linear parent walk-up from cwd; no recursive directory search.
- CLI argument takes priority over auto-detection.
- Auto-detected config behaves exactly like a CLI arg: import, lock, auto-start.
- Invalid auto-detected config must show an error banner and leave the user on the WelcomeScreen with `launchLocked=false`.

---

## Task 1: Add `find_config_in_cwd_or_parents()` helper

**Files:**
- Modify: `src-tauri/src/infrastructure/config_loader.rs`
- Test: `src-tauri/src/infrastructure/config_loader.rs` (existing `tests` module)

**Interfaces:**
- Produces: `pub fn find_config_in_cwd_or_parents() -> Option<PathBuf>`

- [ ] **Step 1: Write the helper**

Insert just after `find_project_config()`:

```rust
/// Walk up from the current working directory to the filesystem root
/// looking for `devapp.yml`. Returns the first match, or `None`.
pub fn find_config_in_cwd_or_parents() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let candidate = current.join(PROJECT_CONFIG_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}
```

- [ ] **Step 2: Add unit tests**

Add to the `tests` module at the end of the file:

```rust
#[test]
fn finds_config_in_cwd() {
    let root = std::env::temp_dir().join(format!(
        "devapp-config-loader-cwd-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&root).expect("create temp dir");
    let config_path = root.join(PROJECT_CONFIG_FILE_NAME);
    fs::write(&config_path, "version: 1\nprocesses: {}\n").expect("write config");

    let original = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(&root).expect("set cwd");
    let result = find_config_in_cwd_or_parents();
    std::env::set_current_dir(original).expect("restore cwd");

    let found = result.expect("should find config");
    assert!(found.is_file());
    assert_eq!(found.file_name().unwrap(), PROJECT_CONFIG_FILE_NAME);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn finds_config_in_parent_directory() {
    let root = std::env::temp_dir().join(format!(
        "devapp-config-loader-parent-{}",
        uuid::Uuid::new_v4()
    ));
    let child = root.join("child");
    fs::create_dir_all(&child).expect("create temp dirs");
    let config_path = root.join(PROJECT_CONFIG_FILE_NAME);
    fs::write(&config_path, "version: 1\nprocesses: {}\n").expect("write config");

    let original = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(&child).expect("set cwd");
    let result = find_config_in_cwd_or_parents();
    std::env::set_current_dir(original).expect("restore cwd");

    let found = result.expect("should find config in parent");
    assert!(found.is_file());
    assert_eq!(found.file_name().unwrap(), PROJECT_CONFIG_FILE_NAME);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn returns_none_when_no_config_found() {
    let root = std::env::temp_dir().join(format!(
        "devapp-config-loader-none-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&root).expect("create temp dir");

    let original = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(&root).expect("set cwd");
    let result = find_config_in_cwd_or_parents();
    std::env::set_current_dir(original).expect("restore cwd");

    assert_eq!(result, None);
    let _ = fs::remove_dir_all(root);
}
```

- [ ] **Step 3: Run the new tests**

Run:

```bash
cd src-tauri && cargo test config_loader::tests::finds_config
```

Expected: three tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/infrastructure/config_loader.rs
git commit -m "feat: add find_config_in_cwd_or_parents helper"
```

---

## Task 2: Add `launch_error` field to AppState

**Files:**
- Modify: `src-tauri/src/tauri_api/state.rs`

**Interfaces:**
- Produces: `pub launch_error: Arc<Mutex<Option<String>>>` on `AppState`

- [ ] **Step 1: Add the field**

Modify `AppState`:

```rust
pub struct AppState {
    pub orchestrator: ProcessOrchestrator,
    pub project_store: Arc<Mutex<ProjectStore>>,
    pub terminal_manager: TerminalManager,
    pub launch_project_id: Arc<Mutex<Option<ProjectId>>>,
    pub launch_error: Arc<Mutex<Option<String>>>,
}
```

Modify `AppState::new()`:

```rust
Ok(Self {
    orchestrator: ProcessOrchestrator::new(),
    project_store: Arc::new(Mutex::new(ProjectStore::new()?)),
    terminal_manager: TerminalManager::new(),
    launch_project_id: Arc::new(Mutex::new(None)),
    launch_error: Arc::new(Mutex::new(None)),
})
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/tauri_api/state.rs
git commit -m "feat: add launch_error to AppState"
```

---

## Task 3: Expose launch error in `LaunchProjectInfo`

**Files:**
- Modify: `src-tauri/src/tauri_api/commands.rs`

**Interfaces:**
- Consumes: `state.launch_error`
- Produces: `LaunchProjectInfo.error: Option<String>`

- [ ] **Step 1: Extend the struct**

Change:

```rust
pub struct LaunchProjectInfo {
    pub project_id: Option<ProjectId>,
    pub locked: bool,
}
```

To:

```rust
pub struct LaunchProjectInfo {
    pub project_id: Option<ProjectId>,
    pub locked: bool,
    pub error: Option<String>,
}
```

- [ ] **Step 2: Populate the error field**

Change `get_launch_project`:

```rust
#[tauri::command]
pub async fn get_launch_project(
    state: State<'_, AppState>,
) -> Result<LaunchProjectInfo, String> {
    let launch_project_id = state.launch_project_id.lock().await;
    let launch_error = state.launch_error.lock().await;
    Ok(LaunchProjectInfo {
        project_id: launch_project_id.clone(),
        locked: launch_project_id.is_some(),
        error: launch_error.clone(),
    })
}
```

- [ ] **Step 3: Run backend tests**

Run:

```bash
cd src-tauri && cargo test
```

Expected: all tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tauri_api/commands.rs
git commit -m "feat: expose launch_error through get_launch_project"
```

---

## Task 4: Integrate auto-detection into launch setup

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `find_config_in_cwd_or_parents()` and `state.launch_error`

- [ ] **Step 1: Add the import**

At the top of `src-tauri/src/lib.rs`, add:

```rust
use crate::infrastructure::config_loader::find_config_in_cwd_or_parents;
```

- [ ] **Step 2: Replace the launch setup block**

Change:

```rust
if let Some(config_path) = launch_config_path() {
    let project = {
        let store = state.project_store.lock().await;
        store.import_project_config_path(config_path)?
    };
    let mut launch_project_id = state.launch_project_id.lock().await;
    *launch_project_id = Some(project.id);
}
```

To:

```rust
let config_path = launch_config_path()
    .or_else(|| find_config_in_cwd_or_parents());

if let Some(config_path) = config_path {
    match state.project_store.lock().await
        .import_project_config_path(&config_path)
    {
        Ok(project) => {
            let mut launch_project_id = state.launch_project_id.lock().await;
            *launch_project_id = Some(project.id);
        }
        Err(error) => {
            let mut launch_error = state.launch_error.lock().await;
            *launch_error = Some(format!(
                "Failed to load auto-detected config {}: {}",
                config_path.display(), error
            ));
        }
    }
}
```

- [ ] **Step 3: Run backend tests**

Run:

```bash
cd src-tauri && cargo test
```

Expected: all tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: auto-detect devapp.yml on launch when no CLI arg given"
```

---

## Task 5: Update frontend types and mock

**Files:**
- Modify: `src/lib/tauri/client.ts`
- Modify: `src/lib/tauri/mock/transport.ts`

**Interfaces:**
- Produces: `LaunchProjectInfo.error?: string | null`

- [ ] **Step 1: Extend the frontend interface**

In `src/lib/tauri/client.ts`:

```typescript
export interface LaunchProjectInfo {
  projectId: string | null;
  locked: boolean;
  error?: string | null;
}
```

- [ ] **Step 2: Fix the mock transport return shape**

In `src/lib/tauri/mock/transport.ts`, change the `get_launch_project` case from:

```typescript
case "get_launch_project":
  return mockProjects[0].record.id as unknown as T;
```

To:

```typescript
case "get_launch_project":
  return {
    projectId: mockProjects[0]?.record.id ?? null,
    locked: false,
    error: null,
  } as unknown as T;
```

- [ ] **Step 3: Run frontend type check**

Run:

```bash
deno task check
```

Expected: exits successfully. (Ignore the known non-blocking `tsconfig.json` extension warning.)

- [ ] **Step 4: Commit**

```bash
git add src/lib/tauri/client.ts src/lib/tauri/mock/transport.ts
git commit -m "feat: add error field to LaunchProjectInfo and fix mock"
```

---

## Task 6: Display launch error in the runtime store

**Files:**
- Modify: `src/lib/stores/runtime.svelte.ts`

**Interfaces:**
- Consumes: `launchInfo.error`

- [ ] **Step 1: Set uiError from launch error**

In `#applyLaunchParams()`, after:

```typescript
const launchInfo = await getLaunchProject();
this.launchLocked = launchInfo.locked;
```

Add:

```typescript
this.uiError = launchInfo.error ?? null;
```

- [ ] **Step 2: Run frontend type check**

Run:

```bash
deno task check
```

Expected: exits successfully.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/runtime.svelte.ts
git commit -m "feat: display auto-detect config errors in uiError banner"
```

---

## Task 7: Full validation

- [ ] **Step 1: Backend tests**

Run:

```bash
cd src-tauri && cargo test
```

Expected: all tests PASS.

- [ ] **Step 2: Frontend checks**

Run:

```bash
deno task check
```

Expected: exits successfully (known tsconfig warning is acceptable).

- [ ] **Step 3: Manual smoke test (optional but recommended)**

From the project root:

```bash
# Valid config in cwd
deno task app
```

Then try from a directory without `devapp.yml`:

```bash
cd /tmp && mkdir devapp-smoke && cd devapp-smoke && /path/to/devapp/binary
```

(Exact binary path depends on Tauri build; use `cargo tauri dev` from the target workspace during development.)

- [ ] **Step 4: Final commit if any changes**

If smoke tests produced fixes, commit them. Otherwise no action.

---

## Plan Self-Review

- **Spec coverage:**
  - Auto-detection function: Task 1
  - Parent walk-up from cwd: Task 1
  - CLI arg priority: Task 4 via `or_else`
  - Import/lock/auto-start on success: Task 4
  - Error banner on invalid config: Tasks 2, 3, 6
  - WelcomeScreen fallback when no config: no code change needed
- **Placeholder scan:** No TBD/TODO/vague steps. Each step includes actual code or exact commands.
- **Type consistency:** `LaunchProjectInfo.error` matches `Option<String>` in Rust and `string | null | undefined` in TypeScript. `find_config_in_cwd_or_parents` returns `Option<PathBuf>`.
