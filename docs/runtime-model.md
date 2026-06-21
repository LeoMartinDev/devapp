# Runtime Model

Devapp runs one project session per desktop window. A session is created from a
registered project and one loaded YAML configuration. Starting another project
from a window that already owns an active session opens or focuses an independent
project window instead of replacing the current session.

## Process States

Processes use these states:

- `pending`: declared but not started yet.
- `blocked`: reserved for dependency-blocked runtime reporting.
- `starting`: Devapp is spawning the command.
- `running`: the command has started.
- `ready`: a service has passed its readiness check.
- `succeeded`: a task exited with status `0`.
- `failed`: the process failed, exited unexpectedly, or readiness failed.
- `stopping`: Devapp requested termination.
- `stopped`: the process stopped because the session or process was stopped.

## Startup Flow

At session start, every declared process receives a runtime id and starts as
`pending`. Devapp scans the graph and starts every process whose dependencies are
satisfied.

Dependency conditions:

- `success`: satisfied when the dependency reaches `succeeded`.
- `ready`: satisfied when the dependency reaches `ready`.

When a process becomes runnable, it moves through:

```text
pending -> starting -> running
```

A task then exits:

```text
running -> succeeded
running -> failed
```

A service without a readiness check is marked ready after it starts:

```text
running -> ready
```

A service with a readiness check remains `running` until the check succeeds:

```text
running -> ready
running -> failed
```

After a task reaches `succeeded` or a service reaches `ready`, Devapp scans the
graph again and starts newly unblocked dependents.

## Failure Behavior

V1 treats blocking runtime failures as session failures. Devapp stops the whole
session when:

- a task exits with a non-zero status;
- a service exits unexpectedly;
- a readiness check times out or errors;
- process spawning or process waiting fails.

On session failure, Devapp marks the session as stopped, requests termination for
running processes, emits the latest snapshot, and reports a runtime error event
to the UI.

## Manual Stop

When the user stops the project, Devapp marks the session as stopping and asks
all running, starting, or ready processes to terminate. Pending or blocked
processes are marked `stopped` during an explicit stop.

The desktop API also stops integrated terminals associated with the project when
the project session is stopped.

## Restarting A Process

The backend exposes process restart by stopping the named process, resetting its
runtime snapshot to `pending`, and scanning the graph again. Dependents are not
modeled as a separate restart tree in V1.

## Logs And Terminals

Stdout and stderr are streamed as process log events and stored in memory for the
active session. Logs are session-scoped and not persisted as V1 project history.

Integrated terminals are separate from configured processes. They run in the
project base directory and are stopped when the project session is stopped.

## Frontend Review Remediation

The frontend remediation keeps the runtime UI dense and desktop-focused while
removing several fragile behaviors from the first implementation.

- The terminal remains integrated in the Svelte view through xterm. Opening a
  terminal attaches the existing xterm instance to the right pane and routes
  input, resize, output, and close through the Tauri terminal commands.
- Runtime state is scoped to the current project window. If a different project
  is launched while the current window has an active session, the frontend calls
  the backend window command instead of replacing local logs, terminals, or
  process snapshots.
- Project configuration has two edit paths. The form mode covers the supported
  V1 schema with inline validation; Raw YAML validates and saves the document
  exactly as written so comments and future fields are not discarded by the form.
- Dialogs use shared UI primitives with ARIA metadata, Escape handling, basic
  focus trapping, overlay close control, and restored focus after close.
- Process actions are status-aware. Global busy state disables process actions,
  `Stop` is unavailable for terminal statuses, and `Restart` is unavailable for
  pending, blocked, starting, or stopping processes.
- Logs are capped in memory per process at 10,000 lines. The UI reports how many
  lines are visible, how many older lines were hidden, supports a paused view,
  and keeps collecting new log events in the store while paused.
- The main shell and configuration editor use responsive grids. On small
  viewports, the shell stacks navigation and content, and the configuration
  process list becomes a horizontal scroll area with one-column forms.
