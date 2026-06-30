import { invoke } from "$lib/tauri/transport";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { canUseTauriApis } from "$lib/tauri/environment";

import type {
  GitInfo,
  ProjectConfigDocument,
  ProjectId,
  ProjectRecord,
  ProjectSource,
  RunSessionSnapshot,
  TerminalSnapshot,
} from "$lib/types";

export const TAURI_EVENTS = {
  sessionSnapshot: "session-snapshot",
  processLog: "process-log",
  terminalOutput: "terminal-output",
  terminalSnapshot: "terminal-snapshot",
  runtimeError: "runtime-error",
} as const;

export type SaveProjectInput = {
  id?: ProjectId;
  name: string;
  baseDir: string;
  configSource?: ProjectSource;
};

export interface LaunchProjectInfo {
  projectId: string | null;
  locked: boolean;
  error?: string | null;
}

export async function getLaunchProject(): Promise<LaunchProjectInfo> {
  return invoke<LaunchProjectInfo>("get_launch_project");
}

export async function listProjects(): Promise<ProjectRecord[]> {
  return invoke<ProjectRecord[]>("list_projects");
}

export async function saveProject(input: SaveProjectInput): Promise<ProjectRecord> {
  return invoke<ProjectRecord>("save_project", { request: input });
}

export async function removeProject(projectId: ProjectId): Promise<void> {
  return invoke("remove_project", { projectId });
}

export async function loadProjectConfig(
  projectId: ProjectId,
  yaml?: string,
): Promise<ProjectConfigDocument> {
  return invoke<ProjectConfigDocument>("load_project_config", {
    request: { projectId, yaml },
  });
}

export async function saveProjectConfig(
  projectId: ProjectId,
  yaml: string,
): Promise<ProjectConfigDocument> {
  return invoke<ProjectConfigDocument>("save_project_config", {
    request: { projectId, yaml },
  });
}

export async function startProject(projectId: ProjectId): Promise<RunSessionSnapshot> {
  return invoke<RunSessionSnapshot>("start_project", {
    request: { projectId },
  });
}

export async function stopProject(): Promise<RunSessionSnapshot | null> {
  return invoke<RunSessionSnapshot | null>("stop_project");
}

export async function restartProcess(processName: string): Promise<RunSessionSnapshot | null> {
  return invoke<RunSessionSnapshot | null>("restart_process", {
    request: { processName },
  });
}

export async function startProcess(
  processName: string,
): Promise<RunSessionSnapshot | null> {
  return invoke<RunSessionSnapshot | null>("start_process", {
    request: { processName },
  });
}

export async function stopProcess(processName: string): Promise<RunSessionSnapshot | null> {
  return invoke<RunSessionSnapshot | null>("stop_process", {
    request: { processName },
  });
}

export async function getSessionSnapshot(): Promise<RunSessionSnapshot | null> {
  return invoke<RunSessionSnapshot | null>("get_session_snapshot");
}

export async function openTerminal(
  projectId: ProjectId,
  title?: string,
): Promise<TerminalSnapshot> {
  return invoke<TerminalSnapshot>("open_terminal", {
    request: { projectId, title },
  });
}

export async function writeTerminal(terminalId: string, data: string): Promise<void> {
  return invoke("write_terminal", {
    request: { terminalId, data },
  });
}

export async function resizeTerminal(
  terminalId: string,
  cols: number,
  rows: number,
): Promise<void> {
  return invoke("resize_terminal", {
    request: { terminalId, cols, rows },
  });
}

export async function closeTerminal(terminalId: string): Promise<TerminalSnapshot | null> {
  return invoke<TerminalSnapshot | null>("close_terminal", {
    request: { terminalId },
  });
}

export async function getGitInfo(baseDir: string): Promise<GitInfo> {
  return invoke<GitInfo>("get_git_info", { baseDir });
}

export function setWindowTitle(title: string): Promise<void> {
  if (!canUseTauriApis()) {
    return Promise.resolve();
  }
  return getCurrentWindow().setTitle(title);
}
