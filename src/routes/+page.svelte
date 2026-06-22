<script lang="ts">
  import { onMount } from "svelte";

  import AppShell from "$lib/components/AppShell.svelte";
  import ConfigEditor from "$lib/components/ConfigEditor.svelte";
  import LogViewer from "$lib/components/LogViewer.svelte";
  import ProcessList from "$lib/components/ProcessList.svelte";
  import ProjectMenu from "$lib/components/ProjectMenu.svelte";
  import ProjectSettingsDialog from "$lib/components/ProjectSettingsDialog.svelte";
  import RunStopButton from "$lib/components/RunStopButton.svelte";
  import TerminalPane from "$lib/components/TerminalPane.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import type { ProjectRecord } from "$lib/types";

  let detailsOpen = $state(false);
  let configOpen = $state(false);
  let editingProject = $state<ProjectRecord | null>(null);

  // Externally-callable log actions, registered by LogViewer.
  let logActions = $state<{ copy: () => void; clear: () => void } | null>(null);

  const project = $derived(runtimeStore.project);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);
  const selection = $derived(runtimeStore.selection);

  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const selectedTerminal = $derived(runtimeStore.selectedTerminal);

  onMount(() => {
    void runtimeStore.init();
    return () => {
      void runtimeStore.teardown();
    };
  });

  function openCreateDialog() {
    editingProject = null;
    detailsOpen = true;
  }

  function openEditDialog(openedProject: ProjectRecord) {
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
  <title>devapp</title>
</svelte:head>

{#snippet sidebarHeader()}
      <div class="border-b border-border px-4 py-3">
        <div class="flex items-center justify-between gap-2">
          <div class="min-w-0">
            <div class="truncate text-sm font-semibold text-text">{project?.name ?? "devapp"}</div>
          </div>
          <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
            +
          </IconButton>
        </div>

        <div class="mt-2.5 flex items-center gap-1.5">
          <RunStopButton
            active={sessionActive}
            busy={runtimeStore.busy}
            disabled={!runtimeStore.projectId}
            onRun={() => runtimeStore.startCurrentProject()}
            onStop={() => runtimeStore.stopCurrentProject()}
          />
        </div>
      </div>
{/snippet}

{#snippet processList()}
      <section class="min-h-0 overflow-y-auto px-3 pb-4 pt-4">
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
      </section>
{/snippet}

{#snippet contentHeader()}
      <header class="flex h-12 min-w-0 items-center justify-between gap-4 border-b border-border px-4">
        <div class="min-w-0">
          {#if selection?.kind === "terminal" && selectedTerminal}
            <div class="truncate text-sm font-semibold text-text">{selectedTerminal.title}</div>
            <div class="truncate text-[11px] text-text-subtle">{selectedTerminal.cwd}</div>
          {:else if selectedProcess}
            <div class="truncate text-sm font-semibold text-text">{selectedProcess.name}</div>
            <div class="truncate text-[11px] text-text-subtle">{selectedProcess.status}</div>
          {:else}
            <div class="truncate text-sm font-semibold text-text">
              {project?.name ?? "devapp"}
            </div>
            <div class="truncate text-[11px] text-text-subtle">
              {project?.baseDir ?? "No project registered"}
            </div>
          {/if}
        </div>

        <div class="flex shrink-0 items-center gap-1">
          <ProjectMenu
            {project}
            {selection}
            {selectedProcess}
            {selectedTerminal}
            busy={runtimeStore.busy}
            {logActions}
            onEditProject={openEditDialog}
            onOpenConfig={openConfigDialog}
            onRestartProcess={(name) => runtimeStore.restartSessionProcess(name)}
            onStopProcess={(name) => runtimeStore.stopSessionProcess(name)}
            onCloseTerminal={() => runtimeStore.closeSelectedTerminal()}
            onOpenTerminal={openTerminal}
          />
        </div>
      </header>
{/snippet}

<AppShell
  {sidebarHeader}
  {processList}
  {contentHeader}
>
        {#if selection?.kind === "terminal" && selectedTerminal}
          <TerminalPane
            terminalId={selectedTerminal.terminalId}
            output={runtimeStore.terminalOutput[selectedTerminal.terminalId] ?? ""}
            onInput={(data) => runtimeStore.writeToTerminal(data)}
            onResize={(cols, rows) => runtimeStore.resizeSelectedTerminal(cols, rows)}
          />
        {:else if selection?.kind === "process" && session}
          <LogViewer
            logs={runtimeStore.logsForSelectedProcess()}
            processName={selectedProcess?.name ?? null}
            truncatedCount={runtimeStore.truncatedLogCountForSelectedProcess()}
            onClear={() => runtimeStore.clearSelectedProcessLogs()}
            onActions={(actions) => (logActions = actions)}
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
/>

<ConfigEditor
  open={configOpen}
  {project}
  onClose={() => {
    configOpen = false;
  }}
/>
