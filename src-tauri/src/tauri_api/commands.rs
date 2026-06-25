use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::{
    domain::{
        config::DevappConfig,
        project::{ProjectId, ProjectRecord, ProjectSource},
        runtime::RunSessionSnapshot,
        terminal::{TerminalSessionId, TerminalSnapshot},
    },
    error::AppError,
    infrastructure::{
        config_loader::{load_config, parse_config_document},
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

fn window_key(window: &WebviewWindow) -> String {
    window.label().to_string()
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectRecord>, String> {
    let store = state.project_store.lock().await;
    store.load().map_err(String::from)
}

#[tauri::command]
pub async fn get_launch_project(state: State<'_, AppState>) -> Result<Option<ProjectId>, String> {
    let launch_project_id = state.launch_project_id.lock().await;
    Ok(launch_project_id.clone())
}

#[tauri::command]
pub async fn save_project(
    state: State<'_, AppState>,
    request: SaveProjectRequest,
) -> Result<ProjectRecord, String> {
    let store = state.project_store.lock().await;
    let record = store
        .prepare_project_record(
            request.id,
            request.name,
            request.base_dir,
            request.config_source,
        )
        .map_err(String::from)?;
    store.upsert(record).map_err(String::from)
}

#[tauri::command]
pub async fn remove_project(
    state: State<'_, AppState>,
    project_id: ProjectId,
) -> Result<(), String> {
    let store = state.project_store.lock().await;
    store.remove(&project_id).map_err(String::from)
}

#[tauri::command]
pub async fn load_project_config(
    state: State<'_, AppState>,
    request: LoadProjectConfigRequest,
) -> Result<ProjectConfigDocument, String> {
    let store = state.project_store.lock().await;
    let project = store
        .get(&request.project_id)
        .map_err(String::from)?
        .ok_or_else(|| AppError::project_store("project not found").to_string())?;

    let yaml = match request.yaml {
        Some(yaml) => yaml,
        None => store
            .load_project_config_raw(&project)
            .map_err(String::from)?,
    };

    let loaded = parse_config_document(&project.config_path, &yaml).map_err(String::from)?;
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
    let store = state.project_store.lock().await;
    let project = store
        .get(&request.project_id)
        .map_err(String::from)?
        .ok_or_else(|| AppError::project_store("project not found").to_string())?;
    let loaded =
        parse_config_document(&project.config_path, &request.yaml).map_err(String::from)?;
    store
        .save_project_config_raw(&project, &request.yaml)
        .map_err(String::from)?;
    Ok(ProjectConfigDocument {
        project,
        yaml: request.yaml,
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
    let project = {
        let store = state.project_store.lock().await;
        store
            .get(&request.project_id)
            .map_err(String::from)?
            .ok_or_else(|| AppError::project_store("project not found").to_string())?
    };

    let loaded = load_config(&project.config_path).map_err(String::from)?;
    state
        .orchestrator
        .start_session(app_handle, window_key(&window), project, loaded)
        .await
        .map_err(String::from)
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
        .map_err(String::from)?;
    state
        .terminal_manager
        .close_all_for_window(app_handle, &key)
        .map_err(String::from)?;
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
        .map_err(String::from)
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
        .map_err(String::from)
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
        .map_err(String::from)
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
        .map_err(String::from)
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
            .map_err(String::from)?
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
        .map_err(String::from)
}

#[tauri::command]
pub async fn write_terminal(
    state: State<'_, AppState>,
    request: WriteTerminalRequest,
) -> Result<(), String> {
    state
        .terminal_manager
        .write_terminal(&request.terminal_id, &request.data)
        .map_err(String::from)
}

#[tauri::command]
pub async fn resize_terminal(
    state: State<'_, AppState>,
    request: ResizeTerminalRequest,
) -> Result<(), String> {
    state
        .terminal_manager
        .resize_terminal(&request.terminal_id, request.cols, request.rows)
        .map_err(String::from)
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
        .map_err(String::from)
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
    let project = {
        let store = state.project_store.lock().await;
        store
            .get(&request.project_id)
            .map_err(String::from)?
            .ok_or_else(|| AppError::project_store("project not found").to_string())?
    };

    let project_id = project.id.0.to_string();
    let label = format!("project-{project_id}");
    if let Some(window) = app_handle.get_webview_window(&label) {
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let url = WebviewUrl::App(format!("?projectId={project_id}&autorun=1").into());
    WebviewWindowBuilder::new(&app_handle, label, url)
        .title(format!("{} — devapp", project.name))
        .build()
        .map_err(|error| error.to_string())?;

    Ok(())
}
