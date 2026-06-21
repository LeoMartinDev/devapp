<script lang="ts">
  import { onMount } from "svelte";

  import AppShell from "$lib/components/AppShell.svelte";
  import ConfigEditor from "$lib/components/ConfigEditor.svelte";
  import LogViewer from "$lib/components/LogViewer.svelte";
  import ProcessList from "$lib/components/ProcessList.svelte";
  import ProjectSettingsDialog from "$lib/components/ProjectSettingsDialog.svelte";
  import TerminalPane from "$lib/components/TerminalPane.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import type { ProjectRecord } from "$lib/types";

  let detailsOpen = $state(false);
  let configOpen = $state(false);
  let editingProject = $state<ProjectRecord | null>(null);
  let runtimeView = $state<"logs" | "terminal">("logs");
  const runtimeViewOptions: { value: "logs" | "terminal"; label: string }[] = [
    { value: "logs", label: "Logs" },
    { value: "terminal", label: "Terminal" },
  ];

  const project = $derived(runtimeStore.project);
  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);

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

  async function openTerminalAndShow() {
    await runtimeStore.openProjectTerminal(session && !session.stoppedAt ? session.projectId : undefined);
    runtimeView = "terminal";
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
            <div class="truncate text-[11px] text-text-subtle">
              {project?.baseDir ?? "No project registered"}
            </div>
          </div>
          <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
            +
          </IconButton>
        </div>

        <div class="mt-2.5 flex items-center gap-1.5">
          {#if sessionActive}
            <button
              type="button"
              class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-md border border-danger/30 bg-danger/10 px-2 text-xs font-medium text-danger transition-colors hover:bg-danger/20 disabled:cursor-not-allowed disabled:opacity-55"
              onclick={() => runtimeStore.stopCurrentProject()}
              disabled={runtimeStore.busy}
              aria-label="Stop current run"
              title="Stop current run"
            >
              <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <rect x="6" y="6" width="12" height="12" rx="1.5" />
              </svg>
              Stop
            </button>
          {:else}
            <button
              type="button"
              class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-md bg-accent px-2 text-xs font-medium text-canvas transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:bg-surface-hover disabled:text-text-subtle"
              onclick={() => runtimeStore.startCurrentProject()}
              disabled={!runtimeStore.projectId || runtimeStore.busy}
              aria-label="Run project"
              title="Run project"
            >
              <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M8 5v14l11-7z" />
              </svg>
              Run
            </button>
          {/if}
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
          selectedRuntimeId={runtimeStore.selectedProcessRuntimeId}
          busy={runtimeStore.busy}
          onSelect={(runtimeId) => {
            runtimeStore.selectProcess(runtimeId);
            runtimeView = "logs";
          }}
          onRestart={(processName) => runtimeStore.restartSessionProcess(processName)}
          onStop={(processName) => runtimeStore.stopSessionProcess(processName)}
        />
      </section>
{/snippet}

{#snippet contentHeader()}
      <header class="flex h-12 min-w-0 items-center justify-between gap-4 border-b border-border px-4">
        <div class="min-w-0">
          <div class="truncate text-sm font-semibold text-text">
            {runtimeView === "logs"
              ? (selectedProcess?.name ?? "Process logs")
              : (runtimeStore.selectedTerminal?.title ?? "Terminal")}
          </div>
          <div class="truncate text-[11px] text-text-subtle">
            {runtimeView === "logs"
              ? (project?.baseDir ?? "No project selected")
              : (runtimeStore.selectedTerminal?.cwd ?? project?.baseDir ?? "No terminal opened")}
          </div>
        </div>

        <div class="flex shrink-0 items-center gap-1">
          <IconButton
            label="Open project terminal"
            onclick={openTerminalAndShow}
            disabled={!runtimeStore.projectId || runtimeStore.busy}
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
          </IconButton>
          <IconButton
            label="Open runtime settings"
            onclick={() => project && openConfigDialog()}
            disabled={!project}
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
          </IconButton>
          <IconButton
            label="Edit project details"
            onclick={() => project && openEditDialog(project)}
            disabled={!project}
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
            </svg>
          </IconButton>
          <div class="mx-1 h-5 w-px bg-border"></div>
          <SegmentedControl
            value={runtimeView}
            options={runtimeViewOptions}
            onChange={(value) => (runtimeView = value as "logs" | "terminal")}
          />
        </div>
      </header>
{/snippet}

<AppShell
  {sidebarHeader}
  {processList}
  {contentHeader}
>
        {#if runtimeView === "logs"}
          {#if session}
            <LogViewer
              logs={runtimeStore.logsForSelectedProcess()}
              processName={selectedProcess?.name ?? null}
              truncatedCount={runtimeStore.truncatedLogCountForSelectedProcess()}
              onClear={() => runtimeStore.clearSelectedProcessLogs()}
            />
          {:else}
            <div class="grid h-full place-items-center px-6 text-center">
              <div class="max-w-sm">
                <div class="text-sm font-semibold text-text">No process is running</div>
                <p class="mt-2 text-sm leading-6 text-text-subtle">
                  Start the current project to populate the process list and inspect logs here.
                </p>
              </div>
            </div>
          {/if}
        {:else}
          <TerminalPane
            terminalId={runtimeStore.selectedTerminalId}
            title={runtimeStore.selectedTerminal?.title ?? null}
            output={runtimeStore.selectedTerminalId
              ? (runtimeStore.terminalOutput[runtimeStore.selectedTerminalId] ?? "")
              : ""}
            onInput={(data) => runtimeStore.writeToTerminal(data)}
            onResize={(cols, rows) => runtimeStore.resizeSelectedTerminal(cols, rows)}
            onClose={() => runtimeStore.closeSelectedTerminal()}
          />
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
