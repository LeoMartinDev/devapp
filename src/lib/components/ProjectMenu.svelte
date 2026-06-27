<script lang="ts">
  import Menu from "$lib/components/ui/Menu.svelte";
  import type { MenuItem } from "$lib/components/ui/Menu.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import {
    canShowNativeMenu,
    showNativeMenu,
    type NativeMenuItem,
  } from "$lib/tauri/nativeMenu";
  import type { Selection } from "$lib/stores/runtime.svelte";
  import type {
    ProcessSnapshot,
    ProjectRecord,
    TerminalSnapshot,
  } from "$lib/types";

  type LogActions = { copy: () => void; clear: () => void };

  type Props = {
    project: ProjectRecord | null;
    selection: Selection;
    selectedProcess: ProcessSnapshot | null;
    selectedTerminal: TerminalSnapshot | null;
    busy: boolean;
    logActions: LogActions | null;
    onEditProject: (project: ProjectRecord) => void;
    onOpenConfig: () => void;
    onRestartProcess: (name: string) => void;
    onStopProcess: (name: string) => void;
    launchLocked: boolean;
    onCloseTerminal: () => void;
    onOpenTerminal: () => void;
  };

  let {
    project,
    selection,
    selectedProcess,
    selectedTerminal,
    busy,
    logActions,
    onEditProject,
    onOpenConfig,
    onRestartProcess,
    onStopProcess,
    launchLocked,
    onCloseTerminal,
    onOpenTerminal,
  }: Props = $props();

  const nativeMenuAvailable = canShowNativeMenu();

  // The ⚙ menu is contextual: project actions always; process actions when a
  // process is selected; terminal actions when a terminal is selected.
  const items = $derived<MenuItem[]>(buildItems());
  const nativeItems = $derived<NativeMenuItem[]>(buildNativeItems());

  function buildItems(): MenuItem[] {
    const built: MenuItem[] = [];

    if (project) {
      if (!launchLocked) {
        built.push({ label: "Edit project", icon: "edit", onSelect: () => onEditProject(project) });
      }
      built.push({ label: "Runtime config", icon: "config", onSelect: onOpenConfig, dividerAfter: true });
    }

    if (selection?.kind === "process" && selectedProcess) {
      const name = selectedProcess.name;
      built.push({ label: `Restart ${name}`, icon: "restart", onSelect: () => onRestartProcess(name) });
      built.push({
        label: `Stop ${name}`,
        icon: "stop",
        onSelect: () => onStopProcess(name),
        danger: true,
        dividerAfter: true,
      });
      built.push({ label: "Copy logs", icon: "copy", onSelect: () => logActions?.copy(), disabled: !logActions });
      built.push({
        label: "Clear logs",
        icon: "clear",
        onSelect: () => logActions?.clear(),
        disabled: !logActions,
        danger: true,
      });
    }

    if (selection?.kind === "terminal" && selectedTerminal) {
      built.push({
        label: `Close ${selectedTerminal.title}`,
        icon: "close",
        onSelect: onCloseTerminal,
        danger: true,
      });
    }

    built.push({ label: "Open terminal", icon: "terminal", onSelect: onOpenTerminal, disabled: busy });

    return built;
  }

  function buildNativeItems(): NativeMenuItem[] {
    return items.map((item) => ({
      label: item.label,
      enabled: !item.disabled,
      danger: item.danger,
      dividerAfter: item.dividerAfter,
      action: item.onSelect,
    }));
  }

  async function openNativeMenu(event: MouseEvent) {
    const trigger = event.currentTarget as HTMLElement | null;
    const rect = trigger?.getBoundingClientRect();
    const hasMeasuredRect = !!rect && (rect.width > 0 || rect.height > 0);
    const x = hasMeasuredRect ? Math.round(rect.left) : Math.round(event.clientX);
    const y = hasMeasuredRect ? Math.round(rect.bottom) : Math.round(event.clientY);
    await showNativeMenu(nativeItems, { x, y });
  }
</script>

{#if nativeMenuAvailable}
  <IconButton label="Project menu" onclick={(event) => { void openNativeMenu(event); }}>
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  </IconButton>
{:else}
  <Menu label="Project menu" {items} />
{/if}
