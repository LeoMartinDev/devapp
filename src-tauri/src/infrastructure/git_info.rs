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
