# Configuration

Devapp V1 reads YAML documents named `devapp.yml`. The supported schema version
is `1`.

## Top-Level Fields

```yaml
version: 1
env:
  KEY: value
processes:
  process-name:
    kind: task
    cmd: echo "hello"
```

Fields:

- `version`: required. Must be `1`.
- `env`: optional map of environment variables injected into every configured
  process and command readiness check.
- `processes`: required map keyed by process name. Declaration order is retained
  for display, but scheduling is controlled by dependencies.

Process names must be non-empty and unique inside the YAML map.

## Process Fields

```yaml
processes:
  api:
    kind: service
    cmd: npm run api
    dependsOn:
      setup: success
    ready:
      type: http
      url: http://127.0.0.1:3000/health
```

Fields:

- `kind`: required. Either `task` or `service`.
- `cmd`: required non-empty shell command.
- `dependsOn`: optional map of process name to dependency condition.
- `ready`: optional readiness check. It is meaningful for services.

`task` processes are expected to finish. A successful task exits with status `0`
and moves to `succeeded`. `service` processes are expected to keep running; if a
service exits unexpectedly, the session is stopped.

## Dependency Conditions

`dependsOn` supports two conditions:

```yaml
processes:
  setup:
    kind: task
    cmd: npm install

  api:
    kind: service
    cmd: npm run api
    dependsOn:
      setup: success

  worker:
    kind: service
    cmd: npm run worker
    dependsOn:
      api: ready
```

- `success`: the dependency must be a task that has exited successfully.
- `ready`: the dependency must be a service that has reached `ready`.

Validation rejects dependencies that point to unknown processes and dependency
cycles.

## Command Execution

`cmd` is a shell command string. Devapp does not split the command into argv
itself.

- On Windows, commands run with `cmd /C`.
- On non-Windows platforms, commands run with `sh -c`.

This allows common shell features such as quoted arguments and multi-command
strings, but it also means behavior follows the platform shell.

All configured commands run with the project base directory as the current
working directory and receive the top-level `env` values.

## Readiness Checks

Readiness checks are declared under `ready` and use a tagged `type` field.

### HTTP

```yaml
ready:
  type: http
  url: http://127.0.0.1:3000/health
  intervalMs: 1000
  timeoutMs: 60000
```

`url` must be an absolute `http` or `https` URL. Devapp polls the URL until it
receives a successful HTTP response or the timeout expires.

Optional fields:

- `intervalMs`: polling interval.
- `timeoutMs`: maximum wait time.

### Log

```yaml
ready:
  type: log
  pattern: "Server listening"
  regex: false
  timeoutMs: 60000
```

Devapp watches the process stdout and stderr lines. With `regex: false`, readiness
is reached when a line contains `pattern`. With `regex: true`, `pattern` is
compiled as a regular expression.

Optional fields:

- `regex`: defaults to `false`.
- `timeoutMs`: maximum wait time.

### Delay

```yaml
ready:
  type: delay
  durationMs: 1500
```

Devapp waits for the configured duration after the service starts.

Required fields:

- `durationMs`: fixed wait time in milliseconds.

### Command

```yaml
ready:
  type: command
  cmd: curl -fsS http://127.0.0.1:3000/health
  intervalMs: 1000
  timeoutMs: 60000
```

Devapp repeatedly runs `cmd` in the project base directory until it exits with
status `0` or the timeout expires. Use idempotent commands because they can run
many times.

Optional fields:

- `intervalMs`: polling interval.
- `timeoutMs`: maximum wait time.

## Configuration Sources

The UI can edit either source:

- `ProjectFile`: `devapp.yml` inside the project directory.
- `AppConfigFile`: YAML stored by Devapp in the OS application configuration
  directory for projects that do not keep a local `devapp.yml`.

When edited in the app, the YAML is validated before it is saved.
