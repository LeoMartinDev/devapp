# Architecture Hardening — Design Document

**Date:** 2026-06-26
**Status:** approved
**Scope:** `src-tauri/` (Rust/Tauri backend) + `src/` (Svelte frontend, when needed)

## Context

A comprehensive architecture review of `src-tauri/` (25 Rust files, ~3,800 lines, 6 modules, 18
Tauri commands, 25 tests) surfaced 21 issues across four categories: bugs (6), architecture (6),
performance (4), and code-quality improvements (5).

The user wants to fix all 21 issues following an incremental PR strategy.

## Design Decisions

### Delivery: 6 Incremental PRs

Each PR addresses 3-5 related issues. PRs are sequenced so each builds on the previous only when
structurally necessary. This keeps each PR reviewable, revertible, and shippable independently.

### Strategy: Backend + Frontend Together

When a backend change requires frontend work (structured error codes, batched log events), both
sides are delivered in the same PR. This prevents merge-order coupling where backend ships a
breaking API change before the frontend is ready.

### Ordering Philosophy

- **Critical bugs first** — The environment inheritance bug prevents real-world usage. Fix it first.
- **Structural work before tests** — Split the orchestrator before adding integration tests so
  tests target well-defined module seams.
- **Observability before hygiene** — Add tracing/logging before optimizing threads/mutexes, so
  the optimization effort is measurable.
- **Performance last** — Optimizations are safe to defer; they benefit from having the rest of the
  system stable.

---

## PR Breakdown

### PR 1: Fix Critical Bugs

**Issues addressed:** 5 (4 bugs + 1 dead code)

| Issue | Location | Fix |
|-------|----------|-----|
| Spawned processes lack `PATH` | `orchestrator.rs:785` `build_process_env()` | Seed env map with `std::env::vars()`, then merge global + per-process overrides |
| Readiness command also lacks system env | `readiness.rs:114` | Same fix — the env map passed to `wait_until_command_ready` will already be correct after fixing `build_process_env` |
| TOCTOU race: process spawned after session stopped | `orchestrator.rs:343-368` | Add `stop_requested` guard in the second lock block; kill newly-spawned child if session is already stopping |
| Broadcast channel lag drops readiness patterns | `orchestrator.rs:106` | Increase broadcast channel capacity from 256 to 4096; ensure readiness subscriber creates its receiver before log tasks start producing |
| `close_all_for_window` short-circuits on first error | `pty.rs:245` | Replace `?` with `.ok();` |
| Dead assignment `let _ = project_name;` | `orchestrator.rs:494` | Remove the extraction at line 320 and the suppression at 494 |

**Verification:**
- `deno task check` passes
- `cargo test` passes
- Manual: `deno task app examples/deno-runner.yml` — processes spawn and run successfully
- Manual: Stop a project with multiple terminals open — all terminals close

**No frontend changes.**

---

### PR 2: Split the Orchestrator God Module

**Issues addressed:** 1 (arch)

| Issue | Location | Fix |
|-------|----------|-----|
| `orchestrator.rs` is 1113 lines, 18 methods | `application/orchestrator.rs` | Split into 4 sub-modules |

**Target module structure:**

```
src-tauri/src/application/
├── mod.rs
├── command_runner.rs      (unchanged)
├── events.rs              (unchanged)
├── readiness.rs           (unchanged)
├── orchestrator/
│   ├── mod.rs             — re-exports, ProcessOrchestrator (facade)
│   ├── session.rs         — SessionManager: create/delete session, stop_requested flag
│   ├── lifecycle.rs       — ProcessLifecycle: spawn, stop, restart, status transitions
│   ├── dependency.rs      — DepResolver: runnable processes, dependency satisfaction
│   └── log.rs             — LogDistributor: log task spawning, store integration
```

**Seams (interfaces between modules):**

- **SessionManager** exposes: `create()`, `get()/get_mut()`, `is_stop_requested()`, `mark_stopped()`
- **ProcessLifecycle** exposes: `spawn()`, `kill()`, `restart()`, `transition_status()`
- **DepResolver** exposes: `runnable_processes()`, `dependencies_satisfied()` (pure, no state)
- **LogDistributor** exposes: `spawn_log_tasks()`, `append_log()`

**Rules:**
- SessionManager owns the `ActiveSession` struct and the `HashMap<String, ActiveSession>`
- ProcessLifecycle, DepResolver, and LogDistributor operate on borrowed `&mut ActiveSession`
- `ProcessOrchestrator` becomes a thin facade that delegates to each module
- All existing public methods on `ProcessOrchestrator` keep their signatures unchanged
- Tests move to the module they test

**Verification:**
- `deno task check` + `cargo test` pass
- All 7 orchestrator tests still pass (now in their respective sub-modules)
- Manual smoke: start/stop/restart a project — behavior unchanged

**No frontend changes** (the facade preserves the public API).

---

### PR 3: Integration Tests for Full Lifecycle

**Issues addressed:** 1 (arch)

| Issue | Location | Fix |
|-------|----------|-----|
| Zero integration tests | n/a | Add `src-tauri/tests/` directory with at least one lifecycle test |

**Test: `session_lifecycle_integration_test`**

```rust
// src-tauri/tests/session_lifecycle.rs
#[tokio::test]
async fn task_succeeds_and_session_stops() {
    // 1. Write a devapp.yml with a "hello" task (echo hello && exit 0)
    // 2. Call start_session with that config
    // 3. Poll snapshot until hello reaches Succeeded
    // 4. Call stop_session
    // 5. Verify snapshot.stopped_at is Some
}
```

**Additional tests (if time permits):**
- Service with `ready.type: delay` reaches Ready → unblocks dependent process
- Process failure triggers session stop
- `restart_process` resets a Failed service back to Running

**Verification:**
- `cargo test` in `src-tauri/` includes the new integration tests
- Tests use `tempfile` for isolated config directories

**No frontend changes.**

---

### PR 4: Structured Error Codes + Tracing

**Issues addressed:** 4 (1 arch + 3 improvements)

| Issue | Location | Fix |
|-------|----------|-----|
| Stringly-typed errors | `error.rs` | Add `ErrorCode` enum with 10-15 codes; each `AppError` variant carries a `code` field |
| No structured logging | entire codebase | Add `tracing` crate; instrument session/process lifecycle events |
| `fs::canonicalize` blocks async in async context | `config_loader.rs:23` | Wrap in `tokio::task::spawn_blocking` |
| Readiness error lacks process name | `orchestrator.rs:421` | Include process name in error message |

**Error code design:**

```rust
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
```

**Frontend changes:**
- `src/lib/tauri/client.ts`: update error handling to extract `ErrorCode` from the structured error response
- `src/lib/stores/runtime.svelte.ts`: show targeted recovery actions based on error code
- Error display component: show a short user-facing message per error code (no string parsing)

**Tracing instrumentation points:**
- `start_session`: info!("starting session for project {}", project.name)
- `spawn_named_process`: info!(process = %name, pid = ?pid, "process started")
- `handle_process_exit`: info!(process = %name, exit_code = ?code, "process exited")
- `handle_process_failure`: error!(process = %name, error = %msg, "process failed")
- Readiness timeout: warn!("readiness timeout for {process}")
- `finish_session`: info!("session stopped")

**Verification:**
- `deno task check` + `cargo test` pass
- Frontend: error messages display correctly with codes

---

### PR 5: Thread/Task Hygiene + State Standardization

**Issues addressed:** 5 (2 arch + 3 improvements)

| Issue | Location | Fix |
|-------|----------|-----|
| `thread::spawn` + `block_on` anti-pattern | `orchestrator.rs:401,452` | Replace with `tokio::spawn` for readiness watcher and exit watcher |
| Mixed Mutex types | `pty.rs:24` vs `orchestrator.rs:38` | Convert `TerminalManager` to `tokio::sync::Mutex`; PTY reader uses `Handle::block_on` |
| `setup()` errors silently swallowed | `lib.rs:35-46` | Log the error with `tracing` before mapping; store in AppState if recoverable |
| `load_project_config` holds lock during YAML parse | `commands.rs:163-183` | Clone project data, drop lock, parse outside |
| `save_project_config` same issue | `commands.rs:186-205` | Same fix |

**PTY Mutex migration:**
- `TerminalManager.inner` changed from `Arc<std::sync::Mutex<...>>` to `Arc<tokio::sync::Mutex<...>>`
- Reader thread (pty.rs:108) gets a `tokio::runtime::Handle` and calls `handle.block_on(async { inner.lock().await })`
- `close_all_for_window` no longer blocks the Tokio worker during mutex acquisition

**Verification:**
- `deno task check` + `cargo test` pass
- Manual: open/close/resize terminals — no regressions
- Manual: stop project with terminals open — all close cleanly

**No frontend changes.**

---

### PR 6: Performance Optimizations + Cleanup

**Issues addressed:** 5 (3 perf + 1 improvement + 1 arch)

| Issue | Location | Fix |
|-------|----------|-----|
| Full snapshot clone on every emit | `orchestrator.rs:752` | Wrap `RunSessionSnapshot` in `Arc`; use `Arc::make_mut` for COW updates |
| Each log line = 1 IPC event | `orchestrator.rs:525` | Batch lines at ~16ms intervals using a `tokio::time::interval` |
| `list()` clones 10K entries | `log_store.rs:44` | Return `&[ProcessLogPayload]` or add pagination; alternatively, accept clone cost for now (not called) |
| `process_dependency_statuses` dead code | `config_loader.rs:208` | Wire to a Tauri command or remove |
| `expect()` in lib.rs for decorations | `lib.rs:24,31` | Log warning instead of panicking; macOS/Windows can recover gracefully |

**Log batching design:**

```rust
// New event type
struct ProcessLogBatchEvent {
    lines: Vec<ProcessLogPayload>,
}

// In spawn_log_task:
let mut batch = Vec::with_capacity(64);
let mut flush = tokio::time::interval(Duration::from_millis(16));
loop {
    tokio::select! {
        line = lines.next_line() => {
            batch.push(build_payload(line));
            if batch.len() >= 64 { flush_batch().await; batch.clear(); }
        }
        _ = flush.tick() => {
            if !batch.is_empty() { flush_batch().await; batch.clear(); }
        }
    }
}
```

**Frontend changes:**
- Listen for `process-log-batch` event in addition to (or replacing) `process-log`
- Append all lines from the batch at once to the log store

**Snapshot Arc migration:**
- `ActiveSession.snapshot` becomes `Arc<RunSessionSnapshot>`
- `sync_snapshot_process` uses `Arc::make_mut` before mutating
- `emit_snapshot` clones only the `Arc` (pointer copy, not deep clone)

**Verification:**
- `deno task check` + `cargo test` pass
- Performance: spawn a project with a verbose process (e.g., `yes`), verify UI doesn't lag
- Memory: snapshot clone count drops from N-per-status-change to 1-per-status-change

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Orchestrator split breaks subtle state machine logic | Medium | PR2 keeps public API unchanged; existing unit tests pass; PR3 adds integration tests |
| `tokio::spawn` for exit watcher changes task lifetime | Low | Tokio spawned tasks outlive their spawning future; the tasks are fire-and-forget already |
| PTY Mutex migration introduces deadlocks | Low | Single lock held briefly; no nested locking in PTY paths |
| Log batching changes frontend rendering behavior | Low | Batch preserves line order; frontend appends lines at once |

## Success Criteria

After all 6 PRs are merged:
1. `deno task check` passes
2. `cargo test` passes (including new integration tests)
3. `deno task app examples/deno-runner.yml` starts and runs correctly
4. Project start/stop/restart lifecycle works end-to-end
5. Terminals open, receive output, resize, and close correctly
6. Errors display with actionable codes in the frontend
7. Structured logs are emitted to stderr for debugging
8. No orphan processes after rapid start/stop cycles
9. No performance degradation under high log volume (1000+ lines/second)
