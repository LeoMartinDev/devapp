pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod tauri_api;

use std::path::PathBuf;

use tauri::Manager;
use tracing::{error, info, warn};

use crate::infrastructure::config_loader::find_config_in_cwd_or_parents;
use crate::tauri_api::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // intentionally fatal — the app cannot run without app state
    let app_state = AppState::new().expect("failed to initialize app state");
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = window.set_title_bar_style(tauri::TitleBarStyle::Overlay) {
                    warn!(error = %err, "failed to set titlebar overlay style on macOS");
                }
            }

            #[cfg(not(target_os = "macos"))]
            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = window.set_decorations(false) {
                    warn!(error = %err, "failed to disable window decorations");
                }
            }

            let state = app.state::<AppState>().inner().clone();
            tauri::async_runtime::block_on(async move {
                let cwd = std::env::current_dir().ok();
                let cli_config_path = launch_config_path();
                let auto_detected_path = find_config_in_cwd_or_parents();
                let config_path = cli_config_path
                    .clone()
                    .or_else(|| auto_detected_path.clone());

                info!(
                    cwd = ?cwd,
                    cli_path = ?cli_config_path,
                    auto_detected_path = ?auto_detected_path,
                    selected_path = ?config_path,
                    "launch config detection"
                );

                if let Some(config_path) = config_path {
                    if cli_config_path.is_some() {
                        let project = {
                            let store = state.project_store.lock().await;
                            store.import_project_config_path(config_path)?
                        };
                        info!(project_id = ?project.id, "imported CLI config");
                        let mut launch_project_id = state.launch_project_id.lock().await;
                        *launch_project_id = Some(project.id);
                    } else {
                        let config_path_for_error = config_path.clone();
                        match state
                            .project_store
                            .lock()
                            .await
                            .import_project_config_path(config_path)
                        {
                            Ok(project) => {
                                info!(project_id = ?project.id, "imported auto-detected config");
                                let mut launch_project_id =
                                    state.launch_project_id.lock().await;
                                *launch_project_id = Some(project.id);
                            }
                            Err(error) => {
                                info!(error = %error, "failed to import auto-detected config");
                                let mut launch_error = state.launch_error.lock().await;
                                *launch_error = Some(format!(
                                    "Failed to load auto-detected config {}: {}",
                                    config_path_for_error.display(), error
                                ));
                            }
                        }
                    }
                }
                Ok::<(), crate::error::AppError>(())
            })
            .map_err(|err| {
                error!(error = %err, code = ?err.code(), "failed to import launch config");
                Box::<dyn std::error::Error>::from(err.to_string())
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_api::commands::get_launch_project,
            tauri_api::commands::list_projects,
            tauri_api::commands::save_project,
            tauri_api::commands::remove_project,
            tauri_api::commands::load_project_config,
            tauri_api::commands::save_project_config,
            tauri_api::commands::start_project,
            tauri_api::commands::stop_project,
            tauri_api::commands::restart_process,
            tauri_api::commands::start_process,
            tauri_api::commands::stop_process,
            tauri_api::commands::get_session_snapshot,
            tauri_api::commands::open_terminal,
            tauri_api::commands::write_terminal,
            tauri_api::commands::resize_terminal,
            tauri_api::commands::close_terminal,
            tauri_api::commands::open_project_window,
            tauri_api::commands::get_git_info
        ])
        // intentionally fatal — the app cannot recover from a Tauri runtime failure
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn launch_config_path() -> Option<PathBuf> {
    std::env::args_os()
        .skip(1)
        .find(|argument| !argument.to_string_lossy().starts_with("--"))
        .map(resolve_launch_config_path)
}

fn resolve_launch_config_path(argument: impl Into<PathBuf>) -> PathBuf {
    let path = argument.into();
    if path.is_absolute() || path.exists() {
        return path;
    }

    let Ok(current_dir) = std::env::current_dir() else {
        return path;
    };

    if current_dir
        .file_name()
        .is_some_and(|name| name == "src-tauri")
    {
        let workspace_relative = current_dir
            .parent()
            .map(|workspace| workspace.join(&path))
            .filter(|candidate| candidate.exists());
        if let Some(candidate) = workspace_relative {
            return candidate;
        }
    }

    path
}
