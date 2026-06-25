import { listen, type UnlistenFn } from "$lib/tauri/transport";

import {
  TAURI_EVENTS,
  closeTerminal,
  getLaunchProject,
  getGitInfo,
  getSessionSnapshot,
  listProjects,
  loadProjectConfig,
  openTerminal,
  removeProject,
  resizeTerminal,
  restartProcess,
  saveProject,
  saveProjectConfig,
  setWindowTitle,
  startProcess,
  startProject,
  stopProcess,
  stopProject,
  writeTerminal,
  type SaveProjectInput,
} from "$lib/tauri/client";
import type {
  GitInfo,
  ProcessLogEvent,
  ProcessLogPayload,
  ProcessRuntimeId,
  ProjectConfigDocument,
  ProjectId,
  ProjectRecord,
  RunSessionSnapshot,
  RuntimeErrorEvent,
  SessionStatusEvent,
  TerminalEvent,
  TerminalOutputEvent,
  TerminalSessionId,
  TerminalSnapshot,
} from "$lib/types";

export const MAX_LOG_LINES_PER_PROCESS = 10_000;

export type Selection =
  | { kind: "process"; runtimeId: ProcessRuntimeId }
  | { kind: "terminal"; terminalId: TerminalSessionId }
  | null;

class RuntimeStore {
  projects = $state<ProjectRecord[]>([]);
  session = $state<RunSessionSnapshot | null>(null);
  terminals = $state<TerminalSnapshot[]>([]);
  projectId = $state<ProjectId | null>(null);
  selectedProcessRuntimeId = $state<ProcessRuntimeId | null>(null);
  selectedTerminalId = $state<TerminalSessionId | null>(null);
  processLogs = $state<Record<string, ProcessLogPayload[]>>({});
  processLogTruncation = $state<Record<string, number>>({});
  terminalOutput = $state<Record<string, string>>({});
  projectConfig = $state<ProjectConfigDocument | null>(null);
  uiError = $state<string | null>(null);
  busy = $state(false);
  gitInfo = $state<GitInfo | null>(null);
  #gitInfoTimer: ReturnType<typeof setInterval> | null = null;

  #initialized = false;
  #unlisteners: UnlistenFn[] = [];

  async #fetchGitInfo() {
    const dir = this.project?.baseDir ?? this.session?.baseDir;
    if (!dir) return;
    try {
      this.gitInfo = await getGitInfo(dir);
    } catch {
      this.gitInfo = null;
    }
  }

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

  #startGitPolling() {
    this.#stopGitPolling();
    void this.#fetchGitInfo();
    this.#gitInfoTimer = setInterval(() => {
      void this.#fetchGitInfo();
    }, 60_000);
  }

  #stopGitPolling() {
    if (this.#gitInfoTimer !== null) {
      clearInterval(this.#gitInfoTimer);
      this.#gitInfoTimer = null;
    }
    this.gitInfo = null;
  }

  async init() {
    if (this.#initialized) {
      return;
    }
    this.#initialized = true;
    await this.refreshProjects();
    this.session = await getSessionSnapshot();
    if (this.session?.projectId) {
      this.projectId = this.session.projectId;
    }
    await this.#applyLaunchParams();
    window.addEventListener("focus", () => {
      void this.#fetchGitInfo();
    });
    this.syncProcessSelection();
    await this.#attachEventListeners();
  }

  async teardown() {
    for (const unlisten of this.#unlisteners) {
      await unlisten();
    }
    this.#unlisteners = [];
    this.#initialized = false;
  }

  get project(): ProjectRecord | null {
    return this.projects.find((project) => project.id === this.projectId) ?? null;
  }

  get selectedProcess() {
    return (
      this.session?.processes.find(
        (process) => process.runtimeId === this.selectedProcessRuntimeId,
      ) ?? null
    );
  }

  get selectedTerminal(): TerminalSnapshot | null {
    return this.terminals.find((terminal) => terminal.terminalId === this.selectedTerminalId) ?? null;
  }

  get selection(): Selection {
    if (this.selectedProcessRuntimeId) {
      return { kind: "process", runtimeId: this.selectedProcessRuntimeId };
    }
    if (this.selectedTerminalId) {
      return { kind: "terminal", terminalId: this.selectedTerminalId };
    }
    return null;
  }

  logsForSelectedProcess() {
    return this.selectedProcessRuntimeId
      ? (this.processLogs[this.selectedProcessRuntimeId] ?? [])
      : [];
  }

  truncatedLogCountForSelectedProcess() {
    return this.selectedProcessRuntimeId
      ? (this.processLogTruncation[this.selectedProcessRuntimeId] ?? 0)
      : 0;
  }

  async refreshProjects() {
    try {
      this.projects = await listProjects();
      if (!this.projectId && this.projects.length > 0) {
        this.projectId = this.projects[0].id;
      }
      if (
        this.projectId &&
        !this.projects.some((project) => project.id === this.projectId)
      ) {
        this.projectId = this.projects[0]?.id ?? null;
      }
    } catch (error) {
      this.setError(error);
    }
  }

  async saveProject(input: SaveProjectInput) {
    this.busy = true;
    try {
      const project = await saveProject(input);
      await this.refreshProjects();
      this.projectId = project.id;
      return project;
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async removeProject(projectId: ProjectId) {
    this.busy = true;
    try {
      await removeProject(projectId);
      if (this.projectConfig?.project.id === projectId) {
        this.projectConfig = null;
      }
      await this.refreshProjects();
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async startCurrentProject() {
    if (!this.projectId || (this.session && !this.session.stoppedAt)) {
      return;
    }
    this.busy = true;
    this.processLogs = {};
    this.processLogTruncation = {};
    try {
      this.session = await startProject(this.projectId);
      this.projectId = this.session.projectId;
      this.syncProcessSelection();
      this.#startGitPolling();
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async stopCurrentProject() {
    this.busy = true;
    try {
      await stopProject();
      this.#stopGitPolling();
      this.syncProcessSelection();
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async restartSessionProcess(processName: string) {
    this.busy = true;
    try {
      await restartProcess(processName);
      this.syncProcessSelection();
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async startSessionProcess(processName: string) {
    this.busy = true;
    try {
      this.session = await startProcess(processName);
      this.syncProcessSelection();
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async stopSessionProcess(processName: string) {
    this.busy = true;
    try {
      await stopProcess(processName);
      this.syncProcessSelection();
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async loadConfig(projectId = this.projectId, yaml?: string) {
    if (!projectId) {
      this.projectConfig = null;
      return null;
    }
    try {
      const document = await loadProjectConfig(projectId, yaml);
      if (!yaml) {
        this.projectConfig = document;
      }
      return document;
    } catch (error) {
      this.setError(error);
      throw error;
    }
  }

  async saveConfig(yaml: string, projectId = this.projectId) {
    if (!projectId) {
      return null;
    }
    this.busy = true;
    try {
      const document = await saveProjectConfig(projectId, yaml);
      this.projectConfig = document;
      return document;
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async openProjectTerminal(projectId = this.projectId, title?: string) {
    if (!projectId) {
      return null;
    }
    this.busy = true;
    try {
      const terminal = await openTerminal(projectId, title);
      this.upsertTerminal(terminal);
      this.selectedTerminalId = terminal.terminalId;
      this.selectedProcessRuntimeId = null;
      this.terminalOutput[terminal.terminalId] ??= "";
      return terminal;
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async openTitledTerminal(projectId = this.projectId) {
    if (!projectId) {
      return null;
    }
    const openCount = this.terminals.filter((t) => t.isOpen).length;
    const title = openCount === 0 ? "bash" : `bash ${openCount + 1}`;
    return this.openProjectTerminal(projectId, title);
  }

  async closeSelectedTerminal() {
    if (!this.selectedTerminalId) {
      return;
    }
    this.busy = true;
    try {
      const closed = await closeTerminal(this.selectedTerminalId);
      if (closed) {
        this.upsertTerminal(closed);
      }
      this.selectedTerminalId =
        this.terminals.find((terminal) => terminal.isOpen)?.terminalId ?? null;
    } catch (error) {
      this.setError(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }

  async writeToTerminal(data: string) {
    if (!this.selectedTerminalId) {
      return;
    }
    try {
      await writeTerminal(this.selectedTerminalId, data);
    } catch (error) {
      this.setError(error);
    }
  }

  async resizeSelectedTerminal(cols: number, rows: number) {
    if (!this.selectedTerminalId) {
      return;
    }
    try {
      await resizeTerminal(this.selectedTerminalId, cols, rows);
    } catch (error) {
      this.setError(error);
    }
  }

  clearSelectedProcessLogs() {
    if (!this.selectedProcessRuntimeId) {
      return;
    }
    this.processLogs[this.selectedProcessRuntimeId] = [];
    this.processLogTruncation[this.selectedProcessRuntimeId] = 0;
  }

  clearError() {
    this.uiError = null;
  }

  selectProcess(runtimeId: ProcessRuntimeId) {
    this.selectedProcessRuntimeId = runtimeId;
    this.selectedTerminalId = null;
  }

  selectTerminal(terminalId: TerminalSessionId) {
    this.selectedTerminalId = terminalId;
    this.selectedProcessRuntimeId = null;
  }

  syncProcessSelection() {
    const processes = this.session?.processes ?? [];
    if (processes.length === 0) {
      this.selectedProcessRuntimeId = null;
      return;
    }
    if (
      this.selectedProcessRuntimeId &&
      processes.some((process) => process.runtimeId === this.selectedProcessRuntimeId)
    ) {
      return;
    }
    this.selectedProcessRuntimeId = processes[0]?.runtimeId ?? null;
  }

  upsertTerminal(snapshot: TerminalSnapshot) {
    const next = [...this.terminals];
    const index = next.findIndex((terminal) => terminal.terminalId === snapshot.terminalId);
    if (index >= 0) {
      next[index] = snapshot;
    } else {
      next.unshift(snapshot);
    }
    this.terminals = next;
  }

  setError(error: unknown) {
    this.uiError = error instanceof Error ? error.message : String(error);
  }

  async #attachEventListeners() {
    this.#unlisteners.push(
      await listen<SessionStatusEvent>(TAURI_EVENTS.sessionSnapshot, (event) => {
        const snapshot = event.payload.snapshot;
        if (this.projectId && snapshot && snapshot.projectId !== this.projectId) {
          return;
        }
        if (snapshot) {
          this.projectId = snapshot.projectId;
        }
        this.session = event.payload.snapshot;
        this.syncProcessSelection();
      }),
    );
    this.#unlisteners.push(
      await listen<ProcessLogEvent>(TAURI_EVENTS.processLog, (event) => {
        const payload = event.payload.payload;
        if (this.session && payload.sessionId !== this.session.sessionId) {
          return;
        }
        const current = this.processLogs[payload.runtimeId] ?? [];
        const appended = [...current, payload];
        const overflow = Math.max(0, appended.length - MAX_LOG_LINES_PER_PROCESS);
        this.processLogs[payload.runtimeId] = overflow > 0 ? appended.slice(overflow) : appended;
        this.processLogTruncation[payload.runtimeId] =
          (this.processLogTruncation[payload.runtimeId] ?? 0) + overflow;
      }),
    );
    this.#unlisteners.push(
      await listen<TerminalOutputEvent>(TAURI_EVENTS.terminalOutput, (event) => {
        const payload = event.payload.payload;
        if (!this.terminals.some((terminal) => terminal.terminalId === payload.terminalId)) {
          return;
        }
        this.terminalOutput[payload.terminalId] =
          (this.terminalOutput[payload.terminalId] ?? "") + payload.chunk;
      }),
    );
    this.#unlisteners.push(
      await listen<TerminalEvent>(TAURI_EVENTS.terminalSnapshot, (event) => {
        this.upsertTerminal(event.payload.snapshot);
      }),
    );
    this.#unlisteners.push(
      await listen<RuntimeErrorEvent>(TAURI_EVENTS.runtimeError, (event) => {
        this.uiError = event.payload.message;
      }),
    );
  }

  async #applyLaunchParams() {
    if (typeof window === "undefined") {
      return;
    }
    const params = new URLSearchParams(window.location.search);
    const launchProjectId = await getLaunchProject();
    const projectId = params.get("projectId") ?? launchProjectId;
    const autorun = params.get("autorun") === "1";
    if (!projectId || !this.projects.some((project) => project.id === projectId)) {
      return;
    }
    this.projectId = projectId;
    if ((autorun || launchProjectId === projectId) && !this.session) {
      await this.startCurrentProject();
    }
  }
}

export const runtimeStore = new RuntimeStore();
