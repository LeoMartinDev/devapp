<script lang="ts">
  import Menu from "$lib/components/ui/Menu.svelte";
  import type { MenuItem } from "$lib/components/ui/Menu.svelte";
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
    onCloseTerminal,
    onOpenTerminal,
  }: Props = $props();

  // The ⚙ menu is contextual: project actions always; process actions when a
  // process is selected; terminal actions when a terminal is selected.
  const items = $derived<MenuItem[]>(buildItems());

  function buildItems(): MenuItem[] {
    const built: MenuItem[] = [];

    if (project) {
      built.push({ label: "Edit project", onSelect: () => onEditProject(project) });
      built.push({ label: "Runtime config", onSelect: onOpenConfig, dividerAfter: true });
    }

    if (selection?.kind === "process" && selectedProcess) {
      const name = selectedProcess.name;
      built.push({ label: `Restart ${name}`, onSelect: () => onRestartProcess(name) });
      built.push({
        label: `Stop ${name}`,
        onSelect: () => onStopProcess(name),
        danger: true,
        dividerAfter: true,
      });
      built.push({ label: "Copy logs", onSelect: () => logActions?.copy(), disabled: !logActions });
      built.push({
        label: "Clear logs",
        onSelect: () => logActions?.clear(),
        disabled: !logActions,
        danger: true,
      });
    }

    if (selection?.kind === "terminal" && selectedTerminal) {
      built.push({
        label: `Close ${selectedTerminal.title}`,
        onSelect: onCloseTerminal,
        danger: true,
      });
    }

    built.push({ label: "Open terminal", onSelect: onOpenTerminal, disabled: busy });

    return built;
  }
</script>

<Menu label="Project menu" {items} />
