# Sidebar Contextual Process Action Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the sidebar's single Restart button with a single contextual button that Stops a running process or Starts a stopped/failed process, backed by a new `start_process` backend command.

**Architecture:** New Rust `start_process` command (reset→spawn, no stop step) wired through the Tauri invoke handler, exposed via a typed TS client wrapper and a runtime-store method, and rendered by `ProcessList.svelte` as one state-aware button per process row.

**Tech Stack:** Rust + Tauri 2 (backend), Svelte 5 (runes) + TypeScript + Vitest + @testing-library/svelte (frontend).

**Spec:** `docs/superpowers/specs/2026-06-21-sidebar-contextual-process-action-design.md`

**Tooling rules (from AGENTS.md):** Use `deno task` as the JS entrypoint — do NOT run `npm`/`pnpm`/`yarn` directly. Backend tests via `cd src-tauri && cargo test`. The repo is **not** a git repository, so there are no commit steps; verification is via `deno task check`, `deno task test`, and `cargo test`.

---

## File Structure

**Backend (Rust):**
- Modify `src-tauri/src/application/orchestrator.rs` — add `start_process` method.
- Modify `src-tauri/src/tauri_api/commands.rs` — add `start_process` Tauri command.
- Modify `src-tauri/src/lib.rs:44-45` — register `start_process` in `generate_handler!`.

**Frontend (TS/Svelte):**
- Modify `src/lib/tauri/client.ts` — add `startProcess` wrapper.
- Modify `src/lib/stores/runtime.svelte.ts` — add `startSessionProcess` method.
- Modify `src/lib/components/ProcessList.svelte` — contextual button.
- Create `src/lib/components/ProcessList.test.ts` — component tests for the button.
- Modify `src/routes/+page.svelte` — wire `onStart`/`onStop` into `<ProcessList>`.

---

## Task 1: Backend `start_process` orchestrator method

**Files:**
- Modify: `src-tauri/src/application/orchestrator.rs` (add method inside `impl ProcessOrchestrator`, right after `restart_process` ending at line 167, before `stop_process` at line 169)

- [ ] **Step 1: Add the `start_process` method**

Insert this method immediately after the closing brace of `restart_process` (line 167) and before `pub async fn stop_process` (line 169):

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

This mirrors `restart_process` (lines 155–167) but omits the `self.stop_process(...)` call, so a failed/stopped process is reset and re-spawned without flipping `terminating` or killing an already-dead child.

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo build`
Expected: builds with no errors. (A dead-code warning for the unused method is expected until Task 3 wires it into the invoke handler — that is fine, do not add `#[allow(dead_code)]`.)

---

## Task 2: Backend Tauri command + registration

**Files:**
- Modify: `src-tauri/src/tauri_api/commands.rs` (add command after `restart_process`, which ends at line 235, before `stop_process` at line 237)
- Modify: `src-tauri/src/lib.rs` (add one line inside `generate_handler!`, after line 44 `restart_process`)

- [ ] **Step 1: Add the `start_process` Tauri command**

In `src-tauri/src/tauri_api/commands.rs`, insert this command immediately after the closing brace of `restart_process` (line 235) and before `pub async fn stop_process` (line 237):

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

It reuses the existing `ProcessActionRequest` type already used by `restart_process`/`stop_process` (defined elsewhere in this file). The signature and error mapping are identical to `restart_process`.

- [ ] **Step 2: Register the command in the invoke handler**

In `src-tauri/src/lib.rs`, add one line inside the `tauri::generate_handler![...]` block, directly after line 44 (`tauri_api::commands::restart_process,`):

```rust
            tauri_api::commands::start_process,
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo build`
Expected: builds with no errors and no dead-code warning (the command is now referenced by the handler).

---

## Task 3: Backend test for `start_process`

**Files:**
- Modify: `src-tauri/src/application/orchestrator.rs` (the `#[cfg(test)] mod tests` block, which starts at line 734)

- [ ] **Step 1: Write the failing test**

The existing tests in this module test **pure helper functions** (`dependencies_satisfied`, `sync_snapshot_process`) rather than the async orchestrator methods (which require a live `AppHandle` and real process spawning). Follow that same boundary: test the observable state effect of `reset_process` — which is the core of what `start_process` calls before spawning — by asserting that a `Failed`/`Stopped` process is reset to `Pending` with `terminating=false`.

`reset_process` is currently a private async method that takes `&self` and locks `inner`. To make it unit-testable without spinning up a real session, refactor it minimally: extract a pure helper that resets a `&mut ManagedProcess` in place, then have the async `reset_process` call it under the lock. This keeps the async method's behavior identical while making the logic testable (matching the pattern of the existing pure-helper tests).

First, add a free helper function near the other free functions in this file (place it just above the `#[cfg(test)]` block, after `sync_snapshot_process`):

```rust
fn reset_managed_process(process: &mut ManagedProcess) {
    process.child = None;
    process.terminating = false;
    process.snapshot.status = ProcessStatus::Pending;
    process.snapshot.started_at = None;
    process.snapshot.exited_at = None;
    process.snapshot.exit_code = None;
}
```

Then change the existing `reset_process` method body (lines 214–232) to use it. Replace the lines that mutate `process` (currently lines 224–229):

```rust
        reset_managed_process(process);
        sync_snapshot_process(&mut active.snapshot, &process.snapshot);
        Ok(())
```

(Remove the now-redundant direct field assignments `process.child = None;` through `process.snapshot.exit_code = None;` — they move into the helper.)

Then add this test inside the `mod tests` block (after the last existing test, before the closing `}` of the module at line 857):

```rust
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
```

`managed_process` and `Utc` are already imported in the test module (lines 736–737, 746).

- [ ] **Step 2: Run the test and verify it passes**

Run: `cd src-tauri && cargo test reset_managed_process_restores_pending_state`
Expected: PASS (1 test). Since we extracted `reset_managed_process` from the already-correct `reset_process` body, the test passes immediately; this guards the behavior that `start_process` relies on.

- [ ] **Step 3: Run the full backend test suite**

Run: `cd src-tauri && cargo test`
Expected: all tests PASS, including the pre-existing `dependencies_satisfied_*` and `sync_snapshot_process_*` tests (confirming the refactor didn't change behavior).

---

## Task 4: Frontend `startProcess` client wrapper

**Files:**
- Modify: `src/lib/tauri/client.ts` (add function after `restartProcess`, which ends at line 75, before `stopProcess` at line 77)

- [ ] **Step 1: Add the `startProcess` function**

Insert after the closing brace of `restartProcess` (line 75) and before `export async function stopProcess` (line 77):

```ts
export async function startProcess(
  processName: string,
): Promise<RunSessionSnapshot | null> {
  return invoke<RunSessionSnapshot | null>("start_process", {
    request: { processName },
  });
}
```

This mirrors `restartProcess` exactly, only changing the command name to `"start_process"`.

- [ ] **Step 2: Verify the frontend type-checks**

Run: `deno task check`
Expected: passes (the known non-blocking `tsconfig.json` extension warning from SvelteKit may appear; per AGENTS.md that alone is not a failure if the command exits successfully).

---

## Task 5: Frontend `startSessionProcess` store method

**Files:**
- Modify: `src/lib/stores/runtime.svelte.ts` (add method right after `restartSessionProcess`, which ends at line 214, before `stopSessionProcess` at line 216)
- Modify: `src/lib/stores/runtime.svelte.ts` import section (add `startProcess` to the import from `$lib/tauri/client`)

- [ ] **Step 1: Add `startProcess` to the client import**

Find the import statement that brings in `restartProcess`/`stopProcess` from `$lib/tauri/client` (used at lines 206 and 219) and add `startProcess` to it. For example, if it reads:

```ts
import { ..., restartProcess, stopProcess } from "$lib/tauri/client";
```

change it to:

```ts
import { ..., restartProcess, startProcess, stopProcess } from "$lib/tauri/client";
```

(Keep the existing identifiers and ordering style; just insert `startProcess`.)

- [ ] **Step 2: Add the `startSessionProcess` method**

Insert this method immediately after `restartSessionProcess`'s closing brace (line 214) and before `async stopSessionProcess` (line 216):

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

This is identical in shape to `restartSessionProcess` (lines 203–214), calling `startProcess` instead.

- [ ] **Step 3: Verify the frontend type-checks**

Run: `deno task check`
Expected: passes (same SvelteKit warning caveat as above).

---

## Task 6: Frontend `ProcessList` contextual button

**Files:**
- Modify: `src/lib/components/ProcessList.svelte` (full rewrite of the action-button area and props)

- [ ] **Step 1: Update the `Props` type**

In `src/lib/components/ProcessList.svelte`, in the `type Props = { ... }` block (lines 10–20), replace the `onRestart` line:

```ts
    onRestart: (processName: string) => void;
```

with two new props:

```ts
    onStop: (processName: string) => void;
    onStart: (processName: string) => void;
```

- [ ] **Step 2: Update the destructured props**

In the `let { ... }: Props = $props();` block (lines 22–32), replace `onRestart,` with:

```ts
    onStop,
    onStart,
```

- [ ] **Step 3: Replace the restart predicate with action helpers**

Replace the `restartDisabledStatuses` set and `canRestart` function (lines 34–43) with state-aware helpers that classify a process into one of three actions:

```ts
  // A process row exposes a single contextual action that depends on its state:
  //   running/ready/starting -> Stop
  //   stopped/failed/succeeded -> Start
  //   pending/blocked/stopping -> none (button disabled)
  type RowAction = "stop" | "start" | null;

  function rowAction(status: ProcessStatus): RowAction {
    switch (status) {
      case "running":
      case "ready":
      case "starting":
        return "stop";
      case "stopped":
      case "failed":
      case "succeeded":
        return "start";
      default:
        return null;
    }
  }

  function actionEnabled(status: ProcessStatus) {
    return !busy && rowAction(status) !== null;
  }
```

- [ ] **Step 4: Replace the action button markup**

Replace the entire `<div class="flex shrink-0 ...">...</div>` block for processes (lines 82–101, the one containing the Restart button — NOT the terminal close button block at lines 128–146) with this contextual button:

```svelte
        <div
          class="flex shrink-0 items-center gap-0.5 transition md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
        >
          {@const action = rowAction(process.status)}
          {@const danger = action === "stop"}
          <button
            type="button"
            class={`grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-40 ${danger ? "hover:bg-danger/10 hover:text-danger" : "hover:text-text"}`}
            disabled={!actionEnabled(process.status)}
            aria-label={`${action === "stop" ? "Stop" : "Start"} ${process.name}`}
            title={`${action === "stop" ? "Stop" : "Start"} ${process.name}`}
            onclick={(event) => {
              event.stopPropagation();
              if (action === "stop") {
                onStop(process.name);
              } else if (action === "start") {
                onStart(process.name);
              }
            }}
          >
            {#if action === "stop"}
              <!-- Stop: filled square -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <rect x="6" y="6" width="12" height="12" rx="1.5" />
              </svg>
            {:else}
              <!-- Start: triangle -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M7 5l12 7-12 7z" />
              </svg>
            {/if}
          </button>
        </div>
```

Notes on the markup:
- One button per row (never two side by side).
- The `danger` class branch makes the Stop hover red (`hover:text-danger`), matching the sibling terminal-close button's styling; Start stays neutral.
- Hover/focus visibility (`md:opacity-0 → group-hover:opacity-100`) is preserved exactly.
- `event.stopPropagation()` is preserved so the row's select handler doesn't fire.
- The `aria-label`/`title` switch to `Stop {name}` or `Start {name}` based on the resolved action.

- [ ] **Step 5: Verify the frontend type-checks**

Run: `deno task check`
Expected: passes. (If it errors about `onRestart` still being referenced, double-check Task 6 Steps 1–2 removed all `onRestart` references.)

---

## Task 7: Frontend `ProcessList` component tests

**Files:**
- Create: `src/lib/components/ProcessList.test.ts`

- [ ] **Step 1: Write the test file**

Create `src/lib/components/ProcessList.test.ts` with this content. It follows the exact style of `src/lib/components/RunStopButton.test.ts`: `render` + `getByRole("button", { name })` + `fireEvent.click` + `toBeDisabled()`, with `vi.fn()` callbacks (no Tauri transport mocking needed because `ProcessList` receives callbacks as props).

```ts
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";

import ProcessList from "./ProcessList.svelte";
import type { ProcessSnapshot, TerminalSnapshot } from "$lib/types";

function makeProcess(overrides: Partial<ProcessSnapshot> = {}): ProcessSnapshot {
  return {
    runtimeId: "r1",
    name: "api",
    kind: "service",
    status: "running",
    ...overrides,
  };
}

const terminal: TerminalSnapshot = {
  terminalId: "t1",
  title: "bash",
  cwd: "/home/leo/devapp",
  createdAt: "2026-01-01T00:00:00.000Z",
  isOpen: true,
};

function makeProps(overrides: Record<string, unknown> = {}) {
  return {
    processes: [] as ProcessSnapshot[],
    terminals: [] as TerminalSnapshot[],
    selectedProcessRuntimeId: null,
    selectedTerminalId: null,
    busy: false,
    onSelectProcess: vi.fn(),
    onSelectTerminal: vi.fn(),
    onStop: vi.fn(),
    onStart: vi.fn(),
    onCloseTerminal: vi.fn(),
    ...overrides,
  };
}

describe("ProcessList action button", () => {
  describe("when the process is running", () => {
    it("renders a Stop button", () => {
      const { getByRole, queryByRole } = render(ProcessList, {
        props: makeProps({ processes: [makeProcess({ status: "running" })] }),
      });

      expect(getByRole("button", { name: "Stop api" })).toBeInTheDocument();
      expect(queryByRole("button", { name: "Start api" })).toBeNull();
    });

    it("calls onStop when clicked", async () => {
      const onStop = vi.fn();
      const { getByRole } = render(ProcessList, {
        props: makeProps({
          processes: [makeProcess({ status: "running" })],
          onStop,
        }),
      });

      await fireEvent.click(getByRole("button", { name: "Stop api" }));

      expect(onStop).toHaveBeenCalledOnce();
      expect(onStop).toHaveBeenCalledWith("api");
    });
  });

  describe("when the process is ready", () => {
    it("renders a Stop button", () => {
      const { getByRole } = render(ProcessList, {
        props: makeProps({ processes: [makeProcess({ status: "ready" })] }),
      });

      expect(getByRole("button", { name: "Stop api" })).toBeInTheDocument();
    });
  });

  describe("when the process is failed", () => {
    it("renders a Start button", () => {
      const { getByRole, queryByRole } = render(ProcessList, {
        props: makeProps({ processes: [makeProcess({ status: "failed" })] }),
      });

      expect(getByRole("button", { name: "Start api" })).toBeInTheDocument();
      expect(queryByRole("button", { name: "Stop api" })).toBeNull();
    });

    it("calls onStart when clicked", async () => {
      const onStart = vi.fn();
      const { getByRole } = render(ProcessList, {
        props: makeProps({
          processes: [makeProcess({ status: "failed" })],
          onStart,
        }),
      });

      await fireEvent.click(getByRole("button", { name: "Start api" }));

      expect(onStart).toHaveBeenCalledOnce();
      expect(onStart).toHaveBeenCalledWith("api");
    });
  });

  describe("when the process is stopped or succeeded", () => {
    it.each(["stopped", "succeeded"] as const)(
      "renders a Start button for status %s",
      (status) => {
        const { getByRole } = render(ProcessList, {
          props: makeProps({ processes: [makeProcess({ status })] }),
        });

        expect(getByRole("button", { name: "Start api" })).toBeInTheDocument();
      },
    );
  });

  describe("when the process is in a transitional state", () => {
    it.each(["pending", "blocked", "stopping"] as const)(
      "disables the action button for status %s",
      (status) => {
        const { getByRole } = render(ProcessList, {
          props: makeProps({ processes: [makeProcess({ status })] }),
        });

        // The button still renders (with a resolved label of "Start" since
        // rowAction returns null for these), but it must be disabled.
        expect(getByRole("button")).toBeDisabled();
      },
    );
  });

  describe("when busy", () => {
    it("disables the action button even for a running process", () => {
      const { getByRole } = render(ProcessList, {
        props: makeProps({
          processes: [makeProcess({ status: "running" })],
          busy: true,
        }),
      });

      expect(getByRole("button", { name: "Stop api" })).toBeDisabled();
    });
  });
});
```

- [ ] **Step 2: Run the tests and verify they pass**

Run: `deno task test`
Expected: all `ProcessList action button` tests PASS, plus all pre-existing tests (RunStopButton, ProjectMenu, Menu) still pass.

- [ ] **Step 3: Fix the transitional-state label if needed**

If the transitional-state test fails because `getByRole("button")` matches ambiguously (e.g., the row's select `<button>` is also in the DOM), scope the query. The select button has no `aria-label`, so `getByRole("button", { name: /Start|Stop/ })` targets only the action button. Adjust the transitional-state `it.each` block to:

```ts
        expect(getByRole("button", { name: /Start api|Stop api/ })).toBeDisabled();
```

Re-run `deno task test` until green. (Decide between the original and this fallback based on whether the row select button pollutes the query.)

---

## Task 8: Wire `onStart`/`onStop` into `+page.svelte`

**Files:**
- Modify: `src/routes/+page.svelte:117` (the `onRestart` prop of `<ProcessList>`)

- [ ] **Step 1: Replace the `onRestart` prop**

In `src/routes/+page.svelte`, find the `<ProcessList ... />` invocation (lines 109–122). Replace the single `onRestart` line (line 117):

```svelte
          onRestart={(processName) => runtimeStore.restartSessionProcess(processName)}
```

with two props:

```svelte
          onStart={(processName) => runtimeStore.startSessionProcess(processName)}
          onStop={(processName) => runtimeStore.stopSessionProcess(processName)}
```

Do **not** remove `runtimeStore.restartSessionProcess` from anywhere else — it is still used by `ProjectMenu` (line 155, `onRestartProcess`). Only the `ProcessList` invocation changes.

- [ ] **Step 2: Verify the frontend type-checks**

Run: `deno task check`
Expected: passes. This confirms `ProcessList`'s new props match and `startSessionProcess`/`stopSessionProcess` exist on the store.

---

## Task 9: Final verification

- [ ] **Step 1: Run the full frontend check + tests**

Run: `deno task check && deno task test`
Expected: both pass. (`deno task check` may emit the known SvelteKit `tsconfig.json` warning; that is acceptable per AGENTS.md if the command exits 0.)

- [ ] **Step 2: Run the full backend test suite**

Run: `cd src-tauri && cargo test`
Expected: all tests PASS, including the new `reset_managed_process_restores_pending_state`.

- [ ] **Step 3: Build the app**

Run: `deno task build`
Expected: builds successfully (frontend bundles; Tauri backend compiles). The known SvelteKit warning caveat applies.

- [ ] **Step 4: Manual smoke test (optional but recommended)**

Run: `deno task app examples/deno-runner.yml`
Then in the running app:
1. Start the session from the sidebar header Run button.
2. Hover a running process row → confirm a **Stop** (filled square) button appears; click it → process stops.
3. Hover the now-stopped process row → confirm a **Start** (triangle) button appears; click it → process restarts.
4. Confirm the ⚙ menu still offers both Stop and Restart for the selected process.

---

## Notes for the implementer

- **No git in this repo.** There are no commit steps. Verification is `deno task check`, `deno task test`, and `cd src-tauri && cargo test`.
- **`reset_process` refactor in Task 3 is a pure extraction** — the async method keeps identical behavior; only the in-place mutation moves to a testable free function. This is the lightest way to add test coverage that matches the existing module's style (which tests pure helpers, not async orchestrator methods needing a live `AppHandle`).
- **`spawn_runnable_processes` already re-spawns `Stopped`/`Pending`/`Blocked` processes** and short-circuits when `stop_requested` is true, so `start_process` during a session-wide stop won't race shutdown.
- **`start_process` is never reachable for a running process** from the UI: the contextual button shows Stop there, and the ⚙ menu doesn't expose `start_process`. So the no-op "start a running process" path is not a concern.
