# Devapp

**Devapp** is a desktop app that launches and supervises your project's
development commands. You describe your processes in a `devapp.yml` file, and
Devapp starts everything in the right order, checks that each service is ready,
and shuts it all down cleanly when you're done.

No more scattered terminal tabs and sticky notes to remember startup order. One
file, one click, everything runs.

---

## How It Works

1. Create a `devapp.yml` file in your project
2. List the commands you need to run (API, database, worker, etc.)
3. Open Devapp, import your project
4. Click **Start** — Devapp launches everything in order, waits for each
   service to be ready before moving to the next
5. Watch logs for each process, open an integrated terminal, and everything
   shuts down cleanly when you click **Stop**

---

## The `devapp.yml` File

The config file is plain YAML — you can edit it with any text editor, or
directly inside Devapp using the built-in form editor.

Here's a complete example:

```yaml
version: 1
env:
  NODE_ENV: development
  DATABASE_URL: postgres://localhost:5432/myproject

processes:
  setup:
    kind: task
    cmd: npm install && npx prisma migrate dev

  api:
    kind: service
    cmd: npm run dev
    dependsOn:
      setup: success
    ready:
      type: log
      pattern: "listening on port"

  worker:
    kind: service
    cmd: npm run worker
    dependsOn:
      api: ready
    ready:
      type: http
      url: http://localhost:3000/health
```

### Overall Structure

The file has three main sections:

| Section | Required | Description |
|---------|----------|-------------|
| `version` | Yes | Always `1` for now |
| `env` | No | Environment variables shared by all processes |
| `processes` | Yes | All your processes, keyed by name |

### Processes

Each process is a command that Devapp will run. You give it a name (used as a
reference) and describe its behavior.

**Required fields:**

- **`kind`** : what type of process. Two options:
  - `task` — a command that finishes on its own (e.g. `npm install`, a migration
    script)
  - `service` — a long-running process (e.g. an API server, a worker)
- **`cmd`** : the shell command to run. Exactly what you'd type in a terminal
  (e.g. `npm run dev`, `python -m http.server`).

**Optional fields:**

- **`dependsOn`** : processes that must be ready before this one starts
- **`ready`** : how to check that a service is ready

### Multi-line Commands

Use the `|` (pipe) character in YAML to write commands that span multiple lines:

```yaml
api:
  kind: service
  cmd: |
    echo "starting API..."
    cd ./packages/api
    npm run dev
```

Everything indented below `|` becomes a single string with line breaks, which
the shell executes as a mini-script. To collapse line breaks into spaces
instead, use `>` instead of `|`.

### Tasks vs Services

This is the most important concept in the config file.

**A `task`** is a command that does something and then exits on its own:
installing dependencies, compiling code, running migrations, cleaning up files.
Devapp waits for it to finish. If it succeeds (exit code 0), it moves on. If it
fails, everything stops.

**A `service`** is a command that keeps running: a web server, an API, a
worker, a message listener. Devapp starts it and then monitors whether it's
ready to accept traffic. If a service stops unexpectedly, everything stops.

### Dependencies (`dependsOn`)

You tell Devapp that a process should only start once another one is ready,
using `dependsOn`:

```yaml
api:
  kind: service
  cmd: npm run dev
  dependsOn:
    setup: success    # wait for "setup" to succeed
    database: ready   # wait for "database" to be ready
```

There are two dependency conditions:

| Condition | Meaning |
|-----------|---------|
| `success` | Wait for a `task` to exit successfully (exit code 0) |
| `ready` | Wait for a `service` to reach readiness (see next section) |

Dependencies form a **graph** — Devapp automatically figures out the startup
order. If you create a cycle (A depends on B, B depends on A), Devapp reports
it as an error.

### Readiness Checks (`ready`)

For a `service`, you often need to wait until it's *actually* ready before
starting the next process. A web server might take a few seconds to start even
after its command is launched.

Devapp provides 4 ways to check that a service is ready:

---

#### 1. Log — detect a message in the logs

```yaml
ready:
  type: log
  pattern: "listening on port 3000"
  timeoutMs: 30000   # optional: max wait time (30 seconds here)
```

Devapp watches the process output. As soon as a line contains
`"listening on port 3000"`, the service is considered ready.

You can also use a **regular expression**:

```yaml
ready:
  type: log
  pattern: "listening on (port )?\\d+"
  regex: true
  timeoutMs: 30000
```

---

#### 2. HTTP — poll a URL

```yaml
ready:
  type: http
  url: http://localhost:3000/health
  intervalMs: 1000   # optional: interval between attempts (1 second)
  timeoutMs: 60000   # optional: max wait time (60 seconds)
```

Devapp calls the URL regularly. As soon as it gets a 2xx response (200, 201,
etc.), the service is ready. Ideal if your service exposes a `/health` or
`/ready` endpoint.

---

#### 3. Delay — wait a fixed amount of time

```yaml
ready:
  type: delay
  durationMs: 2000   # wait 2 seconds
```

The simplest method: wait for a fixed number of milliseconds after the service
starts. Useful for services that start very quickly, or when you don't have a
better indicator.

---

#### 4. Command — run a check command

```yaml
ready:
  type: command
  cmd: curl -sS http://localhost:3000/health
  intervalMs: 1000
  timeoutMs: 30000
```

Devapp runs the command repeatedly. As soon as it exits with code `0`
(success), the service is ready. Useful for checks more complex than a simple
HTTP call.

**Heads up:** the command must be safe to run multiple times (it must be
*idempotent*). A read-only `curl` check is perfect; a command that modifies
data is not.

---

### Environment Variables

The `env` section defines variables shared by **all** your processes:

```yaml
env:
  NODE_ENV: development
  DATABASE_URL: postgres://localhost:5432/myproject
  API_KEY: abc123
```

Each process receives these variables on top of whatever is already in your
environment. Readiness check commands (`type: command`) also receive them.

---

## Real-World Examples

### A Typical Web Project

```yaml
version: 1

processes:
  install:
    kind: task
    cmd: npm install

  migrate:
    kind: task
    cmd: npx prisma migrate dev
    dependsOn:
      install: success

  api:
    kind: service
    cmd: npm run dev
    dependsOn:
      migrate: success
    ready:
      type: log
      pattern: "Server is listening"
      timeoutMs: 15000

  frontend:
    kind: service
    cmd: npm run dev --workspace=frontend
    dependsOn:
      api: ready
    ready:
      type: http
      url: http://localhost:5173
```

This describes a project with:
1. Installing dependencies
2. Running database migrations (after install)
3. Starting the API (after migrations)
4. Starting the frontend (once the API is ready)

One click and everything starts in the right order.

### A Python Project with Redis

```yaml
version: 1
env:
  PYTHONUNBUFFERED: "1"

processes:
  install:
    kind: task
    cmd: pip install -r requirements.txt

  redis:
    kind: service
    cmd: redis-server
    ready:
      type: command
      cmd: redis-cli ping
      intervalMs: 500
      timeoutMs: 10000

  api:
    kind: service
    cmd: uvicorn main:app --reload --port 8000
    dependsOn:
      install: success
      redis: ready
    ready:
      type: http
      url: http://localhost:8000/health
      timeoutMs: 30000

  worker:
    kind: service
    cmd: celery -A tasks worker --loglevel=info
    dependsOn:
      redis: ready
    ready:
      type: log
      pattern: "celery@.* ready"
      regex: true
```

---

## Integrated Terminal

Beyond configured processes, Devapp includes an integrated terminal (xterm)
opened in your project directory. You can run one-off commands (tests, linting,
scripts) without leaving the app. Each project window has its own terminal.

## Editing the Configuration

Devapp offers two modes for editing `devapp.yml`:

- **Form mode** — a structured interface with fields for each option. Great for
  exploring what's available without knowing YAML.
- **Raw YAML mode** — the built-in text editor. Preserves your comments and
  lets you write YAML directly.

Either way, Devapp validates the config before saving it.

## Launching Devapp

```bash
# Launch with an existing config file
deno task app examples/deno-runner.yml

# Launch without a project (you can import one from the UI)
deno task app
```

Devapp opens a desktop window. From there you can:
- Import a project directory that contains a `devapp.yml`
- Create a config from scratch using the built-in editor
- Start/stop your project with one click

## Tips

- **Name your processes clearly** (`api`, `worker`, `frontend`) — these names
  appear in the UI.
- **Always use a `ready` check for services** — without one, Devapp immediately
  moves on to the next process, which is rarely what you want.
- **Prefer `type: log`** when your service prints a clear startup message: it's
  simple and reliable.
- **Prefer `type: http`** when your service has a health endpoint: it's the
  most robust check.
- **`timeoutMs` defaults to 60 seconds.** If your service takes longer to
  start, increase this value.
- **Failed tasks stop everything** — if `npm install` fails, Devapp won't try
  to start anything else. You'll see the error in the logs.
- **Multi-project** — you can open multiple Devapp windows, each with its own
  project. Handy when working on a frontend and API in separate folders.
