# Devapp Project

Devapp is a desktop application for running and supervising a project's local
development process graph from a YAML file. The goal is to replace ad-hoc
terminal tabs, handwritten startup notes, and fragile command ordering with a
small app that knows what to start, when a service is ready, where logs belong,
and how to stop everything cleanly.

The product is intentionally local-first. It is not a cloud orchestrator, a CI
runner, or a team dashboard. V1 focuses on one developer running one or more
local projects from a Tauri desktop app.

## What We Are Building

Devapp reads a project configuration file, usually `devapp.yml`, and turns it
into a supervised runtime session.

The configuration describes:

- global environment variables;
- named processes;
- whether each process is a short-lived `task` or long-lived `service`;
- shell commands to run;
- dependencies between processes;
- readiness checks that tell Devapp when a service can unblock dependents.

From that configuration, Devapp provides:

- a desktop UI for selecting or importing projects;
- a runtime toolbar to start, stop, configure, and open terminals;
- a process list with status and safe process actions;
- per-process logs with filtering, pausing, truncation feedback, and clearing;
- an integrated xterm terminal attached to the project directory;
- a YAML/form configuration editor with validation;
- project isolation by window, so different projects can run at the same time.

## Primary Workflow

The intended workflow is:

1. A developer opens Devapp with a config file or registers a project directory.
2. Devapp resolves and validates the YAML configuration.
3. The developer starts the project.
4. The Rust runtime starts runnable processes in dependency order.
5. Tasks unblock dependents when they exit successfully.
6. Services unblock dependents when their readiness check passes.
7. Logs stream into the UI and are scoped to the selected process.
8. The developer can inspect logs, open a terminal, restart or stop processes,
   edit configuration, or stop the whole project.

For a self-contained example:

```sh
deno task app examples/deno-runner.yml
```

That launches the Tauri app, imports the YAML as a project, and starts it
through the same desktop runtime used by the UI.

## Runtime Model

The Rust backend is the source of truth for runtime behavior. The frontend
requests actions, but does not directly spawn or manage configured processes.

Important rules:

- A desktop window owns at most one active project session.
- Multiple projects can run at once by using independent Tauri windows.
- Runtime events, logs, selected process, selected terminal, and UI errors are
  scoped to the current window.
- A blocking failure stops the whole session.
- Stopping a session stops all running configured processes and project
  terminals for that window.
- Logs are stored in memory in V1 and capped at 10,000 lines per process in the
  frontend.

Process states are:

```text
pending -> starting -> running -> ready
pending -> starting -> running -> succeeded
pending -> starting -> running -> failed
running/ready/starting -> stopping -> stopped
```

`task` processes are expected to exit. `service` processes are expected to stay
alive until the user stops them or the session fails.

## Configuration Model

The V1 YAML schema is deliberately small:

```yaml
version: 1
env:
  KEY: value

processes:
  setup:
    kind: task
    cmd: deno install

  api:
    kind: service
    cmd: deno task dev
    dependsOn:
      setup: success
    ready:
      type: log
      pattern: "listening"
```

Supported dependency conditions:

- `success`: wait for a task to exit with status `0`.
- `ready`: wait for a service to reach readiness.

Supported readiness checks:

- `http`: poll an absolute HTTP(S) URL until it succeeds.
- `log`: watch stdout/stderr for a string or regex.
- `delay`: wait for a configured duration.
- `command`: run an idempotent shell command until it exits with `0`.

Commands are shell strings in V1. On Unix they run through `sh -c`; on Windows
they run through `cmd /C`. This is intentional because the app is aimed at real
development commands that often rely on shell syntax, quotes, pipes, and
multi-line command bodies.

## Architecture

The app is split into a Rust/Tauri backend and a Svelte frontend.

Rust backend:

- `domain`: typed configuration, project, runtime, process, and terminal models.
- `infrastructure`: YAML loading, project persistence, shell strategy, readiness
  helpers, log storage, and PTY management.
- `application`: process orchestration, command spawning, readiness waiting, and
  runtime event emission.
- `tauri_api`: commands and shared app state exposed to the frontend.

Svelte frontend:

- `src/lib/tauri/client.ts`: typed Tauri command wrappers.
- `src/lib/stores/runtime.svelte.ts`: window-scoped runtime store.
- `src/routes/+page.svelte`: main application composition.
- `src/lib/components/*`: project, process, log, terminal, and config UI.
- `src/lib/components/ui/*`: shared UI primitives.
- `src/lib/config/*`: form/YAML conversion and frontend validation.

The backend emits snapshots, process logs, runtime errors, and terminal output
through Tauri events. The frontend stores and renders those events but leaves
process lifecycle decisions to Rust.

## Product Decisions

Current V1 decisions:

- Execution happens inside the Tauri desktop runtime, not in a separate CLI
  runner.
- The app supports launching from a config path, but still opens the desktop UI.
- `devapp.yml` is the canonical project-local config name.
- If no project-local config exists, Devapp can store YAML in the OS app config
  directory.
- YAML editing supports both structured form mode and raw YAML mode.
- Raw YAML mode exists to preserve comments and unsupported future fields.
- The terminal is integrated into the app with xterm; no external terminal
  window is opened for the terminal feature.
- A running project is isolated to its window. Starting another project opens or
  focuses a separate project window.
- The UI is dense and operational, not a marketing page.

## Non-Goals For V1

Devapp is not trying to solve these yet:

- cloud synchronization;
- team sharing;
- a daemon that keeps sessions alive after the app exits;
- remote execution;
- CI/CD orchestration;
- persistent historical log storage;
- a separate terminal-only CLI runner;
- a full process manager replacement.

## Current State

The main runtime, project store, YAML model, readiness checks, integrated
terminal, multi-window project isolation, config editor, and UI primitives are
in place.

The app can be exercised with:

```sh
deno task app examples/deno-runner.yml
```

Regular validation commands:

```sh
deno task check
deno task build
cd src-tauri && cargo test
```

There is a known non-blocking warning from SvelteKit about `tsconfig.json`
extension. The current checks still pass.

## Direction

The next useful work is less about adding new concepts and more about hardening
the existing product:

- complete manual desktop and multi-window validation;
- improve launch/import error reporting in the UI;
- add dirty-state protection in config dialogs;
- decide whether multiple windows for the same project are allowed or whether
  project windows are singleton;
- consider persistent/exportable logs;
- add more focused tests around launch-by-config-path and window isolation;
- continue reducing places where frontend assumptions duplicate backend rules.
