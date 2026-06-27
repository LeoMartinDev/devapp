use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    application::orchestrator::ProcessOrchestrator,
    domain::project::ProjectId,
    error::AppError,
    infrastructure::{project_store::ProjectStore, pty::TerminalManager},
};

#[derive(Clone)]
pub struct AppState {
    pub orchestrator: ProcessOrchestrator,
    pub project_store: Arc<Mutex<ProjectStore>>,
    pub terminal_manager: TerminalManager,
    pub launch_project_id: Arc<Mutex<Option<ProjectId>>>,
    pub launch_error: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            orchestrator: ProcessOrchestrator::new(),
            project_store: Arc::new(Mutex::new(ProjectStore::new()?)),
            terminal_manager: TerminalManager::new(),
            launch_project_id: Arc::new(Mutex::new(None)),
            launch_error: Arc::new(Mutex::new(None)),
        })
    }
}
