<script lang="ts">
  import { onMount } from "svelte";

  import AppShell from "$lib/components/AppShell.svelte";
  import ConfigEditor from "$lib/components/ConfigEditor.svelte";
  import LogViewer from "$lib/components/LogViewer.svelte";
  import ProcessList from "$lib/components/ProcessList.svelte";
  import RunStopButton from "$lib/components/RunStopButton.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import ProjectSettingsDialog from "$lib/components/ProjectSettingsDialog.svelte";
  import TerminalPane from "$lib/components/TerminalPane.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import { setWindowTitle } from "$lib/tauri/client";
  import { createShortcutRegistry } from "$lib/shortcuts/registry";
  import type { ProjectRecord } from "$lib/types";

  let detailsOpen = $state(false);
  let configOpen = $state(false);
  let editingProject = $state<ProjectRecord | null>(null);

  const project = $derived(runtimeStore.project);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);
  const selection = $derived(runtimeStore.selection);

  $effect(() => {
    const title = runtimeStore.windowTitle;
    document.title = title;
    setWindowTitle(title);
  });

  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const selectedTerminal = $derived(runtimeStore.selectedTerminal);

  const navigableItems = $derived.by(() => {
    const items: Array<
      | { kind: "process"; runtimeId: string }
      | { kind: "terminal"; terminalId: string }
    > = [];
    if (session) {
      for (const p of session.processes) {
        items.push({ kind: "process", runtimeId: p.runtimeId });
      }
    }
    for (const t of runtimeStore.terminals.filter((t) => t.isOpen)) {
      items.push({ kind: "terminal", terminalId: t.terminalId });
    }
    return items;
  });

  function navigateList(direction: 1 | -1) {
    const items = navigableItems;
    if (items.length === 0) return;

    let currentIndex = -1;
    if (selection) {
      currentIndex = items.findIndex((item) => {
        if (item.kind === "process" && selection.kind === "process") {
          return item.runtimeId === selection.runtimeId;
        }
        if (item.kind === "terminal" && selection.kind === "terminal") {
          return item.terminalId === selection.terminalId;
        }
        return false;
      });
    }

    let nextIndex = currentIndex + direction;
    if (nextIndex < 0) nextIndex = items.length - 1;
    if (nextIndex >= items.length) nextIndex = 0;

    const item = items[nextIndex];
    if (item.kind === "process") {
      runtimeStore.selectProcess(item.runtimeId);
    } else {
      runtimeStore.selectTerminal(item.terminalId);
    }
  }

  const shortcutHandler = createShortcutRegistry([
    {
      key: "Mod+Enter",
      description: "Start the current project",
      handler: () => { void runtimeStore.startCurrentProject(); },
      guard: () => !!runtimeStore.projectId && !sessionActive && !runtimeStore.busy,
    },
    {
      key: "Mod+.",
      description: "Stop the current project",
      handler: () => { void runtimeStore.stopCurrentProject(); },
      guard: () => sessionActive && !runtimeStore.busy,
    },
    {
      key: "Mod+T",
      description: "Open a new terminal",
      handler: () => { void openTerminal(); },
      guard: () => !!runtimeStore.projectId && !runtimeStore.busy,
    },
    {
      key: "Mod+J",
      description: "Select next sidebar item",
      handler: () => navigateList(1),
    },
    {
      key: "Mod+K",
      description: "Select previous sidebar item",
      handler: () => navigateList(-1),
    },
  ]);

  onMount(() => {
    void runtimeStore.init();

    const onOpenCreate = () => openCreateDialog();
    const onOpenEdit = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (detail) openEditDialog(detail);
    };
    const onOpenConfig = () => openConfigDialog();

    document.addEventListener("devapp:open-create-dialog", onOpenCreate);
    document.addEventListener("devapp:open-edit-dialog", onOpenEdit);
    document.addEventListener("devapp:open-config-dialog", onOpenConfig);

    return () => {
      void runtimeStore.teardown();
      document.removeEventListener("devapp:open-create-dialog", onOpenCreate);
      document.removeEventListener("devapp:open-edit-dialog", onOpenEdit);
      document.removeEventListener("devapp:open-config-dialog", onOpenConfig);
    };
  });

  function openCreateDialog() {
    if (runtimeStore.launchLocked) return;
    editingProject = null;
    detailsOpen = true;
  }

  function openEditDialog(openedProject: ProjectRecord) {
    if (runtimeStore.launchLocked) return;
    editingProject = openedProject;
    detailsOpen = true;
  }

  function openConfigDialog() {
    configOpen = true;
  }

  async function openTerminal() {
    const terminal = await runtimeStore.openTitledTerminal(
      session && !session.stoppedAt ? session.projectId : undefined,
    );
    if (terminal) {
      runtimeStore.selectTerminal(terminal.terminalId);
    }
  }
</script>

<svelte:head>
  <title>{runtimeStore.windowTitle}</title>
</svelte:head>

<svelte:window onkeydown={shortcutHandler} />

<TitleBar />

{#snippet processList()}
      <section class="flex min-h-0 flex-col">
        <div class="flex gap-2 px-3 pt-4 pb-3">
          <RunStopButton
            active={sessionActive}
            busy={runtimeStore.busy}
            disabled={!runtimeStore.projectId}
            onRun={() => runtimeStore.startCurrentProject()}
            onStop={() => runtimeStore.stopCurrentProject()}
          />
          <IconButton
            label="Open terminal"
            disabled={!runtimeStore.projectId || runtimeStore.busy}
            onclick={openTerminal}
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor"
              stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
          </IconButton>
        </div>
        <div class="mx-3 mb-3 border-t border-border"></div>

        <div class="min-h-0 overflow-y-auto px-3 pb-4">
          {#if runtimeStore.uiError}
            <div class="mb-3 rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
              <div class="break-words">{runtimeStore.uiError}</div>
              <Button variant="ghost" size="sm" class="mt-2 h-auto px-0 text-danger/70 hover:bg-transparent hover:text-danger" onclick={() => runtimeStore.clearError()}>
                Dismiss
              </Button>
            </div>
          {/if}

          <div class="mb-2 flex items-center justify-between px-1">
            <h2 class="text-[11px] font-semibold uppercase tracking-wider text-text-subtle">Processes</h2>
            {#if session}
              <span class="text-[11px] text-text-subtle">
                since {new Date(session.startedAt).toLocaleTimeString()}
              </span>
            {/if}
          </div>
          <ProcessList
            processes={session?.processes ?? []}
            terminals={runtimeStore.terminals}
            selectedProcessRuntimeId={runtimeStore.selectedProcessRuntimeId}
            selectedTerminalId={runtimeStore.selectedTerminalId}
            busy={runtimeStore.busy}
            onSelectProcess={(runtimeId) => runtimeStore.selectProcess(runtimeId)}
            onSelectTerminal={(terminalId) => runtimeStore.selectTerminal(terminalId)}
            onStart={(processName) => runtimeStore.startSessionProcess(processName)}
            onStop={(processName) => runtimeStore.stopSessionProcess(processName)}
            onRestart={(processName) => runtimeStore.restartSessionProcess(processName)}
            onCloseTerminal={(terminalId) => {
              runtimeStore.selectTerminal(terminalId);
              runtimeStore.closeSelectedTerminal();
            }}
          />
        </div>
      </section>
{/snippet}

<AppShell {processList}>
          {#if selection?.kind === "terminal" && selectedTerminal}
            <TerminalPane
              terminalId={selectedTerminal.terminalId}
              output={runtimeStore.terminalOutput[selectedTerminal.terminalId] ?? ""}
              onInput={(data) => runtimeStore.writeToTerminal(data)}
              onResize={(cols, rows) => runtimeStore.resizeSelectedTerminal(cols, rows)}
              onOpenTerminal={openTerminal}
            />
        {:else if selection?.kind === "process" && session}
          <LogViewer
            logs={runtimeStore.logsForSelectedProcess()}
            processName={selectedProcess?.name ?? null}
            truncatedCount={runtimeStore.truncatedLogCountForSelectedProcess()}
            onClear={() => runtimeStore.clearSelectedProcessLogs()}
            onActions={(actions) => (runtimeStore.logActions = actions)}
          />
        {:else}
          <div class="grid h-full place-items-center px-6 text-center">
            <div class="max-w-sm">
              <div class="text-sm font-semibold text-text">
                {session ? "Select a process or terminal" : "No process is running"}
              </div>
              <p class="mt-2 text-sm leading-6 text-text-subtle">
                {session
                  ? "Choose an item in the sidebar to view its logs or terminal."
                  : "Start the current project to populate the process list, or open a terminal from the project menu."}
              </p>
            </div>
          </div>
        {/if}
</AppShell>

<ProjectSettingsDialog
  open={detailsOpen}
  project={editingProject}
  onClose={() => {
    detailsOpen = false;
    editingProject = null;
  }}
  onSave={async (input) => {
    await runtimeStore.saveProject(input);
  }}
  onRemove={async (project) => {
    await runtimeStore.removeProject(project.id);
  }}
  launchLocked={runtimeStore.launchLocked}
/>

<ConfigEditor
  open={configOpen}
  {project}
  onClose={() => {
    configOpen = false;
  }}
/>
