use std::{collections::HashMap, sync::Arc, thread};

use chrono::Utc;
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
    sync::{broadcast, mpsc, Mutex},
};

use crate::{
    application::{
        command_runner::spawn_process,
        events::{
            ProcessLogEvent, RuntimeEvent, SessionStatusEvent, PROCESS_LOG_EVENT,
            RUNTIME_ERROR_EVENT, SESSION_SNAPSHOT_EVENT,
        },
        readiness::wait_until_ready,
    },
    domain::{
        config::{DependencyCondition, ProcessConfig, ProcessKind},
        process::{LogStream, ProcessStatus},
        project::ProjectRecord,
        runtime::{
            ProcessLogPayload, ProcessRuntimeId, ProcessSnapshot, RunSessionId, RunSessionSnapshot,
        },
    },
    error::AppError,
    infrastructure::{
        config_loader::LoadedProjectConfig,
        log_store::{InMemoryLogStore, LogStore},
    },
};

#[derive(Clone)]
pub struct ProcessOrchestrator {
    inner: Arc<Mutex<OrchestratorState>>,
}

struct OrchestratorState {
    sessions: HashMap<String, ActiveSession>,
}

struct ActiveSession {
    snapshot: RunSessionSnapshot,
    project: ProjectRecord,
    loaded_config: LoadedProjectConfig,
    processes: HashMap<String, ManagedProcess>,
    stop_requested: bool,
    logs: InMemoryLogStore,
}

struct ManagedProcess {
    config: ProcessConfig,
    snapshot: ProcessSnapshot,
    child: Option<Arc<Mutex<Child>>>,
    /// OS process id of the immediate child. Stored so that stop/termination
    /// can signal the process group without locking the child mutex (which is
    /// held by the background wait task for the child's whole lifetime).
    pid: Option<u32>,
    /// One-shot kill signal consumed by the process's background wait task.
    /// Routing kills through this channel (rather than locking the child
    /// mutex to call `Child::kill`) is what lets `stop_process` and
    /// `finish_session` terminate a running service without deadlocking the
    /// wait task, which holds that mutex for the child's whole lifetime.
    kill_tx: Option<mpsc::Sender<()>>,
    log_tx: broadcast::Sender<String>,
    terminating: bool,
}

impl ProcessOrchestrator {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(OrchestratorState {
                sessions: HashMap::new(),
            })),
        }
    }

    pub async fn start_session(
        &self,
        app_handle: AppHandle,
        window_key: String,
        project: ProjectRecord,
        loaded_config: LoadedProjectConfig,
    ) -> Result<RunSessionSnapshot, AppError> {
        let snapshot = {
            let mut state = self.inner.lock().await;
            if let Some(active) = state.sessions.get(&window_key) {
                if active.snapshot.stopped_at.is_none() {
                    return Err(AppError::runtime(
                        "a project session is already running in this window",
                    ));
                }
            }

            let session_id = RunSessionId::new();
            let started_at = Utc::now();
            let mut session_snapshot = RunSessionSnapshot {
                session_id: session_id.clone(),
                project_id: project.id.clone(),
                project_name: project.name.clone(),
                base_dir: project.base_dir.clone(),
                started_at,
                stopped_at: None,
                processes: Vec::new(),
            };

            let mut processes = HashMap::new();
            for (name, config) in &loaded_config.config.processes {
                let (log_tx, _) = broadcast::channel(256);
                let snapshot = ProcessSnapshot {
                    runtime_id: ProcessRuntimeId::new(),
                    name: name.clone(),
                    kind: config.kind.clone(),
                    status: ProcessStatus::Pending,
                    started_at: None,
                    exited_at: None,
                    exit_code: None,
                };
                session_snapshot.processes.push(snapshot.clone());
                processes.insert(
                    name.clone(),
                    ManagedProcess {
                        config: config.clone(),
                        snapshot,
                        child: None,
                        pid: None,
                        kill_tx: None,
                        log_tx,
                        terminating: false,
                    },
                );
            }

            state.sessions.insert(
                window_key.clone(),
                ActiveSession {
                    snapshot: session_snapshot.clone(),
                    project,
                    loaded_config,
                    processes,
                    stop_requested: false,
                    logs: InMemoryLogStore::default(),
                },
            );

            session_snapshot
        };

        self.emit_snapshot(&app_handle, &window_key).await?;
        self.spawn_runnable_processes(app_handle.clone(), &window_key)
            .await?;

        Ok(snapshot)
    }

    pub async fn stop_session(
        &self,
        app_handle: AppHandle,
        window_key: &str,
    ) -> Result<Option<RunSessionSnapshot>, AppError> {
        self.finish_session(app_handle, window_key, None, true)
            .await
    }

    pub async fn restart_process(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        process_name: &str,
    ) -> Result<Option<RunSessionSnapshot>, AppError> {
        {
            let state = self.inner.lock().await;
            let active = state
                .sessions
                .get(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            let process = active
                .processes
                .get(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
            if matches!(process.config.kind, ProcessKind::Task) {
                return Err(AppError::runtime("cannot restart a task process"));
            }
        }
        self.stop_process(app_handle.clone(), window_key, process_name)
            .await?;
        self.reset_process(window_key, process_name).await?;
        self.spawn_runnable_processes(app_handle.clone(), window_key)
            .await?;
        self.snapshot(window_key).await
    }

    pub async fn start_process(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        process_name: &str,
    ) -> Result<Option<RunSessionSnapshot>, AppError> {
        self.reset_process(window_key, process_name).await?;
        self.spawn_runnable_processes(app_handle.clone(), window_key)
            .await?;
        self.snapshot(window_key).await
    }

    pub async fn stop_process(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        process_name: &str,
    ) -> Result<Option<RunSessionSnapshot>, AppError> {
        let kill_tx = {
            let mut state = self.inner.lock().await;
            let active = state
                .sessions
                .get_mut(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            let process = active
                .processes
                .get_mut(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
            if matches!(process.config.kind, ProcessKind::Task) {
                return Err(AppError::runtime("cannot stop a task process"));
            }
            let kill_tx = begin_process_termination(process);
            sync_snapshot_process(&mut active.snapshot, &process.snapshot);
            kill_tx
        };

        // Signal the background wait task to kill the child. We must NOT lock
        // the child mutex here: the wait task holds it for the child's whole
        // lifetime, so contending it would deadlock.
        if let Some(kill_tx) = kill_tx {
            let _ = kill_tx.send(()).await;
        }

        self.emit_snapshot(&app_handle, window_key).await?;
        Ok(self.snapshot(window_key).await?)
    }

    pub async fn snapshot(&self, window_key: &str) -> Result<Option<RunSessionSnapshot>, AppError> {
        let state = self.inner.lock().await;
        Ok(state
            .sessions
            .get(window_key)
            .map(|active| active.snapshot.clone()))
    }

    async fn reset_process(&self, window_key: &str, process_name: &str) -> Result<(), AppError> {
        let mut state = self.inner.lock().await;
        let active = state
            .sessions
            .get_mut(window_key)
            .ok_or_else(|| AppError::runtime("no session available"))?;
        let process = active
            .processes
            .get_mut(process_name)
            .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
        reset_managed_process(process);
        sync_snapshot_process(&mut active.snapshot, &process.snapshot);
        Ok(())
    }

    async fn spawn_runnable_processes(
        &self,
        app_handle: AppHandle,
        window_key: &str,
    ) -> Result<(), AppError> {
        let runnable = {
            let state = self.inner.lock().await;
            let Some(active) = state.sessions.get(window_key) else {
                return Ok(());
            };
            if active.stop_requested {
                return Ok(());
            }
            active
                .processes
                .iter()
                .filter_map(|(name, process)| {
                    if process.child.is_some()
                        || !matches!(
                            process.snapshot.status,
                            ProcessStatus::Pending
                                | ProcessStatus::Blocked
                                | ProcessStatus::Stopped
                        )
                    {
                        return None;
                    }
                    if dependencies_satisfied(&active.processes, &process.config) {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        for process_name in runnable {
            Box::pin(self.spawn_named_process(app_handle.clone(), window_key, &process_name))
                .await?;
        }

        Ok(())
    }

    async fn spawn_named_process(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        process_name: &str,
    ) -> Result<(), AppError> {
        let (session_id, project_name, base_dir, env, config, runtime_id, log_tx) = {
            let mut state = self.inner.lock().await;
            let active = state
                .sessions
                .get_mut(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            let session_id = active.snapshot.session_id.clone();
            let project_name = active.project.name.clone();
            let process = active
                .processes
                .get_mut(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
            if process.child.is_some() {
                return Ok(());
            }
            let env = process
                .config
                .env
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<HashMap<_, _>>();
            process.snapshot.status = ProcessStatus::Starting;
            process.snapshot.started_at = Some(Utc::now());
            process.snapshot.exited_at = None;
            process.snapshot.exit_code = None;
            sync_snapshot_process(&mut active.snapshot, &process.snapshot);
            (
                session_id,
                project_name,
                active.loaded_config.base_dir.clone(),
                env,
                process.config.clone(),
                process.snapshot.runtime_id.clone(),
                process.log_tx.clone(),
            )
        };

        self.emit_snapshot(&app_handle, window_key).await?;

        let spawned = spawn_process(&config.cmd, &base_dir, &env)?;
        let child_pid = spawned.child.id();
        let child = Arc::new(Mutex::new(spawned.child));
        let (kill_tx, mut kill_rx) = mpsc::channel::<()>(1);

        {
            let mut state = self.inner.lock().await;
            let active = state
                .sessions
                .get_mut(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            let process = active
                .processes
                .get_mut(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
            process.child = Some(child.clone());
            process.pid = child_pid;
            process.kill_tx = Some(kill_tx);
            process.snapshot.status = ProcessStatus::Running;
            sync_snapshot_process(&mut active.snapshot, &process.snapshot);
        }

        self.emit_snapshot(&app_handle, window_key).await?;

        self.spawn_log_task(
            app_handle.clone(),
            window_key.to_string(),
            session_id.clone(),
            process_name.to_string(),
            runtime_id.clone(),
            LogStream::Stdout,
            spawned.stdout,
            log_tx.clone(),
        );
        self.spawn_log_task(
            app_handle.clone(),
            window_key.to_string(),
            session_id.clone(),
            process_name.to_string(),
            runtime_id.clone(),
            LogStream::Stderr,
            spawned.stderr,
            log_tx.clone(),
        );

        if matches!(config.kind, ProcessKind::Service) {
            if let Some(ready) = config.ready.clone() {
                let orchestrator = self.clone();
                let env_for_ready = env.clone();
                let base_dir_for_ready = base_dir.clone();
                let process_name_for_ready = process_name.to_string();
                let readiness_app_handle = app_handle.clone();
                let readiness_window_key = window_key.to_string();
                thread::spawn(move || {
                    tauri::async_runtime::block_on(async move {
                        let result = wait_until_ready(
                            &ready,
                            &base_dir_for_ready,
                            &env_for_ready,
                            Some(log_tx.subscribe()),
                        )
                        .await;
                        match result {
                            Ok(()) => {
                                let _ = orchestrator
                                    .mark_process_ready(
                                        readiness_app_handle.clone(),
                                        &readiness_window_key,
                                        &process_name_for_ready,
                                    )
                                    .await;
                            }
                            Err(error) => {
                                let _ = readiness_app_handle.emit_to(
                                    &readiness_window_key,
                                    RUNTIME_ERROR_EVENT,
                                    RuntimeEvent::RuntimeError {
                                        message: error.to_string(),
                                    },
                                );
                                let _ = orchestrator
                                    .handle_process_failure(
                                        readiness_app_handle.clone(),
                                        &readiness_window_key,
                                        &process_name_for_ready,
                                        None,
                                        error.to_string(),
                                    )
                                    .await;
                            }
                        }
                    });
                });
            } else {
                self.mark_process_ready(app_handle.clone(), window_key, process_name)
                    .await?;
            }
        }

        let orchestrator = self.clone();
        let process_name_for_wait = process_name.to_string();
        let exit_app_handle = app_handle.clone();
        let exit_window_key = window_key.to_string();
        thread::spawn(move || {
            tauri::async_runtime::block_on(async move {
                // The stop/kill paths signal through `kill_rx` rather than
                // acquiring the child mutex. `tokio::process::Child::wait`
                // borrows `&mut Child` for its whole lifetime, so while this
                // task waits it MUST hold the child lock. If `stop_process`
                // tried to take that same lock to call `kill()`, the two
                // would deadlock for any long-running service and the process
                // would be stuck in `Stopping` forever. Listening for the
                // kill signal here lets us kill the child from within the
                // lock we already hold.
                let exit_status = {
                    let mut child = child.lock().await;
                    tokio::select! {
                        result = child.wait() => result,
                        _ = kill_rx.recv() => {
                            let _ = child.kill().await;
                            child.wait().await
                        }
                    }
                };
                match exit_status {
                    Ok(status) => {
                        let code = status.code();
                        let _ = orchestrator
                            .handle_process_exit(
                                exit_app_handle.clone(),
                                &exit_window_key,
                                &process_name_for_wait,
                                code,
                                status.success(),
                            )
                            .await;
                    }
                    Err(error) => {
                        let _ = orchestrator
                            .handle_process_failure(
                                exit_app_handle.clone(),
                                &exit_window_key,
                                &process_name_for_wait,
                                None,
                                error.to_string(),
                            )
                            .await;
                    }
                }
            });
        });

        let _ = project_name;
        Ok(())
    }

    fn spawn_log_task<R>(
        &self,
        app_handle: AppHandle,
        window_key: String,
        session_id: RunSessionId,
        process_name: String,
        runtime_id: ProcessRuntimeId,
        stream: LogStream,
        reader: R,
        log_tx: broadcast::Sender<String>,
    ) where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
    {
        let orchestrator = self.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = log_tx.send(line.clone());
                let payload = ProcessLogPayload {
                    session_id: session_id.clone(),
                    runtime_id: runtime_id.clone(),
                    process_name: process_name.clone(),
                    stream,
                    line,
                    timestamp: Utc::now(),
                };
                let _ = orchestrator.append_log(&window_key, payload.clone()).await;
                let _ =
                    app_handle.emit_to(&window_key, PROCESS_LOG_EVENT, ProcessLogEvent { payload });
            }
        });
    }

    async fn append_log(
        &self,
        window_key: &str,
        payload: ProcessLogPayload,
    ) -> Result<(), AppError> {
        let mut state = self.inner.lock().await;
        let active = state
            .sessions
            .get_mut(window_key)
            .ok_or_else(|| AppError::runtime("no session available"))?;
        active.logs.append(payload);
        Ok(())
    }

    async fn mark_process_ready(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        process_name: &str,
    ) -> Result<(), AppError> {
        {
            let mut state = self.inner.lock().await;
            let active = state
                .sessions
                .get_mut(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            let process = active
                .processes
                .get_mut(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
            if process.terminating {
                return Ok(());
            }
            process.snapshot.status = ProcessStatus::Ready;
            sync_snapshot_process(&mut active.snapshot, &process.snapshot);
        }
        self.emit_snapshot(&app_handle, window_key).await?;
        self.spawn_runnable_processes(app_handle, window_key)
            .await?;
        Ok(())
    }

    async fn handle_process_exit(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        process_name: &str,
        exit_code: Option<i32>,
        success: bool,
    ) -> Result<(), AppError> {
        let (kind, terminating) = {
            let mut state = self.inner.lock().await;
            let active = state
                .sessions
                .get_mut(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            let process = active
                .processes
                .get_mut(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;
            process.child = None;
            process.snapshot.exited_at = Some(Utc::now());
            process.snapshot.exit_code = exit_code;
            let terminating = process.terminating || active.stop_requested;

            if terminating {
                process.snapshot.status = ProcessStatus::Stopped;
            } else if success && matches!(process.config.kind, ProcessKind::Task) {
                process.snapshot.status = ProcessStatus::Succeeded;
            } else {
                process.snapshot.status = ProcessStatus::Failed;
            }
            sync_snapshot_process(&mut active.snapshot, &process.snapshot);
            (process.config.kind.clone(), terminating)
        };

        self.emit_snapshot(&app_handle, window_key).await?;

        if terminating {
            return Ok(());
        }

        if success && matches!(kind, ProcessKind::Task) {
            self.spawn_runnable_processes(app_handle, window_key)
                .await?;
            return Ok(());
        }

        self.finish_session(
            app_handle,
            window_key,
            Some(format!("process `{process_name}` exited unexpectedly")),
            false,
        )
        .await?;
        Ok(())
    }

    async fn handle_process_failure(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        process_name: &str,
        exit_code: Option<i32>,
        message: String,
    ) -> Result<(), AppError> {
        let terminating = {
            let mut state = self.inner.lock().await;
            let active = state
                .sessions
                .get_mut(window_key)
                .ok_or_else(|| AppError::runtime("no session available"))?;
            let process = active
                .processes
                .get_mut(process_name)
                .ok_or_else(|| AppError::runtime(format!("unknown process `{process_name}`")))?;

            let terminating = process.terminating || active.stop_requested;
            if terminating {
                process.snapshot.status = ProcessStatus::Stopped;
            } else {
                process.snapshot.status = ProcessStatus::Failed;
            }
            process.snapshot.exited_at = Some(Utc::now());
            process.snapshot.exit_code = exit_code;
            process.child = None;
            sync_snapshot_process(&mut active.snapshot, &process.snapshot);
            terminating
        };

        self.emit_snapshot(&app_handle, window_key).await?;

        if terminating {
            return Ok(());
        }

        self.finish_session(app_handle, window_key, Some(message), false)
            .await?;
        Ok(())
    }

    async fn finish_session(
        &self,
        app_handle: AppHandle,
        window_key: &str,
        failure_message: Option<String>,
        explicit_stop: bool,
    ) -> Result<Option<RunSessionSnapshot>, AppError> {
        let kill_txs = {
            let mut state = self.inner.lock().await;
            let Some(active) = state.sessions.get_mut(window_key) else {
                return Ok(None);
            };
            active.stop_requested = true;
            let stopped_at = Utc::now();
            active.snapshot.stopped_at = Some(stopped_at);
            let mut kill_txs = Vec::new();
            for process in active.processes.values_mut() {
                if let Some(kill_tx) = begin_process_termination(process) {
                    kill_txs.push(kill_tx);
                } else if explicit_stop
                    && matches!(
                        process.snapshot.status,
                        ProcessStatus::Pending | ProcessStatus::Blocked
                    )
                {
                    process.snapshot.status = ProcessStatus::Stopped;
                }
                sync_snapshot_process(&mut active.snapshot, &process.snapshot);
            }
            kill_txs
        };

        self.emit_snapshot(&app_handle, window_key).await?;

        // Signal each wait task to kill its own child. Locking the child
        // mutexes here would deadlock against the wait tasks.
        for kill_tx in kill_txs {
            let _ = kill_tx.send(()).await;
        }

        if let Some(message) = failure_message {
            app_handle
                .emit_to(
                    window_key,
                    RUNTIME_ERROR_EVENT,
                    RuntimeEvent::RuntimeError { message },
                )
                .map_err(|error| AppError::runtime(error.to_string()))?;
        }

        self.emit_snapshot(&app_handle, window_key).await?;
        self.snapshot(window_key).await
    }

    async fn emit_snapshot(
        &self,
        app_handle: &AppHandle,
        window_key: &str,
    ) -> Result<(), AppError> {
        let snapshot = self.snapshot(window_key).await?;
        app_handle
            .emit_to(
                window_key,
                SESSION_SNAPSHOT_EVENT,
                SessionStatusEvent { snapshot },
            )
            .map_err(|error| AppError::runtime(error.to_string()))
    }
}

fn dependencies_satisfied(
    processes: &HashMap<String, ManagedProcess>,
    config: &ProcessConfig,
) -> bool {
    config
        .depends_on
        .iter()
        .all(|(dependency_name, condition)| {
            let Some(dependency) = processes.get(dependency_name) else {
                return false;
            };
            match condition {
                DependencyCondition::Success => {
                    matches!(dependency.snapshot.status, ProcessStatus::Succeeded)
                }
                DependencyCondition::Ready => {
                    matches!(dependency.snapshot.status, ProcessStatus::Ready)
                }
            }
        })
}

fn sync_snapshot_process(
    session_snapshot: &mut RunSessionSnapshot,
    process_snapshot: &ProcessSnapshot,
) {
    if let Some(existing) = session_snapshot
        .processes
        .iter_mut()
        .find(|process| process.runtime_id == process_snapshot.runtime_id)
    {
        *existing = process_snapshot.clone();
    }
}

/// Resets a managed process to a clean `Pending` state so it can be re-spawned.
/// Extracted from `reset_process` to make the state transition unit-testable.
fn reset_managed_process(process: &mut ManagedProcess) {
    process.child = None;
    process.pid = None;
    process.kill_tx = None;
    process.terminating = false;
    process.snapshot.status = ProcessStatus::Pending;
    process.snapshot.started_at = None;
    process.snapshot.exited_at = None;
    process.snapshot.exit_code = None;
}

/// Marks a managed process for termination and returns its kill signal
/// sender if the child is still alive. Extracted from `stop_process` /
/// `finish_session` to make the state transition unit-testable.
///
/// The deadlock that prompted this helper: the background wait task holds
/// the child mutex for the child's whole lifetime (because `Child::wait`
/// borrows `&mut Child`), so the kill path MUST NOT lock that mutex to call
/// `Child::kill`. Instead it takes `kill_tx` here and the wait task performs
/// the kill inside the lock it already holds, signaled via the channel.
///
/// On Unix the process group is killed synchronously here (the pid is stored
/// separately from the child mutex so no deadlock is possible). This ensures
/// the process exits before `stop_process` returns its snapshot, eliminating
/// the "stuck at Stopping" race.
fn begin_process_termination(process: &mut ManagedProcess) -> Option<mpsc::Sender<()>> {
    process.terminating = true;

    #[cfg(unix)]
    if let Some(pid) = process.pid {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }

    if matches!(
        process.snapshot.status,
        ProcessStatus::Starting | ProcessStatus::Running | ProcessStatus::Ready
    ) {
        process.snapshot.status = ProcessStatus::Stopping;
        process.kill_tx.take()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use indexmap::IndexMap;
    use tokio::sync::broadcast;

    use super::*;
    use crate::domain::{
        config::{DependencyCondition, ProcessKind},
        project::ProjectId,
    };

    fn managed_process(name: &str, kind: ProcessKind, status: ProcessStatus) -> ManagedProcess {
        let (log_tx, _) = broadcast::channel(4);
        ManagedProcess {
            config: ProcessConfig {
                kind: kind.clone(),
                cmd: format!("run {name}"),
                env: IndexMap::new(),
                depends_on: IndexMap::new(),
                ready: None,
            },
            snapshot: ProcessSnapshot {
                runtime_id: ProcessRuntimeId::new(),
                name: name.to_string(),
                kind,
                status,
                started_at: None,
                exited_at: None,
                exit_code: None,
            },
            child: None,
            pid: None,
            kill_tx: None,
            log_tx,
            terminating: false,
        }
    }

    #[test]
    fn dependencies_satisfied_requires_success_for_task_dependency() {
        let mut processes = HashMap::new();
        processes.insert(
            "setup".to_string(),
            managed_process("setup", ProcessKind::Task, ProcessStatus::Succeeded),
        );
        let mut depends_on = IndexMap::new();
        depends_on.insert("setup".to_string(), DependencyCondition::Success);
        let config = ProcessConfig {
            kind: ProcessKind::Service,
            cmd: "deno task dev".to_string(),
            env: IndexMap::new(),
            depends_on,
            ready: None,
        };

        assert!(dependencies_satisfied(&processes, &config));

        processes
            .get_mut("setup")
            .expect("setup process")
            .snapshot
            .status = ProcessStatus::Ready;
        assert!(!dependencies_satisfied(&processes, &config));
    }

    #[test]
    fn dependencies_satisfied_requires_ready_for_service_dependency() {
        let mut processes = HashMap::new();
        processes.insert(
            "api".to_string(),
            managed_process("api", ProcessKind::Service, ProcessStatus::Running),
        );
        let mut depends_on = IndexMap::new();
        depends_on.insert("api".to_string(), DependencyCondition::Ready);
        let config = ProcessConfig {
            kind: ProcessKind::Service,
            cmd: "deno task dev".to_string(),
            env: IndexMap::new(),
            depends_on,
            ready: None,
        };

        assert!(!dependencies_satisfied(&processes, &config));

        processes
            .get_mut("api")
            .expect("api process")
            .snapshot
            .status = ProcessStatus::Ready;
        assert!(dependencies_satisfied(&processes, &config));
    }

    #[test]
    fn sync_snapshot_process_replaces_matching_runtime_snapshot() {
        let runtime_id = ProcessRuntimeId::new();
        let mut session_snapshot = RunSessionSnapshot {
            session_id: RunSessionId::new(),
            project_id: ProjectId::new(),
            project_name: "demo".to_string(),
            base_dir: std::env::temp_dir(),
            started_at: Utc::now(),
            stopped_at: None,
            processes: vec![ProcessSnapshot {
                runtime_id: runtime_id.clone(),
                name: "web".to_string(),
                kind: ProcessKind::Service,
                status: ProcessStatus::Running,
                started_at: None,
                exited_at: None,
                exit_code: None,
            }],
        };
        let updated = ProcessSnapshot {
            runtime_id,
            name: "web".to_string(),
            kind: ProcessKind::Service,
            status: ProcessStatus::Ready,
            started_at: Some(Utc::now()),
            exited_at: None,
            exit_code: None,
        };

        sync_snapshot_process(&mut session_snapshot, &updated);

        assert_eq!(session_snapshot.processes.len(), 1);
        assert_eq!(session_snapshot.processes[0], updated);
    }

    #[test]
    fn reset_managed_process_restores_pending_state() {
        let mut process = managed_process("api", ProcessKind::Service, ProcessStatus::Failed);
        process.terminating = true;
        process.snapshot.exit_code = Some(1);
        process.snapshot.exited_at = Some(Utc::now());

        reset_managed_process(&mut process);

        assert_eq!(process.snapshot.status, ProcessStatus::Pending);
        assert!(!process.terminating);
        assert!(process.child.is_none());
        assert_eq!(process.snapshot.exit_code, None);
        assert_eq!(process.snapshot.exited_at, None);
        assert_eq!(process.snapshot.started_at, None);
    }

    fn managed_process_with_kill_tx(
        name: &str,
        kind: ProcessKind,
        status: ProcessStatus,
    ) -> (ManagedProcess, mpsc::Receiver<()>) {
        let mut process = managed_process(name, kind.clone(), status);
        let (kill_tx, kill_rx) = mpsc::channel::<()>(1);
        process.kill_tx = Some(kill_tx);
        (process, kill_rx)
    }

    #[tokio::test]
    async fn begin_process_termination_transitions_running_to_stopping() {
        for status in [ProcessStatus::Starting, ProcessStatus::Running, ProcessStatus::Ready] {
            let (mut process, mut kill_rx) =
                managed_process_with_kill_tx("api", ProcessKind::Service, status);

            let kill_tx = begin_process_termination(&mut process);

            assert_eq!(process.snapshot.status, ProcessStatus::Stopping);
            assert!(process.terminating, "terminating flag should be set for {status:?}");
            let kill_tx = kill_tx.expect("kill signal should be returned for {status:?}");
            assert!(
                process.kill_tx.is_none(),
                "kill_tx must be taken (not cloned) so the channel drains"
            );
            // Mirrors `stop_process`: send, then drop. The wait task must
            // observe the signal.
            kill_tx.send(()).await.expect("kill signal should send");
            drop(kill_tx);
            assert_eq!(kill_rx.recv().await, Some(()));
        }
    }

    #[test]
    fn begin_process_termination_leaves_terminal_states_untouched() {
        // A process that already exited (Stopped/Failed/Succeeded) must not
        // produce a kill signal, otherwise stop would hang waiting on a child
        // that no longer exists.
        for status in [
            ProcessStatus::Pending,
            ProcessStatus::Blocked,
            ProcessStatus::Stopping,
            ProcessStatus::Stopped,
            ProcessStatus::Failed,
            ProcessStatus::Succeeded,
        ] {
            let (mut process, _kill_rx) =
                managed_process_with_kill_tx("api", ProcessKind::Service, status);

            let kill_tx = begin_process_termination(&mut process);

            assert!(
                kill_tx.is_none(),
                "no kill signal expected for terminal/pending state {status:?}"
            );
            assert_eq!(
                process.snapshot.status,
                status,
                "status must be unchanged for {status:?}"
            );
        }
    }
}
