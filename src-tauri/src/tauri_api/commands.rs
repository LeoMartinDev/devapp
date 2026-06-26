use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

use crate::{
    domain::{
        config::DevappConfig,
        project::{ProjectId, ProjectRecord, ProjectSource},
        runtime::RunSessionSnapshot,
        terminal::{TerminalSessionId, TerminalSnapshot},
    },
    error::{AppError, ErrorCode},
    infrastructure::{
        config_loader::{load_config_async, parse_config_document},
        git_info::{self, GitInfo},
    },
    tauri_api::state::AppState,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectRequest {
    pub id: Option<ProjectId>,
    pub name: String,
    pub base_dir: PathBuf,
    pub config_source: Option<ProjectSource>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadProjectConfigRequest {
    pub project_id: ProjectId,
    pub yaml: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfigDocument {
    pub project: ProjectRecord,
    pub yaml: String,
    pub config: DevappConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectConfigRequest {
    pub project_id: ProjectId,
    pub yaml: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchProjectInfo {
    pub project_id: Option<ProjectId>,
    pub locked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectActionRequest {
    pub project_id: ProjectId,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessActionRequest {
    pub process_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTerminalRequest {
    pub project_id: ProjectId,
    pub title: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTerminalRequest {
    pub terminal_id: TerminalSessionId,
    pub data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeTerminalRequest {
    pub terminal_id: TerminalSessionId,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseTerminalRequest {
    pub terminal_id: TerminalSessionId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppErrorPayload {
    message: String,
    code: ErrorCode,
}

fn to_error_string(error: AppError) -> String {
    let payload = AppErrorPayload {
        message: error.to_string(),
        code: error.code(),
    };
    serde_json::to_string(&payload).unwrap_or_else(|_| error.to_string())
}

fn window_key(window: &WebviewWindow) -> String {
    window.label().to_string()
}

async fn check_launch_locked(state: &AppState) -> Result<(), String> {
    if state.launch_project_id.lock().await.is_some() {
        return Err(AppError::LaunchLocked.to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectRecord>, String> {
    let store = state.project_store.lock().await;
    store.load().map_err(to_error_string)
}

#[tauri::command]
pub async fn get_launch_project(
    state: State<'_, AppState>,
) -> Result<LaunchProjectInfo, String> {
    let launch_project_id = state.launch_project_id.lock().await;
    Ok(LaunchProjectInfo {
        project_id: launch_project_id.clone(),
        locked: launch_project_id.is_some(),
    })
}

#[tauri::command]
pub async fn save_project(
    state: State<'_, AppState>,
    request: SaveProjectRequest,
) -> Result<ProjectRecord, String> {
    check_launch_locked(&state).await?;
    let store = state.project_store.lock().await;
    let record = store
        .prepare_project_record(
            request.id,
            request.name,
            request.base_dir,
            request.config_source,
        )
        .map_err(to_error_string)?;
    store.upsert(record).map_err(to_error_string)
}

#[tauri::command]
pub async fn remove_project(
    state: State<'_, AppState>,
    project_id: ProjectId,
) -> Result<(), String> {
    check_launch_locked(&state).await?;
    let store = state.project_store.lock().await;
    store.remove(&project_id).map_err(to_error_string)
}

#[tauri::command]
pub async fn load_project_config(
    state: State<'_, AppState>,
    request: LoadProjectConfigRequest,
) -> Result<ProjectConfigDocument, String> {
    let (project, yaml) = {
        let store = state.project_store.lock().await;
        let project = store
            .get(&request.project_id)
            .map_err(to_error_string)?
            .ok_or_else(|| AppError::project_store("project not found").to_string())?;

        let yaml = match request.yaml {
            Some(yaml) => yaml,
            None => store
                .load_project_config_raw(&project)
                .map_err(to_error_string)?,
        };

        (project, yaml)
    };

    let loaded = parse_config_document(&project.config_path, &yaml).map_err(to_error_string)?;
    Ok(ProjectConfigDocument {
        project,
        yaml,
        config: loaded.config,
    })
}

#[tauri::command]
pub async fn save_project_config(
    state: State<'_, AppState>,
    request: SaveProjectConfigRequest,
) -> Result<ProjectConfigDocument, String> {
    let (project, yaml) = {
        let store = state.project_store.lock().await;
        let project = store
            .get(&request.project_id)
            .map_err(to_error_string)?
            .ok_or_else(|| AppError::project_store("project not found").to_string())?;
        (project, request.yaml)
    };
    let loaded =
        parse_config_document(&project.config_path, &yaml).map_err(to_error_string)?;
    let store = state.project_store.lock().await;
    store
        .save_project_config_raw(&project, &yaml)
        .map_err(to_error_string)?;
    Ok(ProjectConfigDocument {
        project,
        yaml,
        config: loaded.config,
    })
}

#[tauri::command]
pub async fn start_project(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    request: ProjectActionRequest,
) -> Result<RunSessionSnapshot, String> {
    {
        let locked_id = state.launch_project_id.lock().await;
        if let Some(locked) = locked_id.as_ref() {
            if locked != &request.project_id {
                return Err(AppError::LaunchLocked.to_string());
            }
        }
    }

    let project = {
        let store = state.project_store.lock().await;
        store
            .get(&request.project_id)
            .map_err(to_error_string)?
            .ok_or_else(|| AppError::project_store("project not found").to_string())?
    };

    let loaded = load_config_async(&project.config_path).await.map_err(to_error_string)?;
    state
        .orchestrator
        .start_session(app_handle, window_key(&window), project, loaded)
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn stop_project(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<Option<RunSessionSnapshot>, String> {
    let key = window_key(&window);
    let snapshot = state
        .orchestrator
        .stop_session(app_handle.clone(), &key)
        .await
        .map_err(to_error_string)?;
    state
        .terminal_manager
        .close_all_for_window(app_handle, &key)
        .await
        .map_err(to_error_string)?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn restart_process(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    request: ProcessActionRequest,
) -> Result<Option<RunSessionSnapshot>, String> {
    state
        .orchestrator
        .restart_process(app_handle, &window_key(&window), &request.process_name)
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn start_process(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    request: ProcessActionRequest,
) -> Result<Option<RunSessionSnapshot>, String> {
    state
        .orchestrator
        .start_process(app_handle, &window_key(&window), &request.process_name)
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn stop_process(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    request: ProcessActionRequest,
) -> Result<Option<RunSessionSnapshot>, String> {
    state
        .orchestrator
        .stop_process(app_handle, &window_key(&window), &request.process_name)
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn get_session_snapshot(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<Option<RunSessionSnapshot>, String> {
    state
        .orchestrator
        .snapshot(&window_key(&window))
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn open_terminal(
    app_handle: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    request: OpenTerminalRequest,
) -> Result<TerminalSnapshot, String> {
    let project = {
        let store = state.project_store.lock().await;
        store
            .get(&request.project_id)
            .map_err(to_error_string)?
            .ok_or_else(|| AppError::project_store("project not found").to_string())?
    };
    state
        .terminal_manager
        .open_terminal(
            app_handle,
            window_key(&window),
            request
                .title
                .unwrap_or_else(|| format!("{} shell", project.name)),
            &project.base_dir,
            request.cols.unwrap_or(100),
            request.rows.unwrap_or(28),
        )
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn write_terminal(
    state: State<'_, AppState>,
    request: WriteTerminalRequest,
) -> Result<(), String> {
    state
        .terminal_manager
        .write_terminal(&request.terminal_id, &request.data)
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn resize_terminal(
    state: State<'_, AppState>,
    request: ResizeTerminalRequest,
) -> Result<(), String> {
    state
        .terminal_manager
        .resize_terminal(&request.terminal_id, request.cols, request.rows)
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub async fn close_terminal(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: CloseTerminalRequest,
) -> Result<Option<TerminalSnapshot>, String> {
    state
        .terminal_manager
        .close_terminal(app_handle, &request.terminal_id)
        .await
        .map_err(to_error_string)
}

#[tauri::command]
pub fn get_git_info(base_dir: String) -> Result<GitInfo, String> {
    Ok(git_info::detect_git_info(std::path::Path::new(&base_dir)))
}

#[tauri::command]
pub async fn open_project_window(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: ProjectActionRequest,
) -> Result<(), String> {
    check_launch_locked(&state).await?;

    let project = {
        let store = state.project_store.lock().await;
        store
            .get(&request.project_id)
            .map_err(to_error_string)?
            .ok_or_else(|| AppError::project_store("project not found").to_string())?
    };

    let project_id = project.id.0.to_string();
    let label = format!("project-{project_id}");
    if let Some(window) = app_handle.get_webview_window(&label) {
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let url = WebviewUrl::App(format!("?projectId={project_id}&autorun=1").into());
    let win_builder = WebviewWindowBuilder::new(&app_handle, label, url)
        .title(format!("{} — devapp", project.name));

    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Overlay);

    #[cfg(not(target_os = "macos"))]
    let win_builder = win_builder.decorations(false);

    win_builder.build().map_err(|error| error.to_string())?;

    Ok(())
}
