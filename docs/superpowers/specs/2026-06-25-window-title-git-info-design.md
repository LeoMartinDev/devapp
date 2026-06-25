# Window Title with Project Path and Git Info

**Date**: 2025-06-25

## Summary

Each devapp window displays the project path and git context (branch or worktree)
in the native window title bar, keeping the user oriented about which project and
branch they are working on.

## Motivation

Currently, all devapp windows show `devapp` or `devapp - {project.name}` in the
title bar. When multiple windows are open on different branches or worktrees, the
user cannot distinguish them at a glance. The title bar is the most visible,
always-present place to surface this information.

## Title Format

```
{parent}/{project} — {git_context} — devapp
```

Examples:

| Scenario | Title |
|---|---|
| Normal branch | `dev/devapp — main — devapp` |
| Linked worktree | `dev/devapp — fix-login — devapp` |
| No git repo | `dev/devapp — devapp` |
| Project at `$HOME` root (no parent) | `devapp — main — devapp` |

**Rules:**
- `{parent}` is the immediate parent directory name relative to `$HOME` (e.g.,
  `dev` for `~/dev/devapp`). Omitted if the project is directly in `$HOME`.
- `{project}` is `project.name` (which defaults to the directory basename).
- `{git_context}` is the current branch name, or the worktree name if the
  project directory is a linked git worktree. Omitted entirely when the
  directory is not inside a git repository.
- Sections are separated by em dashes (` — `).
- The app name `devapp` is always the last segment.

## Architecture

### Backend: Git Detection Module

**New file**: `src-tauri/src/infrastructure/git_info.rs`

**Crate dependency**: `gix` (pure Rust, no system libgit2 required).

**Data type**:
```rust
pub struct GitInfo {
    pub repo_name: Option<String>,
    pub branch: Option<String>,
    pub worktree: Option<String>,
    pub is_worktree: bool,
}
```

**Detection logic** (`detect_git_info(base_dir: &Path) -> GitInfo`):
1. Open the repository from `base_dir` (gix auto-discovers the `.git` directory
   by walking up).
2. `repo_name` is the basename of the repository root directory.
3. `branch` is the short name of HEAD (e.g., `main`). `None` if HEAD is detached.
4. If `.git` is a file (linked worktree), extract the worktree name from the
   file's `gitdir:` path: the last path component before `.git` is the worktree
   name. Set `is_worktree = true`.
5. All fields are `Option` — if no git repository is found, everything is
   `None`.

**Tauri command**:
```rust
#[tauri::command]
fn get_git_info(base_dir: String) -> GitInfo {
    git_info::detect_git_info(Path::new(&base_dir))
}
```

Registered in the Tauri command list alongside existing commands.

### Frontend: Runtime Store

**Additions to `RuntimeStore`** (`src/lib/stores/runtime.svelte.ts`):

```typescript
gitInfo = $state<GitInfo | null>(null);
#gitInfoTimer: ReturnType<typeof setInterval> | null = null;
```

- `#fetchGitInfo()` — calls `getGitInfo(baseDir)` and assigns `this.gitInfo`.
  No-op if `baseDir` is unavailable.
- `#startGitPolling()` — calls `#fetchGitInfo()` immediately, then sets a 60s
  interval. Called when a project starts.
- `#stopGitPolling()` — clears the interval, sets `this.gitInfo = null`.
  Called when a project stops.
- `onWindowFocus` — also calls `#fetchGitInfo()` on window focus events to
  catch branch switches made outside devapp.

**Title derivation** (in `+page.svelte` or as a `$derived` in the store):

```typescript
function formatWindowTitle(
  project: ProjectRecord | null,
  gitInfo: GitInfo | null,
): string {
  if (!project) return "devapp";

  const relPath = relativeToHome(project.baseDir); // e.g. "dev/devapp"
  let context = "";
  if (gitInfo?.worktree) {
    context = ` — ${gitInfo.worktree}`;
  } else if (gitInfo?.branch) {
    context = ` — ${gitInfo.branch}`;
  }

  return `${relPath}${context} — devapp`;
}
```

`relativeToHome` strips the `$HOME` prefix and returns the remaining path
(e.g., `~/dev/devapp` → `dev/devapp`, `~/devapp` → `devapp`).

### Frontend: Window Title Update

**New wrapper in `src/lib/tauri/client.ts`**:
```typescript
import { getCurrentWindow } from "@tauri-apps/api/window";

export function setWindowTitle(title: string): Promise<void> {
  return getCurrentWindow().setTitle(title);
}
```

**Reactive effect in `+page.svelte`**:
```svelte
$effect(() => {
  const title = formatWindowTitle(store.project, store.gitInfo);
  setWindowTitle(title);
});
```

Also updates `document.title` as a fallback for contexts where the native title
isn't accessible (e.g., browser dev mode).

### Types

**New frontend type** in `src/lib/types.ts`:
```typescript
export interface GitInfo {
  repo_name: string | null;
  branch: string | null;
  worktree: string | null;
  is_worktree: boolean;
}
```

### Updated `$effect` in `+page.svelte`

Replace the static `<title>devapp</title>` in `<svelte:head>` with the dynamic
title (or remove it entirely since the native title bar takes precedence).

## Data Flow

```
[gix crate] → git_info.rs → get_git_info command
                                    ↑
                          Tauri invoke (IPC)
                                    ↑
              RuntimeStore.#fetchGitInfo()  ←── focus event
              RuntimeStore.#startGitPolling() ←── 60s interval
                                    ↓
                         formatWindowTitle()
                                    ↓
                         setWindowTitle()  →  native title bar
```

## Behavior Details

- **Session start**: `gitInfo` is fetched immediately and the title updates.
- **During session**: re-fetched every 60s and on window focus.
- **Session stop**: polling stops, `gitInfo` resets to `null`, title reverts
  to `devapp`.
- **No git repo**: `gitInfo` has all fields `null`; title shows path only.
- **Detached HEAD**: `branch` is `null`; title shows path only (no context).
- **Tauri window creation**: the initial title from
  `open_project_window` (`"devapp - {project.name}"`) is immediately
  overwritten by the frontend `$effect` once the page loads and the first
  `get_git_info` call returns. This is acceptable as it happens within
  milliseconds.

## Scope

### In scope
- Git detection via `gix` in the Rust backend.
- `get_git_info` Tauri command.
- Frontend polling, focus-based refresh, title formatting.
- Native window title update via `@tauri-apps/api/window`.

### Out of scope
- Displaying git info anywhere else in the UI (sidebar, content header).
  Existing `baseDir` display in the content header fallback is left unchanged.
- Dirty/clean or ahead/behind indicators.
- Git remote URL or upstream tracking info.
- Persistent or exportable git metadata.

## Risks and Mitigations

- **`gix` compile time**: `gix` is a large crate. Use the `gix` meta-crate
  with minimal feature flags (`repository`, `HEAD`).
- **`git` not in PATH**: Not applicable — `gix` is a library, not a CLI
  dependency.
- **Performance**: `gix` repository open is cheap (reads `.git` metadata).
  Polling every 60s is negligible.
- **Stale title after external branch switch**: 60s polling + focus refresh
  covers most cases. Switching branches inside devapp's own terminal is
  covered by the existing polling timer.

## Testing Strategy

- **Backend**: unit tests for `detect_git_info` against temp directories with
  real git repos, worktrees, and non-git directories.
- **Frontend**: manual validation with multiple windows on different branches
  and worktrees.
