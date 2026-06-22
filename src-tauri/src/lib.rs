pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod tauri_api;

use std::path::PathBuf;

use tauri::Manager;

use crate::tauri_api::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new().expect("failed to initialize app state");
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = app.state::<AppState>().inner().clone();
            tauri::async_runtime::block_on(async move {
                if let Some(config_path) = launch_config_path() {
                    let project = {
                        let store = state.project_store.lock().await;
                        store.import_project_config_path(config_path)?
                    };
                    let mut launch_project_id = state.launch_project_id.lock().await;
                    *launch_project_id = Some(project.id);
                }
                Ok::<(), crate::error::AppError>(())
            })
            .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
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
            tauri_api::commands::open_project_window
        ])
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
