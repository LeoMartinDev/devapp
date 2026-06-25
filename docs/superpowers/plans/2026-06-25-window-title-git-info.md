# Window Title with Project Path and Git Info — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Display project path and git context (branch/worktree) in the native window title bar, refreshing every 60s and on window focus.

**Architecture:** New Rust module `git_info.rs` uses `gix` to detect git metadata. A Tauri command `get_git_info` exposes it. The frontend `RuntimeStore` polls it every 60s and on focus, derives a title string (`dev/devapp — main — devapp`), and sets the native window title via `getCurrentWindow().setTitle()`.

**Tech Stack:** gix (Rust git crate), Tauri v2 `getCurrentWindow` API, Svelte 5 `$effect`

## Global Constraints

- Use `gix` with minimal features (`repository`, `HEAD`).
- Title format: `{parent}/{project} — {git_context} — devapp` (context = branch or worktree name, omitted if no git).
- Polling interval: 60 seconds.
- No git-related fields in `ProjectRecord` or `RunSessionSnapshot` — git info is always live-queried.

---

### Task 1: Add `gix` dependency and `GitInfo` struct

**Files:**
- Modify: `src-tauri/Cargo.toml:28-29` (add after `reqwest`)
- Create: `src-tauri/src/infrastructure/git_info.rs`
- Modify: `src-tauri/src/infrastructure/mod.rs:6` (register module)

**Interfaces:**
- Produces: `GitInfo { repo_name: Option<String>, branch: Option<String>, worktree: Option<String>, is_worktree: bool }`
- Produces: `pub fn detect_git_info(base_dir: &std::path::Path) -> GitInfo`

- [ ] **Step 1: Add gix to Cargo.toml**

```toml
gix = { version = "0.70", default-features = false, features = ["revision"] }
```

Place it after the `reqwest` dependency line (line 28).

- [ ] **Step 2: Create `git_info.rs` with struct and detection function**

New file `src-tauri/src/infrastructure/git_info.rs`:

```rust
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfo {
    pub repo_name: Option<String>,
    pub branch: Option<String>,
    pub worktree: Option<String>,
    pub is_worktree: bool,
    pub display_path: Option<String>,
}

pub fn detect_git_info(base_dir: &Path) -> GitInfo {
    let display_path = home_relative_path(base_dir);

    let repo = match gix::open(base_dir) {
        Ok(repo) => repo,
        Err(_) => {
            return GitInfo {
                display_path,
                ..Default::default()
            };
        }
    };

    let repo_name = repo
        .work_dir()
        .and_then(|work_dir| work_dir.file_name())
        .map(|name| name.to_string_lossy().into_owned());

    let branch = repo
        .head_name()
        .ok()
        .flatten()
        .map(|head| head.shorten().to_string());

    // Detect linked worktree: .git is a file containing "gitdir: <path>"
    let mut worktree: Option<String> = None;
    let mut is_worktree = false;
    let dot_git = base_dir.join(".git");
    if dot_git.is_file() {
        if let Ok(contents) = std::fs::read_to_string(&dot_git) {
            if let Some(gitdir_line) = contents.strip_prefix("gitdir: ") {
                let gitdir_path = gitdir_line.trim();
                // Extract worktree name: last component before /.git/worktrees/<name>
                // The gitdir path format is typically:
                //   <repo>/.git/worktrees/<name>
                let path = Path::new(gitdir_path);
                if let Some(worktrees_idx) = path
                    .components()
                    .position(|c| c.as_os_str() == "worktrees")
                {
                    if let Some(name) = path.components().nth(worktrees_idx + 1) {
                        worktree = Some(name.as_os_str().to_string_lossy().into_owned());
                        is_worktree = true;
                    }
                }
            }
        }
    }

    GitInfo {
        repo_name,
        branch,
        worktree,
        is_worktree,
        display_path,
    }
}

fn home_relative_path(absolute: &Path) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let home_path = Path::new(&home);
    let stripped = absolute.strip_prefix(home_path).ok()?;
    Some(stripped.to_string_lossy().into_owned())
}
```

- [ ] **Step 3: Register module in infrastructure/mod.rs**

In `src-tauri/src/infrastructure/mod.rs`, add after line 6 (`pub mod shell;`):

```rust
pub mod git_info;
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`
Workdir: `src-tauri`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/infrastructure/git_info.rs src-tauri/src/infrastructure/mod.rs
git commit -m "feat: add git_info module with gix-based detection"
```

---

### Task 2: Add `get_git_info` Tauri command

**Files:**
- Modify: `src-tauri/src/tauri_api/commands.rs:340-341` (add `get_git_info` function)
- Modify: `src-tauri/src/lib.rs:34-53` (register command in handler)

**Interfaces:**
- Consumes: `git_info::detect_git_info`, `git_info::GitInfo`
- Produces: `get_git_info(base_dir: String) -> Result<GitInfo, String>`

- [ ] **Step 1: Add `get_git_info` command in commands.rs**

In `src-tauri/src/tauri_api/commands.rs`, add the import at the top (after line 16, the existing `use crate::` block ends):

```rust
use crate::infrastructure::git_info::{self, GitInfo};
```

Then add the command function before `open_project_window` (line 340) or at the end of the file (before line 369):

```rust
#[tauri::command]
pub fn get_git_info(base_dir: String) -> Result<GitInfo, String> {
    Ok(git_info::detect_git_info(std::path::Path::new(&base_dir)))
}
```

- [ ] **Step 2: Register command in lib.rs**

In `src-tauri/src/lib.rs`, add to the `invoke_handler` block after line 52 (`tauri_api::commands::open_project_window`):

```rust
            tauri_api::commands::get_git_info
```

Don't forget the trailing comma on the previous line.

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Workdir: `src-tauri`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tauri_api/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add get_git_info Tauri command"
```

---

### Task 3: Add `GitInfo` frontend type and client wrappers

**Files:**
- Modify: `src/lib/types.ts:152` (add after line 152, end of file)
- Modify: `src/lib/tauri/client.ts:124` (add after imports, before export)

**Interfaces:**
- Produces: `GitInfo` TypeScript type matching Rust struct
- Produces: `getGitInfo(baseDir: string): Promise<GitInfo>`
- Produces: `setWindowTitle(title: string): Promise<void>`

- [ ] **Step 1: Add GitInfo type to types.ts**

In `src/lib/types.ts`, add after line 152 (end of file):

```typescript
export interface GitInfo {
  repoName: string | null;
  branch: string | null;
  worktree: string | null;
  isWorktree: boolean;
  displayPath: string | null;
}
```

- [ ] **Step 2: Add getGitInfo wrapper to client.ts**

In `src/lib/tauri/client.ts`, add `GitInfo` to the type import on line 9:

```typescript
  GitInfo,
```

Then add the function after the existing client functions (before the end of file, after line 124):

```typescript
export async function getGitInfo(baseDir: string): Promise<GitInfo> {
  return invoke<GitInfo>("get_git_info", { baseDir });
}
```

- [ ] **Step 3: Add setWindowTitle wrapper to client.ts**

In the same file, add the import at the top:

```typescript
import { getCurrentWindow } from "@tauri-apps/api/window";
```

Then add after the `getGitInfo` function:

```typescript
export function setWindowTitle(title: string): Promise<void> {
  return getCurrentWindow().setTitle(title);
}
```

- [ ] **Step 4: Verify TypeScript compilation**

Run: `deno task check`

- [ ] **Step 5: Commit**

```bash
git add src/lib/types.ts src/lib/tauri/client.ts
git commit -m "feat: add GitInfo type and client wrappers for get_git_info, setWindowTitle"
```

---

### Task 4: Add git polling and title formatting to RuntimeStore

**Files:**
- Modify: `src/lib/stores/runtime.svelte.ts:1-23` (imports)
- Modify: `src/lib/stores/runtime.svelte.ts:46-59` (state fields)
- Modify: `src/lib/stores/runtime.svelte.ts:172-189` (startCurrentProject)
- Modify: `src/lib/stores/runtime.svelte.ts:191-202` (stopCurrentProject)
- Modify: `src/lib/stores/runtime.svelte.ts:466-468` (end of class, before runtimeStore export)

**Interfaces:**
- Consumes: `getGitInfo` from client.ts, `GitInfo` from types.ts
- Produces: `runtimeStore.gitInfo: GitInfo | null`
- Produces: `runtimeStore.formatWindowTitle(): string`

- [ ] **Step 1: Add imports**

In `src/lib/stores/runtime.svelte.ts`, add `getGitInfo` and `setWindowTitle` to the import from `$lib/tauri/client` (line 5-22). Add after `getSessionSnapshot`:

```typescript
  getGitInfo,
  setWindowTitle,
```

Add `GitInfo` to the type import from `$lib/types` (line 25):

```typescript
  GitInfo,
```

- [ ] **Step 2: Add state fields**

In the class body, add after line 58 (`busy = $state(false);`):

```typescript
  gitInfo = $state<GitInfo | null>(null);
  #gitInfoTimer: ReturnType<typeof setInterval> | null = null;
```

- [ ] **Step 3: Add fetchGitInfo private method**

Add after line 59 (after the two new state fields):

```typescript
  async #fetchGitInfo() {
    const dir = this.project?.baseDir ?? this.session?.baseDir;
    if (!dir) return;
    try {
      this.gitInfo = await getGitInfo(dir);
    } catch {
      this.gitInfo = null;
    }
  }
```

- [ ] **Step 4: Add formatWindowTitle getter**

Add after the `#fetchGitInfo` method:

```typescript
  get windowTitle(): string {
    const proj = this.project;
    if (!proj) return "devapp";

    const relPath = this.gitInfo?.displayPath ?? proj.name;

    let context = "";
    if (this.gitInfo?.worktree) {
      context = ` — ${this.gitInfo.worktree}`;
    } else if (this.gitInfo?.branch) {
      context = ` — ${this.gitInfo.branch}`;
    }

    return `${relPath}${context} — devapp`;
  }
```

- [ ] **Step 5: Start polling on session start**

In `startCurrentProject()` (line 172), after `this.syncProcessSelection();` (line 182, inside the try block, before the `} catch`), add:

```typescript
      this.#startGitPolling();
```

Then add the private method:

```typescript
  #startGitPolling() {
    this.#stopGitPolling();
    void this.#fetchGitInfo();
    this.#gitInfoTimer = setInterval(() => {
      void this.#fetchGitInfo();
    }, 60_000);
  }
```

- [ ] **Step 6: Stop polling on session stop**

In `stopCurrentProject()` (line 191), after `await stopProject();` (line 193, inside the try block), add:

```typescript
      this.#stopGitPolling();
```

Then add the private method:

```typescript
  #stopGitPolling() {
    if (this.#gitInfoTimer !== null) {
      clearInterval(this.#gitInfoTimer);
      this.#gitInfoTimer = null;
    }
    this.gitInfo = null;
  }
```

- [ ] **Step 7: Refresh on window focus**

In `init()` (line 63), add a window focus listener. After `await this.#applyLaunchParams();` (line 73), add:

```typescript
    window.addEventListener("focus", () => {
      void this.#fetchGitInfo();
    });
```

- [ ] **Step 8: Verify TypeScript compilation**

Run: `deno task check`

Note: the project has a known non-blocking SvelteKit warning about `tsconfig.json` extension. That warning alone is not a failure.

- [ ] **Step 9: Commit**

```bash
git add src/lib/stores/runtime.svelte.ts
git commit -m "feat: add git polling and window title formatting to RuntimeStore"
```

---

### Task 5: Wire up reactive title update in +page.svelte

**Files:**
- Modify: `src/routes/+page.svelte:1-17` (imports)
- Modify: `src/routes/+page.svelte:140-142` (svelte:head)
- Modify: `src/routes/+page.svelte:109-114` (onMount)

**Interfaces:**
- Consumes: `runtimeStore.windowTitle`, `setWindowTitle` from client.ts

- [ ] **Step 1: Add import for setWindowTitle**

In `src/routes/+page.svelte`, add to the import from `$lib/tauri/client` or add a new import:

After line 14 (`import { runtimeStore } from "$lib/stores/runtime.svelte";`), add:

```typescript
  import { setWindowTitle } from "$lib/tauri/client";
```

- [ ] **Step 2: Replace static title with reactive $effect**

Replace the `<svelte:head>` block (lines 140-142):

Current:
```svelte
<svelte:head>
  <title>devapp</title>
</svelte:head>
```

Replace with:
```svelte
<svelte:head>
  <title>{runtimeStore.windowTitle}</title>
</svelte:head>
```

- [ ] **Step 3: Add $effect for native window title update**

Add after the `$derived` declarations (after line 28, after `const selection = $derived(runtimeStore.selection);`):

```typescript
  $effect(() => {
    const title = runtimeStore.windowTitle;
    document.title = title;
    setWindowTitle(title);
  });
```

- [ ] **Step 4: Also trigger fetch on onMount (focus event already set up in Task 4)**

No additional change needed — the focus listener is in Task 4's `init()`.

- [ ] **Step 5: Update initial window title in open_project_window**

This is a nice-to-have: change the initial title in `src-tauri/src/tauri_api/commands.rs` line 363 from:

```rust
        .title(format!("devapp - {}", project.name))
```

To:

```rust
        .title(format!("{} — devapp", project.name))
```

This makes the initial flash before the frontend loads more consistent with the final format.

- [ ] **Step 6: Verify build**

Run: `deno task build`

- [ ] **Step 7: Commit**

```bash
git add src/routes/+page.svelte src-tauri/src/tauri_api/commands.rs
git commit -m "feat: wire up reactive window title with git info"
```

---

### Task 6: Backend unit tests for git_info

**Files:**
- Create: `src-tauri/src/infrastructure/git_info.rs` (add `#[cfg(test)]` module)

**Interfaces:**
- Tests: `detect_git_info` against real git repos, worktrees, non-git dirs

- [ ] **Step 1: Add tests module to git_info.rs**

Append to `src-tauri/src/infrastructure/git_info.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_git_repo() -> (TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        Command::new("git")
            .args(["init"])
            .current_dir(&path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&path)
            .output()
            .unwrap();
        // Create an initial commit so HEAD resolves
        std::fs::write(path.join("readme.md"), "# test").unwrap();
        Command::new("git")
            .args(["add", "readme.md"])
            .current_dir(&path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&path)
            .output()
            .unwrap();
        (dir, path)
    }

    #[test]
    fn non_git_directory_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let info = detect_git_info(dir.path());
        assert!(info.repo_name.is_none());
        assert!(info.branch.is_none());
        assert!(info.worktree.is_none());
        assert!(!info.is_worktree);
    }

    #[test]
    fn normal_repo_detects_branch_and_name() {
        let (_dir, path) = setup_git_repo();
        let info = detect_git_info(&path);
        assert!(info.repo_name.is_some());
        assert!(info.branch.is_some());
        // Default branch may be "main" or "master"
        let branch = info.branch.unwrap();
        assert!(branch == "main" || branch == "master" || branch.starts_with("refs/heads/"));
        assert!(!info.is_worktree);
        assert!(info.worktree.is_none());
    }

    #[test]
    fn subdirectory_of_repo_detects_git() {
        let (_dir, path) = setup_git_repo();
        let subdir = path.join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        let info = detect_git_info(&subdir);
        assert!(info.repo_name.is_some());
        assert!(info.branch.is_some());
    }

    #[test]
    fn worktree_is_detected() {
        let (_dir, path) = setup_git_repo();
        // Create a linked worktree
        let worktree_path = path.parent().unwrap().join("worktree_test");
        let _ = std::fs::remove_dir_all(&worktree_path);
        let output = Command::new("git")
            .args(["worktree", "add", worktree_path.to_str().unwrap()])
            .current_dir(&path)
            .output()
            .unwrap();
        assert!(output.status.success(), "git worktree add failed: {:?}", output);
        let info = detect_git_info(&worktree_path);
        assert!(info.is_worktree);
        assert!(info.worktree.is_some());
    }
}
```

- [ ] **Step 2: Add tempfile dev-dependency to Cargo.toml**

In `src-tauri/Cargo.toml`, add after `[dependencies]` block:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run tests**

```bash
cd src-tauri && cargo test --lib infrastructure::git_info::tests
```

Expected: all 4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/infrastructure/git_info.rs
git commit -m "test: add unit tests for git_info detection"
```
