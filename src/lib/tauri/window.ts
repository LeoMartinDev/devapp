import { getCurrentWindow } from "@tauri-apps/api/window";

import { isTauriRuntime } from "./environment";

export function canUseTauriWindow(): boolean {
  return isTauriRuntime();
}

export async function setWindowTitle(title: string): Promise<void> {
  if (!canUseTauriWindow()) return;
  await getCurrentWindow().setTitle(title);
}

export async function startWindowDrag(): Promise<void> {
  if (!canUseTauriWindow()) return;
  await getCurrentWindow().startDragging();
}

export async function minimizeWindow(): Promise<void> {
  if (!canUseTauriWindow()) return;
  await getCurrentWindow().minimize();
}

export async function toggleWindowMaximize(): Promise<void> {
  if (!canUseTauriWindow()) return;
  await getCurrentWindow().toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  if (!canUseTauriWindow()) return;
  await getCurrentWindow().close();
}

export async function isWindowMaximized(): Promise<boolean> {
  if (!canUseTauriWindow()) return false;
  return getCurrentWindow().isMaximized();
}
