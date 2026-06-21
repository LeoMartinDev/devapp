# AGENTS.md

## Project

Devapp is a local-first Tauri desktop app for running and supervising a
project's local development process graph from a YAML file, usually
`devapp.yml`.

The app replaces ad-hoc terminal tabs and startup notes with a supervised
runtime that knows which commands to start, dependency order, readiness checks,
logs, terminals, and clean shutdown behavior.

V1 is intentionally scoped to one developer running local projects from the
desktop app. Devapp is not a cloud orchestrator, CI runner, remote execution
system, or team dashboard.

## Tooling Rules

- Use Deno as the JavaScript tooling entrypoint.
- Do not run `npm`, `pnpm`, or `yarn` commands directly.
- Do not install dependencies with npm-based package managers.
- Do not add new npm-based workflows. If a workflow is needed, expose it through
  `deno.json` and run it with `deno task`.
- `package.json` may exist because the Svelte/Tauri toolchain expects it, but
  agents should interact with the project through Deno commands.

## Common Commands

Run commands from the project root:

```sh
deno task check
deno task build
deno task app examples/deno-runner.yml
```

Backend tests:

```sh
cd src-tauri && cargo test
```

The project has a known non-blocking SvelteKit warning about `tsconfig.json`
extension. Do not treat that warning alone as a failed validation if the command
exits successfully.

## Architecture

The Rust/Tauri backend is the source of truth for runtime behavior. The
frontend requests actions and renders state, but it does not spawn or supervise
configured processes directly.

Backend layout:

- `src-tauri/src/domain`: typed configuration, project, runtime, process, and
  terminal models.
- `src-tauri/src/infrastructure`: YAML loading, persistence, shell strategy,
  readiness helpers, log storage, and PTY management.
- `src-tauri/src/application`: orchestration, command spawning, readiness
  waiting, and runtime event emission.
- `src-tauri/src/tauri_api`: Tauri commands and shared app state exposed to the
  frontend.

Frontend layout:

- `src/lib/tauri/client.ts`: typed Tauri command wrappers.
- `src/lib/stores/runtime.svelte.ts`: window-scoped runtime store.
- `src/routes/+page.svelte`: main app composition.
- `src/lib/components`: project, process, log, terminal, and config UI.
- `src/lib/components/ui`: shared UI primitives.
- `src/lib/config`: form/YAML conversion and frontend validation.

## Runtime Rules

- A desktop window owns at most one active project session.
- Multiple projects can run at once in independent Tauri windows.
- Runtime events, logs, selected process, selected terminal, and UI errors are
  scoped to the current window.
- A blocking failure stops the whole session.
- Stopping a session stops all configured processes and project terminals for
  that window.
- Logs are stored in memory in V1 and capped at 10,000 frontend lines per
  process.

Process lifecycle states:

```text
pending -> starting -> running -> ready
pending -> starting -> running -> succeeded
pending -> starting -> running -> failed
running/ready/starting -> stopping -> stopped
```

Tasks are expected to exit. Services are expected to stay alive until the user
stops them or the session fails.

## Configuration Rules

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

Commands are shell strings in V1. Preserve that behavior: on Unix they run
through `sh -c`; on Windows they run through `cmd /C`.

## Product Constraints

- `devapp.yml` is the canonical project-local config name.
- YAML editing supports both structured form mode and raw YAML mode.
- Raw YAML mode exists to preserve comments and unsupported future fields.
- The integrated terminal uses xterm inside the app; do not open an external
  terminal for this feature.
- A running project is isolated to its window.
- The UI should stay dense and operational, not marketing-oriented.

## Development Guidance

- Prefer backend rules over duplicated frontend assumptions.
- Keep runtime lifecycle decisions in Rust.
- Keep UI state scoped by window and project session.
- Preserve local-first behavior.
- Avoid expanding V1 into non-goals such as cloud sync, remote execution,
  persistent historical logs, CI/CD orchestration, or daemonized sessions unless
  explicitly requested.
- When changing frontend behavior, keep controls compact and suitable for an
  operational desktop tool.

## Current Hardening Direction

Useful next work includes:

- manual desktop and multi-window validation;
- better launch/import error reporting in the UI;
- dirty-state protection in config dialogs;
- a decision on singleton windows for the same project;
- persistent or exportable logs;
- focused tests around launch-by-config-path and window isolation;
- reducing frontend duplication of backend runtime rules.
