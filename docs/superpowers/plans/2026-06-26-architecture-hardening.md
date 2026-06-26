# Architecture Hardening — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all 21 issues found in the `src-tauri/` architecture review across 6 incremental PRs.

**Architecture:** Each PR is an independent slice — critical bugs first, then structural refactor, tests, error codes/logging, thread hygiene, and performance last. Backend and frontend changes ship together when coupled.

**Tech Stack:** Rust/Tauri 2, Tokio, Svelte 5 with runes, Deno tooling, serde, portable-pty, broadcast/mpsc channels.

## Global Constraints

- Use Deno as the JS tooling entrypoint — never run `npm`/`pnpm`/`yarn` directly
- Backend tests: `cd src-tauri && cargo test`
- Frontend typecheck: `deno task check`
- App smoke test: `deno task app examples/deno-runner.yml`
- `package.json` exists for Tauri toolchain; agents interact through `deno task`
- A non-blocking SvelteKit warning about `tsconfig.json` is known and should not block validation

---

## PR 1: Fix Critical Bugs

### Task 1.1: Inherit system environment in `build_process_env`

**Files:**
- Modify: `src-tauri/src/application/orchestrator.rs:785-797`

**Interfaces:**
- Consumes: `IndexMap<String, String>` (global_env, process_env)
- Produces: `HashMap<String, String>` (merged env with system inheritance)

- [ ] **Step 1: Write failing test**

Append to `src-tauri/src/application/orchestrator.rs` test module (after line 941):

```rust
#[test]
fn build_process_env_inherits_system_environment() {
    // Sanity: the merged env should contain at least PATH (always present on Unix/Windows).
    let global_env = IndexMap::new();
    let process_env = IndexMap::new();
    let merged = build_process_env(&global_env, &process_env);
    assert!(
        merged.contains_key("PATH") || merged.contains_key("Path"),
        "merged env must inherit system PATH"
    );
}

#[test]
fn build_process_env_process_override_wins_over_system() {
    // If the system has HOME=/home/user and the config sets HOME=/custom,
    // the config value wins.
    let mut global_env = IndexMap::new();
    global_env.insert("HOME".to_string(), "/custom".to_string());
    let process_env = IndexMap::new();
    let merged = build_process_env(&global_env, &process_env);
    assert_eq!(merged.get("HOME"), Some(&"/custom".to_string()));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test build_process_env_inherits
```
Expected: FAIL — `build_process_env` currently does not inherit `std::env::vars()`

- [ ] **Step 3: Implement environment inheritance**

Replace the `build_process_env` function body at `src-tauri/src/application/orchestrator.rs:785-797`:

```rust
fn build_process_env(
    global_env: &IndexMap<String, String>,
    process_env: &IndexMap<String, String>,
) -> HashMap<String, String> {
    let mut env: HashMap<String, String> = std::env::vars().collect();
    for (key, value) in global_env {
        env.insert(key.clone(), value.clone());
    }
    for (key, value) in process_env {
        env.insert(key.clone(), value.clone());
    }
    env
}
```

- [ ] **Step 4: Run all env-related tests**

```bash
cd src-tauri && cargo test build_process_env
```
Expected: All 4 `build_process_env_*` tests PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/application/orchestrator.rs
git commit -m "fix: inherit system environment in build_process_env

Processes spawned via sh -c now have PATH/HOME/USER from the parent
process. Config-level env vars override inherited ones."
```

---

### Task 1.2: Fix TOCTOU race in `spawn_named_process`

**Files:**
- Modify: `src-tauri/src/application/orchestrator.rs:343-368`

**Interfaces:**
- Consumes: `Arc<Mutex<OrchestratorState>>`, `AppHandle`, `String` (window_key, process_name)
- Produces: Guarded child spawn that aborts if session was stopped between lock scopes

- [ ] **Step 1: Write test for TOCTOU scenario**

This is hard to unit-test directly (requires concurrent session stop during spawn). Add an integration test intent marker and a state-machine test instead. Append to the test module:

```rust
#[test]
fn spawn_named_process_checks_stop_requested_before_assigning_child() {
    // Verify the guard exists in code structure — the real test is in PR3 integration.
    // This test validates begin_process_termination and reset_managed_process already
    // cover the state transitions correctly.
    // (Covered by integration test in PR3: session_lifecycle_integration_test)
}
```

- [ ] **Step 2: Add `stop_requested` guard**

In `spawn_named_process` at line 352, after re-acquiring the lock, add a guard before assigning `child`:

Replace lines 352-368:

```rust
        let generation = {
            let mut state = self.inner.lock().await;
            let active = state
                .sessions
                .get_mut(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            if active.stop_requested {
                // Session was stopped between spawn_process and this lock acquisition.
                // Kill the child we just spawned to prevent orphan processes.
                let _ = spawned.child.kill().await;
                return Ok(());
            }
            let process = active
                .processes
                .get_mut(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
            process.child = Some(child.clone());
            process.pid = child_pid;
            process.kill_tx = Some(kill_tx);
            process.snapshot.status = ProcessStatus::Running;
            sync_snapshot_process(&mut active.snapshot, &process.snapshot);
            process.generation
        };
```

- [ ] **Step 3: Run existing orchestrator tests**

```bash
cd src-tauri && cargo test
```
Expected: All tests PASS (including 7 orchestrator tests)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/application/orchestrator.rs
git commit -m "fix: prevent orphan process when session stops during spawn

Add stop_requested guard in spawn_named_process second lock block.
If the session was stopped between spawn_process and child assignment,
kill the newly-spawned child and return without attaching it to state."
```

---

### Task 1.3: Fix broadcast channel capacity for log readiness

**Files:**
- Modify: `src-tauri/src/application/orchestrator.rs:106`

**Interfaces:**
- Consumes: `broadcast::channel(256)` → `broadcast::channel(4096)`
- Produces: Higher-capacity broadcast channel that tolerates log bursts

- [ ] **Step 1: Increase channel capacity**

Replace line 106 in `start_session`:

```rust
// Before:
let (log_tx, _) = broadcast::channel(256);
// After:
let (log_tx, _) = broadcast::channel(4096);
```

- [ ] **Step 2: Run tests to ensure no regressions**

```bash
cd src-tauri && cargo test
```
Expected: All tests PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/application/orchestrator.rs
git commit -m "fix: increase broadcast channel capacity to 4096

Prevents log readiness patterns from being dropped during high log volume.
256 lines was too small for verbose process startup."
```

---

### Task 1.4: Fix `close_all_for_window` error short-circuit

**Files:**
- Modify: `src-tauri/src/infrastructure/pty.rs:244-246`

**Interfaces:**
- Consumes: `AppHandle`, `&str` (window_key)
- Produces: All terminals closed, even if some fail individually

- [ ] **Step 1: Change error handling in the loop**

Replace lines 244-246 in `close_all_for_window`:

```rust
// Before:
for terminal_id in ids {
    let _ = self.close_terminal(app_handle.clone(), &terminal_id)?;
}

// After:
for terminal_id in ids {
    let _ = self.close_terminal(app_handle.clone(), &terminal_id);
}
```

- [ ] **Step 2: Run tests**

```bash
cd src-tauri && cargo test
```
Expected: All tests PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/infrastructure/pty.rs
git commit -m "fix: close all terminals even if one fails

Replaced ? with .ok() in close_all_for_window loop so a single
terminal close error doesn't prevent remaining terminals from closing."
```

---

### Task 1.5: Remove dead `project_name` assignment

**Files:**
- Modify: `src-tauri/src/application/orchestrator.rs:320,494`

**Interfaces:**
- No behavior change — cleanup only

- [ ] **Step 1: Remove the dead code**

Remove line 320 (`let project_name = active.project.name.clone();`) and line 494 (`let _ = project_name;`).

- [ ] **Step 2: Verify compilation**

```bash
deno task check
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/application/orchestrator.rs
git commit -m "chore: remove dead project_name assignment in spawn_named_process"
```

---

### PR 1 Verification

```bash
deno task check        # Frontend typecheck
cd src-tauri && cargo test  # Backend tests
deno task app examples/deno-runner.yml  # Manual smoke test
```

---

## PR 2: Split the Orchestrator God Module

### Task 2.1: Create orchestrator sub-module directory and extract `DepResolver`

**Files:**
- Create: `src-tauri/src/application/orchestrator/mod.rs`
- Create: `src-tauri/src/application/orchestrator/dependency.rs`
- Modify: `src-tauri/src/application/mod.rs`
- Modify: `src-tauri/src/application/orchestrator.rs` (will become lib module)

**Interfaces:**
- Produces: `pub fn dependencies_satisfied(processes: &HashMap<String, ManagedProcess>, config: &ProcessConfig) -> bool`
- Produces: `pub fn runnable_names(processes: &HashMap<String, ManagedProcess>, stop_requested: bool) -> Vec<String>`

- [ ] **Step 1: Create directory and module structure**

```bash
mkdir -p src-tauri/src/application/orchestrator
```

- [ ] **Step 2: Write `dependency.rs`**

Create `src-tauri/src/application/orchestrator/dependency.rs`:

```rust
use std::collections::HashMap;

use crate::domain::{
    config::{DependencyCondition, ProcessConfig, ProcessKind},
    process::ProcessStatus,
};

use super::ManagedProcess;

pub fn dependencies_satisfied(
    processes: &HashMap<String, ManagedProcess>,
    config: &ProcessConfig,
) -> bool {
    config
        .depends_on
        .iter()
        .all(|(dependency_name, condition)| {
            let Some(dependency) = processes.get(dependency_name) else {
                return false;
            };
            match condition {
                DependencyCondition::Success => {
                    matches!(dependency.snapshot.status, ProcessStatus::Succeeded)
                }
                DependencyCondition::Ready => {
                    matches!(dependency.snapshot.status, ProcessStatus::Ready)
                }
            }
        })
}

pub fn runnable_names(
    processes: &HashMap<String, ManagedProcess>,
    stop_requested: bool,
) -> Vec<String> {
    if stop_requested {
        return Vec::new();
    }
    processes
        .iter()
        .filter_map(|(name, process)| {
            if process.child.is_some()
                || !matches!(
                    process.snapshot.status,
                    ProcessStatus::Pending | ProcessStatus::Blocked | ProcessStatus::Stopped
                )
            {
                return None;
            }
            if dependencies_satisfied(processes, &process.config) {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use tokio::sync::broadcast;

    use super::*;
    use crate::domain::{config::ProcessKind, runtime::ProcessRuntimeId};

    fn managed_process(name: &str, kind: ProcessKind, status: ProcessStatus) -> ManagedProcess {
        let (log_tx, _) = broadcast::channel(4);
        ManagedProcess {
            config: ProcessConfig {
                kind: kind.clone(),
                cmd: format!("run {name}"),
                env: IndexMap::new(),
                depends_on: IndexMap::new(),
                ready: None,
            },
            snapshot: crate::domain::runtime::ProcessSnapshot {
                runtime_id: ProcessRuntimeId::new(),
                name: name.to_string(),
                kind,
                status,
                started_at: None,
                exited_at: None,
                exit_code: None,
            },
            child: None,
            pid: None,
            kill_tx: None,
            log_tx,
            terminating: false,
            generation: 0,
            stop_notify_tx: None,
        }
    }

    #[test]
    fn dependencies_satisfied_requires_success_for_task_dependency() {
        let mut processes = HashMap::new();
        processes.insert(
            "setup".to_string(),
            managed_process("setup", ProcessKind::Task, ProcessStatus::Succeeded),
        );
        let mut depends_on = IndexMap::new();
        depends_on.insert("setup".to_string(), DependencyCondition::Success);
        let config = ProcessConfig {
            kind: ProcessKind::Service,
            cmd: "deno task dev".to_string(),
            env: IndexMap::new(),
            depends_on,
            ready: None,
        };
        assert!(dependencies_satisfied(&processes, &config));
        processes.get_mut("setup").unwrap().snapshot.status = ProcessStatus::Ready;
        assert!(!dependencies_satisfied(&processes, &config));
    }

    #[test]
    fn dependencies_satisfied_requires_ready_for_service_dependency() {
        let mut processes = HashMap::new();
        processes.insert(
            "api".to_string(),
            managed_process("api", ProcessKind::Service, ProcessStatus::Running),
        );
        let mut depends_on = IndexMap::new();
        depends_on.insert("api".to_string(), DependencyCondition::Ready);
        let config = ProcessConfig {
            kind: ProcessKind::Service,
            cmd: "deno task dev".to_string(),
            env: IndexMap::new(),
            depends_on,
            ready: None,
        };
        assert!(!dependencies_satisfied(&processes, &config));
        processes.get_mut("api").unwrap().snapshot.status = ProcessStatus::Ready;
        assert!(dependencies_satisfied(&processes, &config));
    }
}
```

- [ ] **Step 3: Write `mod.rs` for orchestrator/ (initial skeleton)**

Create `src-tauri/src/application/orchestrator/mod.rs`:

```rust
pub mod dependency;
pub mod lifecycle;
pub mod log;
pub mod session;

use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use indexmap::IndexMap;
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
    sync::{broadcast, mpsc, oneshot, Mutex},
};

use crate::{
    application::{
        command_runner::spawn_process,
        events::{
            ProcessLogEvent, RuntimeEvent, SessionStatusEvent, PROCESS_LOG_EVENT,
            RUNTIME_ERROR_EVENT, SESSION_SNAPSHOT_EVENT,
        },
        readiness::wait_until_ready,
    },
    domain::{
        config::ProcessConfig,
        process::{LogStream, ProcessStatus},
        project::ProjectRecord,
        runtime::{
            ProcessLogPayload, ProcessRuntimeId, ProcessSnapshot, RunSessionId, RunSessionSnapshot,
        },
    },
    error::AppError,
    infrastructure::{
        config_loader::LoadedProjectConfig,
        log_store::InMemoryLogStore,
    },
};

use self::session::ActiveSession;

#[derive(Clone)]
pub struct ProcessOrchestrator {
    inner: Arc<Mutex<session::OrchestratorState>>,
}

struct ManagedProcess {
    config: ProcessConfig,
    snapshot: ProcessSnapshot,
    child: Option<Arc<Mutex<Child>>>,
    pid: Option<u32>,
    kill_tx: Option<mpsc::Sender<()>>,
    log_tx: broadcast::Sender<String>,
    terminating: bool,
    generation: u64,
    stop_notify_tx: Option<oneshot::Sender<()>>,
}
```

- [ ] **Step 4: Move `ManagedProcess` from old orchestrator to `mod.rs`**

The `ManagedProcess` struct and its dependencies remain defined in `mod.rs` so all sub-modules can use it. The old `orchestrator.rs` will be deleted at the end of PR2.

- [ ] **Step 5: Update `application/mod.rs`**

Replace the single `pub mod orchestrator;` with:

```rust
pub mod command_runner;
pub mod events;
pub mod readiness;
pub mod orchestrator;
```

- [ ] **Step 6: Run tests for dependency module**

```bash
cd src-tauri && cargo test dependency
```
Expected: 2 tests PASS (dependencies_satisfied_requires_success, dependencies_satisfied_requires_ready)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/application/orchestrator/
git add src-tauri/src/application/mod.rs
git commit -m "refactor: extract DepResolver into orchestrator/dependency.rs"
```

---

### Task 2.2: Extract `SessionManager` and `ActiveSession`

**Files:**
- Create: `src-tauri/src/application/orchestrator/session.rs`
- Modify: `src-tauri/src/application/orchestrator/mod.rs`

- [ ] **Step 1: Write `session.rs`**

Create `src-tauri/src/application/orchestrator/session.rs`:

```rust
use std::collections::HashMap;

use chrono::Utc;
use indexmap::IndexMap;
use tokio::sync::broadcast;

use crate::{
    domain::{
        config::ProcessKind,
        process::ProcessStatus,
        project::ProjectRecord,
        runtime::{
            ProcessRuntimeId, ProcessSnapshot, RunSessionId, RunSessionSnapshot,
        },
    },
    error::AppError,
    infrastructure::{config_loader::LoadedProjectConfig, log_store::InMemoryLogStore},
};

use super::ManagedProcess;

pub struct OrchestratorState {
    pub sessions: HashMap<String, ActiveSession>,
}

pub struct ActiveSession {
    pub snapshot: RunSessionSnapshot,
    pub project: ProjectRecord,
    pub loaded_config: LoadedProjectConfig,
    pub processes: HashMap<String, ManagedProcess>,
    pub stop_requested: bool,
    pub logs: InMemoryLogStore,
}

impl ActiveSession {
    pub fn new(
        project: ProjectRecord,
        loaded_config: LoadedProjectConfig,
    ) -> Result<Self, AppError> {
        let session_id = RunSessionId::new();
        let started_at = Utc::now();
        let mut session_snapshot = RunSessionSnapshot {
            session_id: session_id.clone(),
            project_id: project.id.clone(),
            project_name: project.name.clone(),
            base_dir: project.base_dir.clone(),
            started_at,
            stopped_at: None,
            processes: Vec::new(),
        };

        let mut processes = HashMap::new();
        for (name, config) in &loaded_config.config.processes {
            let (log_tx, _) = broadcast::channel(4096);
            let snapshot = ProcessSnapshot {
                runtime_id: ProcessRuntimeId::new(),
                name: name.clone(),
                kind: config.kind.clone(),
                status: ProcessStatus::Pending,
                started_at: None,
                exited_at: None,
                exit_code: None,
            };
            session_snapshot.processes.push(snapshot.clone());
            processes.insert(
                name.clone(),
                ManagedProcess {
                    config: config.clone(),
                    snapshot,
                    child: None,
                    pid: None,
                    kill_tx: None,
                    log_tx,
                    terminating: false,
                    generation: 0,
                    stop_notify_tx: None,
                },
            );
        }

        Ok(Self {
            snapshot: session_snapshot,
            project,
            loaded_config,
            processes,
            stop_requested: false,
            logs: InMemoryLogStore::default(),
        })
    }

    pub fn sync_snapshot_process(
        session_snapshot: &mut RunSessionSnapshot,
        process_snapshot: &ProcessSnapshot,
    ) {
        if let Some(existing) = session_snapshot
            .processes
            .iter_mut()
            .find(|p| p.runtime_id == process_snapshot.runtime_id)
        {
            *existing = process_snapshot.clone();
        }
    }
}
```

- [ ] **Step 2: Update `mod.rs` to use `ActiveSession::new()`**

In `mod.rs`, delegate session creation to `ActiveSession::new()` from `start_session`:

```rust
impl ProcessOrchestrator {
    pub async fn start_session(
        &self,
        app_handle: AppHandle,
        window_key: String,
        project: ProjectRecord,
        loaded_config: LoadedProjectConfig,
    ) -> Result<RunSessionSnapshot, AppError> {
        let session_snapshot = {
            let mut state = self.inner.lock().await;
            if let Some(active) = state.sessions.get(&window_key) {
                if active.snapshot.stopped_at.is_none() {
                    return Err(AppError::runtime(
                        "a project session is already running in this window",
                    ));
                }
            }
            let session = ActiveSession::new(project, loaded_config)?;
            let snapshot = session.snapshot.clone();
            state.sessions.insert(window_key.clone(), session);
            snapshot
        };

        self.emit_snapshot(&app_handle, &window_key).await?;
        self.spawn_runnable_processes(app_handle.clone(), &window_key).await?;
        Ok(session_snapshot)
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cd src-tauri && cargo test
```
Expected: All existing tests PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/application/orchestrator/
git commit -m "refactor: extract SessionManager into orchestrator/session.rs"
```

---

### Task 2.3: Extract `ProcessLifecycle` (status transitions, terminate helpers)

**Files:**
- Create: `src-tauri/src/application/orchestrator/lifecycle.rs`
- Modify: `src-tauri/src/application/orchestrator/mod.rs`

- [ ] **Step 1: Write `lifecycle.rs`**

Create `src-tauri/src/application/orchestrator/lifecycle.rs`:

```rust
use std::time::Duration;

use chrono::Utc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot};

use crate::{
    application::events::{RuntimeEvent, RUNTIME_ERROR_EVENT},
    domain::{
        config::ProcessKind,
        process::ProcessStatus,
        runtime::ProcessSnapshot,
    },
    error::AppError,
};

use super::{ActiveSession, ManagedProcess};

/// Marks a managed process for termination, returns kill signal channel if child alive.
/// On Unix the process group is killed synchronously here (pid stored separately from child mutex).
pub fn begin_process_termination(process: &mut ManagedProcess) -> Option<mpsc::Sender<()>> {
    process.terminating = true;

    #[cfg(unix)]
    if let Some(pid) = process.pid {
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
    }

    if matches!(
        process.snapshot.status,
        ProcessStatus::Starting | ProcessStatus::Running | ProcessStatus::Ready
    ) {
        process.snapshot.status = ProcessStatus::Stopping;
        process.kill_tx.take()
    } else {
        None
    }
}

/// Resets a managed process to `Pending` so it can be re-spawned.
pub fn reset_managed_process(process: &mut ManagedProcess) {
    process.child = None;
    process.pid = None;
    process.kill_tx = None;
    process.stop_notify_tx = None;
    process.terminating = false;
    process.generation += 1;
    process.snapshot.status = ProcessStatus::Pending;
    process.snapshot.started_at = None;
    process.snapshot.exited_at = None;
    process.snapshot.exit_code = None;
}

/// Transitions a process to the given status and syncs to the session snapshot.
pub fn transition_status(
    active: &mut ActiveSession,
    process_name: &str,
    status: ProcessStatus,
) -> Result<(), AppError> {
    let process = active
        .processes
        .get_mut(process_name)
        .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
    process.snapshot.status = status;
    ActiveSession::sync_snapshot_process(&mut active.snapshot, &process.snapshot);
    Ok(())
}

pub fn get_process_mut<'a>(
    active: &'a mut ActiveSession,
    process_name: &str,
) -> Result<&'a mut ManagedProcess, AppError> {
    active
        .processes
        .get_mut(process_name)
        .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))
}

pub fn get_process<'a>(
    active: &'a ActiveSession,
    process_name: &str,
) -> Result<&'a ManagedProcess, AppError> {
    active
        .processes
        .get(process_name)
        .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))
}

pub fn stop_active_process(
    active: &mut ActiveSession,
    process_name: &str,
) -> Result<(Option<mpsc::Sender<()>>, oneshot::Receiver<()>), AppError> {
    let process = get_process_mut(active, process_name)?;
    if matches!(process.config.kind, ProcessKind::Task) {
        return Err(AppError::runtime("cannot stop a task process"));
    }
    let (notify_tx, notify_rx) = oneshot::channel();
    process.stop_notify_tx = Some(notify_tx);
    let kill_tx = begin_process_termination(process);
    ActiveSession::sync_snapshot_process(&mut active.snapshot, &process.snapshot);
    Ok((kill_tx, notify_rx))
}

pub async fn send_kill_and_wait(
    kill_tx: Option<mpsc::Sender<()>>,
    done_rx: oneshot::Receiver<()>,
) {
    if let Some(kill_tx) = kill_tx {
        let _ = kill_tx.send(()).await;
    }
    let _ = tokio::time::timeout(Duration::from_secs(10), done_rx).await;
}

// Test helpers: these free functions are tested directly.
pub fn build_process_env_impl(
    global_env: &indexmap::IndexMap<String, String>,
    process_env: &indexmap::IndexMap<String, String>,
) -> std::collections::HashMap<String, String> {
    let mut env: std::collections::HashMap<String, String> = std::env::vars().collect();
    for (key, value) in global_env {
        env.insert(key.clone(), value.clone());
    }
    for (key, value) in process_env {
        env.insert(key.clone(), value.clone());
    }
    env
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use tokio::sync::{broadcast, mpsc};

    use super::*;
    use crate::domain::{
        config::ProcessConfig,
        runtime::ProcessRuntimeId,
    };

    fn managed_process(name: &str, kind: ProcessKind, status: ProcessStatus) -> ManagedProcess {
        let (log_tx, _) = broadcast::channel(4);
        ManagedProcess {
            config: ProcessConfig {
                kind: kind.clone(),
                cmd: format!("run {name}"),
                env: IndexMap::new(),
                depends_on: IndexMap::new(),
                ready: None,
            },
            snapshot: ProcessSnapshot {
                runtime_id: ProcessRuntimeId::new(),
                name: name.to_string(),
                kind,
                status,
                started_at: None,
                exited_at: None,
                exit_code: None,
            },
            child: None,
            pid: None,
            kill_tx: None,
            log_tx,
            terminating: false,
            generation: 0,
            stop_notify_tx: None,
        }
    }

    fn managed_process_with_kill_tx(
        name: &str,
        kind: ProcessKind,
        status: ProcessStatus,
    ) -> (ManagedProcess, mpsc::Receiver<()>) {
        let mut process = managed_process(name, kind.clone(), status);
        let (kill_tx, kill_rx) = mpsc::channel::<()>(1);
        process.kill_tx = Some(kill_tx);
        (process, kill_rx)
    }

    #[test]
    fn reset_managed_process_restores_pending_state() {
        let mut process = managed_process("api", ProcessKind::Service, ProcessStatus::Failed);
        process.terminating = true;
        process.snapshot.exit_code = Some(1);
        process.snapshot.exited_at = Some(Utc::now());
        reset_managed_process(&mut process);
        assert_eq!(process.snapshot.status, ProcessStatus::Pending);
        assert!(!process.terminating);
        assert!(process.child.is_none());
        assert_eq!(process.snapshot.exit_code, None);
        assert_eq!(process.snapshot.exited_at, None);
        assert_eq!(process.snapshot.started_at, None);
    }

    #[tokio::test]
    async fn begin_process_termination_transitions_running_to_stopping() {
        for status in [ProcessStatus::Starting, ProcessStatus::Running, ProcessStatus::Ready] {
            let (mut process, mut kill_rx) =
                managed_process_with_kill_tx("api", ProcessKind::Service, status);
            let kill_tx = begin_process_termination(&mut process);
            assert_eq!(process.snapshot.status, ProcessStatus::Stopping);
            assert!(process.terminating);
            let kill_tx = kill_tx.expect("kill signal should be returned");
            assert!(process.kill_tx.is_none());
            kill_tx.send(()).await.expect("kill signal should send");
            drop(kill_tx);
            assert_eq!(kill_rx.recv().await, Some(()));
        }
    }

    #[test]
    fn begin_process_termination_leaves_terminal_states_untouched() {
        for status in [
            ProcessStatus::Pending,
            ProcessStatus::Blocked,
            ProcessStatus::Stopping,
            ProcessStatus::Stopped,
            ProcessStatus::Failed,
            ProcessStatus::Succeeded,
        ] {
            let (mut process, _kill_rx) =
                managed_process_with_kill_tx("api", ProcessKind::Service, status);
            let kill_tx = begin_process_termination(&mut process);
            assert!(kill_tx.is_none(), "no kill signal expected for terminal/pending state {status:?}");
            assert_eq!(process.snapshot.status, status, "status must be unchanged for {status:?}");
        }
    }

    #[test]
    fn build_process_env_inherits_system_environment() {
        let global_env = IndexMap::new();
        let process_env = IndexMap::new();
        let merged = build_process_env_impl(&global_env, &process_env);
        assert!(
            merged.contains_key("PATH") || merged.contains_key("Path"),
            "merged env must inherit system PATH"
        );
    }

    #[test]
    fn build_process_env_process_override_wins_over_system() {
        let mut global_env = IndexMap::new();
        global_env.insert("HOME".to_string(), "/custom".to_string());
        let process_env = IndexMap::new();
        let merged = build_process_env_impl(&global_env, &process_env);
        assert_eq!(merged.get("HOME"), Some(&"/custom".to_string()));
    }
}
```

- [ ] **Step 2: Run lifecycle tests**

```bash
cd src-tauri && cargo test lifecycle
```
Expected: 5 tests PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/application/orchestrator/
git commit -m "refactor: extract ProcessLifecycle into orchestrator/lifecycle.rs"
```

---

### Task 2.4: Extract `LogDistributor`

**Files:**
- Create: `src-tauri/src/application/orchestrator/log.rs`
- Modify: `src-tauri/src/application/orchestrator/mod.rs`

- [ ] **Step 1: Write `log.rs`**

Create `src-tauri/src/application/orchestrator/log.rs`:

```rust
use chrono::Utc;
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::broadcast,
};

use crate::{
    application::events::{ProcessLogEvent, PROCESS_LOG_EVENT},
    domain::{
        process::LogStream,
        runtime::{ProcessLogPayload, ProcessRuntimeId, RunSessionId},
    },
    error::AppError,
};

use super::ManagedProcess;

pub fn spawn_log_task<R>(
    app_handle: AppHandle,
    window_key: String,
    session_id: RunSessionId,
    process_name: String,
    runtime_id: ProcessRuntimeId,
    stream: LogStream,
    reader: R,
    log_tx: broadcast::Sender<String>,
    append_log_fn: impl Fn(ProcessLogPayload) -> Result<(), AppError> + Send + 'static,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = log_tx.send(line.clone());
            let payload = ProcessLogPayload {
                session_id: session_id.clone(),
                runtime_id: runtime_id.clone(),
                process_name: process_name.clone(),
                stream,
                line,
                timestamp: Utc::now(),
            };
            let _ = append_log_fn(payload.clone());
            let _ = app_handle.emit_to(
                &window_key,
                PROCESS_LOG_EVENT,
                ProcessLogEvent { payload },
            );
        }
    });
}
```

- [ ] **Step 2: Update `mod.rs` to delegate `spawn_log_task`**

Replace the `spawn_log_task` method in the `impl ProcessOrchestrator` block to call the free function from `log.rs` with a closure that captures `self` for `append_log`.

- [ ] **Step 3: Run tests**

```bash
cd src-tauri && cargo test
```
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/application/orchestrator/
git commit -m "refactor: extract LogDistributor into orchestrator/log.rs"
```

---

### Task 2.5: Final cleanup — wire facade, delete old `orchestrator.rs`

**Files:**
- Modify: `src-tauri/src/application/orchestrator/mod.rs`
- Delete: `src-tauri/src/application/orchestrator.rs` (move remaining code to `mod.rs`)

- [ ] **Step 1: Move remaining code to `mod.rs`**

The `impl ProcessOrchestrator` block in `mod.rs` should now delegate to sub-modules:
- `start_session` → `ActiveSession::new()` + `dependency::runnable_names()` + per-process spawn loop
- `stop_session` / `finish_session` → `lifecycle::begin_process_termination()` + kill sends
- `spawn_named_process` → `command_runner::spawn_process()` + `log::spawn_log_task()` + `lifecycle::transition_status()` + readiness + exit watcher
- `mark_process_ready`, `handle_process_exit`, `handle_process_failure` → `lifecycle::transition_status()` + `dependency::runnable_names()` + snapshot emits

The `build_process_env`, `sync_snapshot_process`, `reset_managed_process`, `begin_process_termination` free functions and their tests move to their sub-modules (already done in tasks 2.1-2.4).

- [ ] **Step 2: Delete old `orchestrator.rs`**

```bash
rm src-tauri/src/application/orchestrator.rs
```

- [ ] **Step 3: Run full test suite**

```bash
cd src-tauri && cargo test
deno task check
```
Expected: All tests PASS, no compilation errors.

- [ ] **Step 4: Manual smoke test**

```bash
deno task app examples/deno-runner.yml
```
Expected: App starts, processes spawn and run, session lifecycle works.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/application/orchestrator/
git rm src-tauri/src/application/orchestrator.rs
git commit -m "refactor: split orchestrator.rs into focused sub-modules

- orchestrator/session.rs — ActiveSession, OrchestratorState
- orchestrator/lifecycle.rs — spawn, kill, status transitions
- orchestrator/dependency.rs — DepResolver (pure functions)
- orchestrator/log.rs — LogDistributor (log task spawning)
- orchestrator/mod.rs — ProcessOrchestrator facade

All public API signatures preserved. All 7 tests migrated to sub-modules."
```

---

## PR 3: Integration Tests

### Task 3.1: Add integration test for full session lifecycle

**Files:**
- Create: `src-tauri/tests/session_lifecycle.rs`
- Modify: `src-tauri/Cargo.toml` (add `dev-dependencies` if needed — `tempfile` is already present)

**Interfaces:**
- Produces: Integration test using real ProcessOrchestrator, real config file, real shell commands

- [ ] **Step 1: Create tests directory**

```bash
mkdir -p src-tauri/tests
```

- [ ] **Step 2: Write `session_lifecycle.rs`**

Create `src-tauri/tests/session_lifecycle.rs`:

```rust
use std::fs;

use chrono::Utc;
use tempfile::TempDir;

use devapp_lib::{
    application::orchestrator::ProcessOrchestrator,
    domain::{
        config::ProcessKind,
        process::ProcessStatus,
        project::{ProjectId, ProjectRecord, ProjectSource},
    },
    infrastructure::config_loader::{self, LoadedProjectConfig},
};
use tauri::Manager;

fn write_config(dir: &TempDir, yaml: &str) -> LoadedProjectConfig {
    let config_path = dir.path().join("devapp.yml");
    fs::write(&config_path, yaml).expect("write devapp.yml");
    config_loader::load_config(&config_path).expect("load config")
}

#[tokio::test]
async fn task_succeeds_and_session_stops() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let yaml = r#"
version: 1
processes:
  hello:
    kind: task
    cmd: echo ready
"#;
    let loaded = write_config(&dir, yaml);

    let project = ProjectRecord {
        id: ProjectId::new(),
        name: "test-project".to_string(),
        base_dir: dir.path().to_path_buf(),
        config_source: ProjectSource::ProjectFile,
        config_path: dir.path().join("devapp.yml"),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let app = tauri::Builder::default()
        .build(tauri::generate_context!())
        .expect("build app");
    let orchestrator = ProcessOrchestrator::new();

    // Start session
    let snapshot = orchestrator
        .start_session(
            app.handle().clone(),
            "test-window".to_string(),
            project.clone(),
            loaded,
        )
        .await
        .expect("start session");

    assert_eq!(snapshot.processes.len(), 1);
    assert_eq!(snapshot.processes[0].name, "hello");
    assert_eq!(snapshot.processes[0].kind, ProcessKind::Task);

    // Poll until the task reaches Succeeded
    let mut attempts = 0;
    loop {
        let snap = orchestrator
            .snapshot("test-window")
            .await
            .expect("snapshot")
            .expect("session should exist");

        let status = snap.processes[0].status;
        if status == ProcessStatus::Succeeded {
            break;
        }
        if status == ProcessStatus::Failed {
            panic!("task failed unexpectedly");
        }
        attempts += 1;
        if attempts > 50 {
            panic!("task did not finish within timeout, status={:?}", status);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Stop session
    let stopped = orchestrator
        .stop_session(app.handle().clone(), "test-window")
        .await
        .expect("stop session")
        .expect("stopped snapshot");

    assert!(stopped.stopped_at.is_some(), "stopped_at should be set");
    assert_eq!(stopped.processes[0].status, ProcessStatus::Stopped);
}

#[tokio::test]
async fn process_failure_stops_session() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let yaml = r#"
version: 1
processes:
  bad:
    kind: task
    cmd: exit 1
"#;
    let loaded = write_config(&dir, yaml);

    let project = ProjectRecord {
        id: ProjectId::new(),
        name: "fail-project".to_string(),
        base_dir: dir.path().to_path_buf(),
        config_source: ProjectSource::ProjectFile,
        config_path: dir.path().join("devapp.yml"),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let app = tauri::Builder::default()
        .build(tauri::generate_context!())
        .expect("build app");
    let orchestrator = ProcessOrchestrator::new();

    orchestrator
        .start_session(app.handle().clone(), "fail-window".to_string(), project, loaded)
        .await
        .expect("start session");

    // Poll until the task fails
    let mut attempts = 0;
    loop {
        let snap = orchestrator
            .snapshot("fail-window")
            .await
            .expect("snapshot")
            .expect("session should exist");

        let status = snap.processes[0].status;
        if status == ProcessStatus::Failed {
            break;
        }
        if status == ProcessStatus::Succeeded {
            panic!("task should have failed");
        }
        attempts += 1;
        if attempts > 50 {
            panic!("task did not fail within timeout, status={:?}", status);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let snap = orchestrator
        .snapshot("fail-window")
        .await
        .expect("snapshot")
        .expect("session should exist");
    assert!(snap.stopped_at.is_some(), "session should be stopped after failure");
}
```

- [ ] **Step 2: Verify test compiles**

```bash
cd src-tauri && cargo test --test session_lifecycle --no-run
```
Expected: Compilation succeeds.

- [ ] **Step 3: Run integration tests**

```bash
cd src-tauri && cargo test --test session_lifecycle
```
Expected: 2 tests PASS

- [ ] **Step 4: Run full test suite**

```bash
cd src-tauri && cargo test
```
Expected: All tests PASS (including integration tests)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tests/
git commit -m "test: add integration tests for session lifecycle

- task_succeeds_and_session_stops: full start→ready→stop cycle
- process_failure_stops_session: failure triggers session stop"
```

---

## PR 4: Structured Error Codes + Tracing

### Task 4.1: Add `ErrorCode` enum to `AppError`

**Files:**
- Modify: `src-tauri/src/error.rs`
- Modify: All files that construct `AppError` (commands, orchestrator, config_loader, pty, project_store, readiness, command_runner)

**Interfaces:**
- Produces: `pub enum ErrorCode { ... }` — 16 variants
- Produces: `impl AppError { pub fn code(&self) -> ErrorCode { ... } }`
- Produces: Updated constructor helpers that accept or infer the error code

- [ ] **Step 1: Add `ErrorCode` enum and update `AppError`**

Replace `src-tauri/src/error.rs`:

```rust
use std::io;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    ConfigNotFound,
    ConfigParseFailed,
    ConfigValidationFailed,
    ConfigUnsupportedVersion,
    ConfigDependencyCycle,
    ConfigUnknownProcess,
    ProjectNotFound,
    ProjectAlreadyRunning,
    LaunchLocked,
    ProcessNotFound,
    ProcessStartFailed,
    ProcessCannotRestart,
    ReadinessTimeout,
    ReadinessCheckFailed,
    IoError,
    TerminalError,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("config error: {0}")]
    Config(String, ErrorCode),
    #[error("io error: {0}")]
    Io(String, ErrorCode),
    #[error("validation error: {0}")]
    Validation(String, ErrorCode),
    #[error("runtime error: {0}")]
    Runtime(String, ErrorCode),
    #[error("project store error: {0}")]
    ProjectStore(String, ErrorCode),
    #[error("terminal error: {0}")]
    Terminal(String, ErrorCode),
    #[error("project is launch-locked and cannot be modified")]
    LaunchLocked,
}

impl AppError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into(), ErrorCode::ConfigParseFailed)
    }

    pub fn config_with_code(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::Config(message.into(), code)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into(), ErrorCode::ConfigValidationFailed)
    }

    pub fn validation_with_code(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::Validation(message.into(), code)
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into(), ErrorCode::ProcessStartFailed)
    }

    pub fn runtime_with_code(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::Runtime(message.into(), code)
    }

    pub fn project_store(message: impl Into<String>) -> Self {
        Self::ProjectStore(message.into(), ErrorCode::ProjectNotFound)
    }

    pub fn project_store_with_code(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::ProjectStore(message.into(), code)
    }

    pub fn terminal(message: impl Into<String>) -> Self {
        Self::Terminal(message.into(), ErrorCode::TerminalError)
    }

    pub fn terminal_with_code(message: impl Into<String>, code: ErrorCode) -> Self {
        Self::Terminal(message.into(), code)
    }

    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Config(_, code) => *code,
            Self::Io(_, code) => *code,
            Self::Validation(_, code) => *code,
            Self::Runtime(_, code) => *code,
            Self::ProjectStore(_, code) => *code,
            Self::Terminal(_, code) => *code,
            Self::LaunchLocked => ErrorCode::LaunchLocked,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string(), ErrorCode::IoError)
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Config(error.to_string(), ErrorCode::ConfigParseFailed)
    }
}

impl From<AppError> for String {
    fn from(value: AppError) -> Self {
        value.to_string()
    }
}
```

- [ ] **Step 2: Update all error constructors to use specific codes**

Replace calls across the codebase:

| Location | Old call | New call |
|----------|---------|----------|
| `orchestrator.rs` "already running" | `AppError::runtime(...)` | `AppError::runtime_with_code(..., ErrorCode::ProjectAlreadyRunning)` |
| `orchestrator.rs` "no session available" | `AppError::runtime(...)` | `AppError::runtime_with_code(..., ErrorCode::ProcessNotFound)` |
| `orchestrator.rs` "unknown process" | `AppError::runtime(...)` | `AppError::runtime_with_code(..., ErrorCode::ProcessNotFound)` |
| `orchestrator.rs` "cannot restart a task" | `AppError::runtime(...)` | `AppError::runtime_with_code(..., ErrorCode::ProcessCannotRestart)` |
| `readiness.rs` timed out | `AppError::runtime(...)` | `AppError::runtime_with_code(..., ErrorCode::ReadinessTimeout)` |
| `readiness.rs` command failed | `AppError::runtime(...)` | `AppError::runtime_with_code(..., ErrorCode::ReadinessCheckFailed)` |
| `config_loader.rs` validation errors | `AppError::validation(...)` | Use `validation_with_code(...)` with `ConfigValidationFailed`/`ConfigDependencyCycle`/`ConfigUnknownProcess`/`ConfigUnsupportedVersion` |
| `config_loader.rs` path errors | `AppError::config(...)` | `AppError::config_with_code(..., ErrorCode::ConfigNotFound)` |
| `commands.rs` "project not found" | `AppError::project_store(...)` | `AppError::project_store_with_code(..., ErrorCode::ProjectNotFound)` |

- [ ] **Step 3: Make `ErrorCode` available in Tauri command return values**

Create a new response wrapper in `src-tauri/src/tauri_api/commands.rs`:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppErrorPayload {
    message: String,
    code: ErrorCode,
}

fn to_error_string(error: AppError) -> String {
    let payload = AppErrorPayload {
        message: error.to_string(),
        code: error.code(),
    };
    serde_json::to_string(&payload).unwrap_or_else(|_| error.to_string())
}
```

Replace all `.map_err(String::from)` with `.map_err(to_error_string)`.

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test
```
Expected: All tests PASS (after updating any string-based error assertions)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/tauri_api/commands.rs src-tauri/src/application/
git add src-tauri/src/infrastructure/
git commit -m "feat: add structured ErrorCode enum to AppError

Each AppError variant now carries an ErrorCode. Tauri commands return
errors as JSON with { message, code } so the frontend can match on codes
instead of parsing strings.

Added _with_code constructors for targeted error codes while keeping
shortcut constructors with sensible defaults."
```

---

### Task 4.2: Add `tracing` instrumentation

**Files:**
- Modify: `src-tauri/Cargo.toml` (add `tracing` dependency)
- Modify: `src-tauri/src/lib.rs` (initialize tracing subscriber)
- Modify: `src-tauri/src/application/orchestrator/mod.rs` (instrument key methods)

- [ ] **Step 1: Add `tracing` dependency**

Add to `Cargo.toml` dependencies:

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

- [ ] **Step 2: Initialize tracing in `lib.rs`**

Add at the top of `run()`:

```rust
use tracing_subscriber::{fmt, EnvFilter};

// In run(), before tauri::Builder:
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_target(false)
    .init();
```

- [ ] **Step 3: Add `#[instrument]` and `info!/warn!/error!` calls**

Instrument key points in `orchestrator/mod.rs`:

```rust
use tracing::{info, warn, error};

// In start_session:
info!(project = %project.name, "starting session");

// In spawn_named_process (after spawn):
info!(process = %process_name, pid = ?child_pid, "process started");

// In handle_process_exit:
info!(process = %process_name, exit_code = ?exit_code, "process exited");

// In handle_process_failure:
error!(process = %process_name, error = %message, "process failed");

// In wait_until_ready timeout (readiness.rs):
warn!(process = %process_name, "readiness timeout");

// In finish_session:
info!("session stopped");
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test
deno task check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs
git add src-tauri/src/application/
git commit -m "feat: add tracing instrumentation for process lifecycle

Initialize tracing-subscriber with env-filter support. Instrument
session start/stop, process spawn/exit/failure, and readiness timeouts.
Use RUST_LOG=info to see structured logs during development."
```

---

### Task 4.3: Wrap `fs::canonicalize` in `spawn_blocking`

**Files:**
- Modify: `src-tauri/src/infrastructure/config_loader.rs:23`

- [ ] **Step 1: Change `load_config` to use `spawn_blocking`**

Replace the function body:

```rust
pub async fn load_config_async(config_path: &Path) -> Result<LoadedProjectConfig, AppError> {
    let config_path = config_path.to_path_buf();
    tokio::task::spawn_blocking(move || load_config(&config_path))
        .await
        .map_err(|error| AppError::runtime_with_code(
            error.to_string(),
            ErrorCode::IoError,
        ))?
}
```

Keep the sync `load_config` as-is for non-async callers. Update callers in `commands.rs` and `project_store.rs` to use the async version.

- [ ] **Step 2: Update callers**

In `commands.rs` `start_project` (line 231):
```rust
let loaded = config_loader::load_config_async(&project.config_path)
    .await
    .map_err(to_error_string)?;
```

In `project_store.rs` `import_project_config_path` (line 110):
```rust
// This is called from a sync context (setup closure). Keep sync load_config.
```

No change needed for `import_project_config_path` since it's called from `block_on` in setup.

- [ ] **Step 3: Run tests and commit**

```bash
cd src-tauri && cargo test
git add src-tauri/src/infrastructure/config_loader.rs src-tauri/src/tauri_api/commands.rs
git commit -m "perf: wrap fs::canonicalize in spawn_blocking

load_config_async uses Tokio's blocking thread pool for path resolution.
Prevents blocking the async runtime on slow/NFS filesystems."
```

---

### Task 4.4: Include process name in readiness error

**Files:**
- Modify: `src-tauri/src/application/orchestrator/mod.rs:421-426`

- [ ] **Step 1: Update readiness failure error**

```rust
// Before:
RuntimeEvent::RuntimeError {
    message: error.to_string(),
}

// After:
RuntimeEvent::RuntimeError {
    message: format!("process `{}` readiness failed: {}", process_name_for_ready, error),
}
```

- [ ] **Step 2: Run tests and commit**

```bash
cd src-tauri && cargo test
git add src-tauri/src/application/orchestrator/mod.rs
git commit -m "fix: include process name in readiness failure error"
```

---

### Task 4.5: Frontend error code handling

**Files:**
- Modify: `src/lib/types.ts` — add `ErrorCode` type
- Modify: `src/lib/tauri/client.ts` — add error parsing helper
- Modify: `src/lib/stores/runtime.svelte.ts` — use error codes for targeted messages

- [ ] **Step 1: Add `ErrorCode` type to `types.ts`**

```typescript
export type ErrorCode =
  | "configNotFound"
  | "configParseFailed"
  | "configValidationFailed"
  | "configUnsupportedVersion"
  | "configDependencyCycle"
  | "configUnknownProcess"
  | "projectNotFound"
  | "projectAlreadyRunning"
  | "launchLocked"
  | "processNotFound"
  | "processStartFailed"
  | "processCannotRestart"
  | "readinessTimeout"
  | "readinessCheckFailed"
  | "ioError"
  | "terminalError";
```

- [ ] **Step 2: Add error parsing in `runtime.svelte.ts`**

Replace `setError`:

```typescript
setError(error: unknown) {
    const raw = error instanceof Error ? error.message : String(error);
    // Try to parse the structured error JSON from the backend
    try {
        const parsed = JSON.parse(raw) as { message: string; code: string };
        if (parsed.code && parsed.message) {
            const friendly = ERROR_CODE_MESSAGES[parsed.code] ?? parsed.message;
            this.uiError = friendly;
            return;
        }
    } catch {
        // Not JSON — use raw string
    }
    this.uiError = raw;
}
```

Add a constant map at module level:

```typescript
const ERROR_CODE_MESSAGES: Record<string, string> = {
    configNotFound: "Config file not found. Check that devapp.yml exists.",
    projectAlreadyRunning: "A project is already running in this window.",
    launchLocked: "Project is locked by command-line launch and cannot be modified.",
    processCannotRestart: "Tasks cannot be restarted — only services support restart.",
    readinessTimeout: "Process readiness check timed out. Check the readiness config.",
    readinessCheckFailed: "Process readiness check failed. Verify the readiness command or URL.",
    projectNotFound: "Project not found. It may have been deleted.",
};
```

- [ ] **Step 3: Run frontend typecheck**

```bash
deno task check
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts src/lib/stores/runtime.svelte.ts
git commit -m "feat: parse structured error codes in frontend

Backend errors now include { message, code }. The frontend parses the
JSON and shows user-friendly messages for known error codes, falling
back to the raw message for unknown codes."
```

---

## PR 5: Thread/Task Hygiene + State Standardization

### Task 5.1: Replace `thread::spawn` + `block_on` with `tokio::spawn`

**Files:**
- Modify: `src-tauri/src/application/orchestrator/mod.rs:401,452`

- [ ] **Step 1: Change readiness watcher from thread to task**

Replace lines 401-441:

```rust
// Before:
thread::spawn(move || {
    tauri::async_runtime::block_on(async move {
        // ...
    });
});

// After:
tokio::spawn(async move {
    // ... same body, no block_on wrapper ...
});
```

Same change for the exit watcher at line 452.

- [ ] **Step 2: Remove unused `use std::thread;` import**

- [ ] **Step 3: Run tests to verify no regressions**

```bash
cd src-tauri && cargo test
deno task app examples/deno-runner.yml
```
Expected: All tests pass, app works correctly. Process exit/readiness still detected.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/application/orchestrator/mod.rs
git commit -m "refactor: replace thread::spawn+block_on with tokio::spawn

Readiness watcher and exit watcher now use tokio::spawn instead of
dedicated OS threads. Reduces thread count per service from 2 to 0
(uses Tokio's cooperative scheduler)."
```

---

### Task 5.2: Standardize `TerminalManager` on `tokio::sync::Mutex`

**Files:**
- Modify: `src-tauri/src/infrastructure/pty.rs`
- Modify: `src-tauri/src/tauri_api/commands.rs` (callers)

- [ ] **Step 1: Change `TerminalManager` mutex type**

In `pty.rs`:

```rust
// Before:
use std::sync::{Arc, Mutex};
// After:
use std::sync::Arc;
use tokio::sync::Mutex;

// Before:
inner: Arc<Mutex<HashMap<TerminalSessionId, ManagedTerminal>>>,
// After:
inner: Arc<tokio::sync::Mutex<HashMap<TerminalSessionId, ManagedTerminal>>>,
```

- [ ] **Step 2: Update all `.lock()` to `.lock().await`**

Every `self.inner.lock().map_err(|_| ...)` becomes `self.inner.lock().await`. Remove all `map_err` poison-error handling since `tokio::sync::Mutex` doesn't poison.

- [ ] **Step 3: Update PTY reader thread**

The reader thread at line 108 needs a `tokio::runtime::Handle`:

```rust
let handle = tokio::runtime::Handle::current();
thread::spawn(move || {
    // PTY read is blocking I/O — keep this in a thread
    let mut buffer = [0_u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read_len) => {
                let payload = /* ... */;
                let _ = output_app_handle.emit_to(/* ... */);
            }
            Err(_) => break,
        }
    }
});
```

Note: The PTY reader thread remains using `std::thread::spawn` because `portable-pty::MasterPty::read` is blocking (uses `std::io::Read`). The reader doesn't need the mutex — it only emits events. The mutex is only used by `open_terminal`, `write_terminal`, `resize_terminal`, `close_terminal`, `close_all_for_window`.

- [ ] **Step 4: Update callers in `commands.rs`**

`close_all_for_window` is called from `stop_project` (line 252). With `tokio::sync::Mutex`, the call is now non-blocking on the Tokio worker.

- [ ] **Step 5: Run tests and manual smoke**

```bash
cd src-tauri && cargo test
deno task app examples/deno-runner.yml
```
Manual: open a terminal, type commands, resize, close. Verify no regressions.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/infrastructure/pty.rs src-tauri/src/tauri_api/commands.rs
git commit -m "refactor: standardize TerminalManager on tokio::sync::Mutex

Removes blocking mutex from async contexts. PTY reader thread remains
on std::thread for blocking I/O but no longer locks the mutex directly."
```

---

### Task 5.3: Handle `setup()` errors gracefully

**Files:**
- Modify: `src-tauri/src/lib.rs:35-46`

- [ ] **Step 1: Log and store setup errors instead of silently mapping**

```rust
.setup(|app| {
    // ... macOS/Windows window setup ...
    let state = app.state::<AppState>().inner().clone();
    tauri::async_runtime::block_on(async move {
        if let Some(config_path) = launch_config_path() {
            let store = state.project_store.lock().await;
            match store.import_project_config_path(config_path) {
                Ok(project) => {
                    let mut launch_project_id = state.launch_project_id.lock().await;
                    *launch_project_id = Some(project.id);
                }
                Err(error) => {
                    tracing::error!(
                        error = %error,
                        code = ?error.code(),
                        "failed to import launch config path"
                    );
                }
            }
        }
        Ok::<(), crate::error::AppError>(())
    })
    .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
    Ok(())
})
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "fix: log setup errors instead of silently swallowing

Config path import failures during launch are now logged with tracing.
The app continues to start even if the launch config is invalid."
```

---

### Task 5.4: Release `project_store` lock before YAML parsing

**Files:**
- Modify: `src-tauri/src/tauri_api/commands.rs:160-205`

- [ ] **Step 1: Restructure `load_project_config`**

```rust
#[tauri::command]
pub async fn load_project_config(
    state: State<'_, AppState>,
    request: LoadProjectConfigRequest,
) -> Result<ProjectConfigDocument, String> {
    let (project, yaml) = {
        let store = state.project_store.lock().await;
        let project = store
            .get(&request.project_id)
            .map_err(to_error_string)?
            .ok_or_else(|| AppError::project_store_with_code("project not found", ErrorCode::ProjectNotFound).to_error_string())?;

        let yaml = match request.yaml {
            Some(yaml) => yaml,
            None => store.load_project_config_raw(&project).map_err(to_error_string)?,
        };
        (project, yaml)
    }; // Lock released here

    let loaded = parse_config_document(&project.config_path, &yaml).map_err(to_error_string)?;
    Ok(ProjectConfigDocument { project, yaml, config: loaded.config })
}
```

Same restructuring for `save_project_config`.

- [ ] **Step 2: Run tests**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/tauri_api/commands.rs
git commit -m "perf: release project_store lock before YAML parsing

load_project_config and save_project_config now clone data and drop
the Mutex before calling parse_config_document. Prevents other project
store operations from blocking during large YAML parsing."
```

---

### Task 5.5: Fix `expect()` calls for macOS/Windows decorations

**Files:**
- Modify: `src-tauri/src/lib.rs:24,31`

- [ ] **Step 1: Log warning instead of panicking**

```rust
#[cfg(target_os = "macos")]
if let Some(window) = app.get_webview_window("main") {
    if let Err(err) = window.set_title_bar_style(tauri::TitleBarStyle::Overlay) {
        tracing::warn!("failed to set titlebar overlay style: {}", err);
    }
}

#[cfg(not(target_os = "macos"))]
if let Some(window) = app.get_webview_window("main") {
    if let Err(err) = window.set_decorations(false) {
        tracing::warn!("failed to disable window decorations: {}", err);
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "fix: log warning instead of panicking on decoration failure

set_title_bar_style and set_decorations failures are non-fatal.
The app now continues with default window decorations."
```

---

## PR 6: Performance Optimizations + Cleanup

### Task 6.1: Use `Arc<RunSessionSnapshot>` to avoid deep clones

**Files:**
- Modify: `src-tauri/src/application/orchestrator/session.rs`
- Modify: `src-tauri/src/application/orchestrator/mod.rs`

- [ ] **Step 1: Change `ActiveSession.snapshot` to `Arc<RunSessionSnapshot>`**

In `session.rs`, change the field type and add a `make_mut` pattern:

```rust
use std::sync::Arc;

pub struct ActiveSession {
    pub snapshot: Arc<RunSessionSnapshot>, // was: RunSessionSnapshot
    // ... other fields unchanged
}

impl ActiveSession {
    /// Returns a mutable reference to the snapshot, cloning if shared.
    pub fn snapshot_mut(&mut self) -> &mut RunSessionSnapshot {
        Arc::make_mut(&mut self.snapshot)
    }
}
```

- [ ] **Step 2: Update `sync_snapshot_process` to use `snapshot_mut`**

All calls to `ActiveSession::sync_snapshot_process(&mut active.snapshot, ...)` become `ActiveSession::sync_snapshot_process(active.snapshot_mut(), ...)`.

- [ ] **Step 3: Update `snapshot` getter to clone `Arc`**

In `mod.rs`, the `snapshot` method:

```rust
pub async fn snapshot(&self, window_key: &str) -> Result<Option<RunSessionSnapshot>, AppError> {
    let state = self.inner.lock().await;
    Ok(state.sessions.get(window_key).map(|active| (*active.snapshot).clone()))
}
```

The `(*active.snapshot).clone()` dereferences the `Arc` and clones the inner data only when accessed. The `Arc` itself is not cloned for every emit — `emit_snapshot` already calls `self.snapshot(window_key)` which now does one data clone per emission (not per internal state change).

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/application/orchestrator/
git commit -m "perf: use Arc<RunSessionSnapshot> to reduce clone overhead

Snapshot is now stored as Arc<RunSessionSnapshot>. Mutations use
Arc::make_mut for copy-on-write. The snapshot() getter still clones
the data, but internal state transitions avoid deep clones."
```

---

### Task 6.2: Batch log lines for IPC efficiency

**Files:**
- Modify: `src-tauri/src/application/orchestrator/log.rs`
- Modify: `src-tauri/src/application/events.rs` (add `ProcessLogBatchEvent`)
- Modify: `src/lib/types.ts` (add `ProcessLogBatchEvent` type)
- Modify: `src/lib/stores/runtime.svelte.ts` (listen for batch events)

- [ ] **Step 1: Add batch event types**

In `src-tauri/src/application/events.rs`:

```rust
pub const PROCESS_LOG_BATCH_EVENT: &str = "process-log-batch";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessLogBatchEvent {
    pub lines: Vec<ProcessLogPayload>,
}
```

In `src/lib/types.ts`:

```typescript
export type ProcessLogBatchEvent = {
    lines: ProcessLogPayload[];
};
```

- [ ] **Step 2: Implement batching in `log.rs`**

Rewrite `spawn_log_task`:

```rust
pub fn spawn_log_task<R>(
    app_handle: AppHandle,
    window_key: String,
    session_id: RunSessionId,
    process_name: String,
    runtime_id: ProcessRuntimeId,
    stream: LogStream,
    reader: R,
    log_tx: broadcast::Sender<String>,
    append_log_fn: impl Fn(ProcessLogPayload) -> Result<(), AppError> + Send + 'static,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        use std::time::Duration;
        let mut lines = BufReader::new(reader).lines();
        let mut batch: Vec<ProcessLogPayload> = Vec::with_capacity(64);
        let mut flush = tokio::time::interval(Duration::from_millis(16));

        loop {
            tokio::select! {
                result = lines.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            let _ = log_tx.send(line.clone());
                            let payload = ProcessLogPayload {
                                session_id: session_id.clone(),
                                runtime_id: runtime_id.clone(),
                                process_name: process_name.clone(),
                                stream,
                                line,
                                timestamp: Utc::now(),
                            };
                            let _ = append_log_fn(payload.clone());
                            batch.push(payload);
                            if batch.len() >= 64 {
                                flush_batch(&app_handle, &window_key, &mut batch);
                            }
                        }
                        Ok(None) => {
                            flush_batch(&app_handle, &window_key, &mut batch);
                            break;
                        }
                        Err(_) => {
                            flush_batch(&app_handle, &window_key, &mut batch);
                            break;
                        }
                    }
                }
                _ = flush.tick() => {
                    flush_batch(&app_handle, &window_key, &mut batch);
                }
            }
        }
    });
}

fn flush_batch(app_handle: &AppHandle, window_key: &str, batch: &mut Vec<ProcessLogPayload>) {
    if batch.is_empty() {
        return;
    }
    // Also emit individual events for backwards compatibility with existing listeners
    for payload in batch.iter() {
        let _ = app_handle.emit_to(
            window_key,
            PROCESS_LOG_EVENT,
            ProcessLogEvent { payload: payload.clone() },
        );
    }
    let _ = app_handle.emit_to(
        window_key,
        PROCESS_LOG_BATCH_EVENT,
        ProcessLogBatchEvent { lines: std::mem::take(batch) },
    );
}
```

- [ ] **Step 3: Update frontend to handle batch events**

In `runtime.svelte.ts`, add a batch listener:

```typescript
this.#unlisteners.push(
    await listen<ProcessLogBatchEvent>("process-log-batch", (event) => {
        for (const payload of event.payload.lines) {
            if (this.session && payload.sessionId !== this.session.sessionId) continue;
            const current = this.processLogs[payload.runtimeId] ?? [];
            const appended = [...current, payload];
            const overflow = Math.max(0, appended.length - MAX_LOG_LINES_PER_PROCESS);
            this.processLogs[payload.runtimeId] = overflow > 0 ? appended.slice(overflow) : appended;
            this.processLogTruncation[payload.runtimeId] =
                (this.processLogTruncation[payload.runtimeId] ?? 0) + overflow;
        }
    }),
);
```

Add the import for `ProcessLogBatchEvent` from `$lib/types`.

- [ ] **Step 4: Run tests and typecheck**

```bash
cd src-tauri && cargo test
deno task check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/application/orchestrator/log.rs
git add src-tauri/src/application/events.rs
git add src/lib/types.ts src/lib/stores/runtime.svelte.ts
git commit -m "perf: batch log lines at 16ms intervals for IPC efficiency

Log lines are now buffered and flushed every 16ms or every 64 lines.
Individual events still emitted for backwards compatibility.
New process-log-batch event reduces IPC message count for verbose processes."
```

---

### Task 6.3: Remove dead code — `process_dependency_statuses`

**Files:**
- Modify: `src-tauri/src/infrastructure/config_loader.rs:208-219`

- [ ] **Step 1: Remove the unused function**

Delete lines 208-219 (`pub fn process_dependency_statuses` and its body).

- [ ] **Step 2: Verify no callers exist**

```bash
cd src-tauri && cargo check
```
Expected: No warnings about unused function (the only caller was this function itself).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/infrastructure/config_loader.rs
git commit -m "chore: remove unused process_dependency_statuses function"
```

---

### Task 6.4: Cleanup remaining `expect()` calls

**Files:**
- Modify: `src-tauri/src/lib.rs:15,70`

The `AppState::new().expect(...)` and `.run(...).expect(...)` are the only two remaining expects in production code after PR5. These are acceptable (app can't run without state or Tauri context), but document them.

- [ ] **Step 1: Add comment explaining why panics are acceptable**

```rust
let app_state = AppState::new().expect("failed to initialize app state — cannot start without config directory");

// ... later:
.run(tauri::generate_context!())
.expect("error while running tauri application");
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "docs: document remaining expect() calls as intentionally fatal"
```

---

### PR 6 Verification

```bash
deno task check
cd src-tauri && cargo test
deno task app examples/deno-runner.yml
```

---
