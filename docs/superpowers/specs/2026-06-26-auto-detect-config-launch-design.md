# Auto-Detect Config on Launch

## Summary

When devapp is launched without a config path argument, the backend walks
up from the current working directory to the filesystem root looking for
a `devapp.yml`. If found, it behaves exactly like a CLI argument launch
(import, lock, auto-start). If not found, the current WelcomeScreen
behaviour is preserved. If found but invalid, an error banner is shown
and the WelcomeScreen is displayed.

## Motivation

- Users often launch the app from their project directory expecting it
  to "just work", without needing to type the config path.
- Walking up parent directories matches `git` behaviour and catches
  cases where the user is in a subdirectory of their project.
- Auto-detection should be a transparent convenience, not a different
  mode — it reuses the exact same lock + auto-start flow as CLI args.

## Design

### 1. Auto-detection function

New function in `src-tauri/src/infrastructure/config_loader.rs`:

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

- Uses `std::env::current_dir()` as the starting point.
- Walks up with `Path::pop()` until reaching the filesystem root.
- Stops at the first `devapp.yml` found (closest to cwd).
- Makes the existing `src-tauri` special case in `resolve_launch_config_path`
  redundant: if cwd is `src-tauri/`, the walk-up naturally finds the
  workspace root's `devapp.yml`.

### 2. Setup integration

In `src-tauri/src/lib.rs`, the `setup` closure currently only checks CLI
arguments:

```rust
if let Some(config_path) = launch_config_path() { ... }
```

This becomes:

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
            let mut ui_error = state.ui_error.lock().await;
            *ui_error = Some(format!(
                "Failed to load auto-detected config {}: {}",
                config_path.display(), error
            ));
        }
    }
}
```

Key points:
- `launch_config_path()` (CLI arg) takes priority via `or_else`.
- On import success, `launch_project_id` is set → frontend receives
  `launchLocked=true` and auto-starts the session.
- On import failure (bad YAML, schema error), the error is stored in
  `ui_error` and `launch_project_id` stays `None` → frontend shows the
  error banner on top of the WelcomeScreen.
- `launchLocked` remains `false` on error, so the user can navigate freely.

### 3. Imports

`src-tauri/src/lib.rs` gains one import:

```rust
use crate::infrastructure::config_loader::find_config_in_cwd_or_parents;
```

## Behaviour Matrix

| CLI arg | cwd config | Valid? | Result |
|---------|-----------|--------|--------|
| Yes     | Any       | Any    | CLI arg used (current behaviour, unchanged) |
| No      | Found     | Yes    | Auto-detected, project loaded, locked, auto-started |
| No      | Found     | No     | Error banner + WelcomeScreen |
| No      | Not found | N/A    | WelcomeScreen (current behaviour, unchanged) |

## Files Changed

| File | Change |
|------|--------|
| `src-tauri/src/infrastructure/config_loader.rs` | Add `find_config_in_cwd_or_parents()` |
| `src-tauri/src/lib.rs` | Integrate auto-detection in setup; add import |

## Non-Goals

- No frontend changes required. The existing `WelcomeScreen`, error
  banner, and locked-launch flows handle all cases.
- No search for alternate filenames (`.devapp.yml`, `devapp.yaml`, etc.).
  Only `devapp.yml` is checked.
- No recursive directory search. Only linear parent walk-up.
- No configurable search boundaries. Always walks to filesystem root.

## Testing

- Unit test: `find_config_in_cwd_or_parents()` with temp directories
  containing/not containing `devapp.yml` at various depths.
- Integration test: launch the app from a directory with a valid
  `devapp.yml` and verify the project is loaded and auto-started.
- Integration test: launch from a directory with an invalid `devapp.yml`
  and verify the error banner + WelcomeScreen.
- Integration test: launch from a directory with no config and verify
  the WelcomeScreen.
