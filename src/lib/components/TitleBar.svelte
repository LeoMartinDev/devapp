<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import ProjectMenu from "$lib/components/ProjectMenu.svelte";
  import RunStopButton from "$lib/components/RunStopButton.svelte";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";

  const isMac = navigator.platform.toLowerCase().includes("mac");

  const project = $derived(runtimeStore.project);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);
  const selection = $derived(runtimeStore.selection);
  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const selectedTerminal = $derived(runtimeStore.selectedTerminal);
  const gitInfo = $derived(runtimeStore.gitInfo);

  const logActions = $derived(runtimeStore.logActions);

  function onDragStart(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-tauri-no-drag]")) return;
    void getCurrentWindow().startDragging();
  }

  async function openTerminal() {
    const terminal = await runtimeStore.openTitledTerminal(
      sessionActive ? session?.projectId : undefined,
    );
    if (terminal) {
      runtimeStore.selectTerminal(terminal.terminalId);
    }
  }

  function openCreateDialog() {
    document.dispatchEvent(new CustomEvent("devapp:open-create-dialog"));
  }

  function openEditDialog(p: typeof project) {
    if (!p) return;
    document.dispatchEvent(new CustomEvent("devapp:open-edit-dialog", { detail: p }));
  }

  function openConfigDialog() {
    document.dispatchEvent(new CustomEvent("devapp:open-config-dialog"));
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header
  class="titlebar"
  data-tauri-drag-region
  style:padding-left={isMac ? "74px" : "16px"}
  style:padding-right={isMac ? "8px" : "8px"}
  onmousedown={onDragStart}
>
  <div class="titlebar-left" data-tauri-no-drag></div>

  <div class="titlebar-center" data-tauri-no-drag>
    <div class="project-chip">
      <span class="project-chip-name">{project?.name ?? "devapp"}</span>
      {#if project?.baseDir}
        <span class="project-chip-dir">{project.baseDir}</span>
      {/if}
      {#if gitInfo?.branch || gitInfo?.worktree}
        <span class="project-chip-git">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M6 6v12M18 6v12" />
          </svg>
          {gitInfo.worktree ?? gitInfo.branch}
        </span>
      {/if}
      <span class="project-chip-sep"></span>
      <span class="project-chip-action">
        <RunStopButton
          active={sessionActive}
          busy={runtimeStore.busy}
          disabled={!runtimeStore.projectId}
          compact
          onRun={() => runtimeStore.startCurrentProject()}
          onStop={() => runtimeStore.stopCurrentProject()}
        />
      </span>
    </div>
  </div>

  <div class="titlebar-right" data-tauri-no-drag>
    {#if !runtimeStore.launchLocked}
      <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
        +
      </IconButton>
    {/if}
    <ProjectMenu
      {project}
      {selection}
      {selectedProcess}
      {selectedTerminal}
      busy={runtimeStore.busy}
      {logActions}
      launchLocked={runtimeStore.launchLocked}
      onEditProject={(p) => openEditDialog(p)}
      onOpenConfig={openConfigDialog}
      onRestartProcess={(name) => runtimeStore.restartSessionProcess(name)}
      onStopProcess={(name) => runtimeStore.stopSessionProcess(name)}
      onCloseTerminal={() => runtimeStore.closeSelectedTerminal()}
      onOpenTerminal={openTerminal}
    />

    {#if !isMac}
      <WindowControls />
    {/if}
  </div>
</header>

<style>
  .titlebar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    height: 44px;
    background: var(--color-canvas, #08090b);
    border-bottom: 1px solid var(--color-border, #ffffff14);
    user-select: none;
    -webkit-user-select: none;
    gap: 8px;
    overflow: hidden;
  }

  .titlebar-left {
    min-width: 0;
  }

  .titlebar-center {
    display: flex;
    justify-content: center;
    min-width: 0;
    overflow: hidden;
  }

  .project-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    max-width: 100%;
    padding: 6px 12px;
    border-radius: 7px;
    background: var(--color-surface, #0e0f12);
    border: 1px solid var(--color-border, #ffffff14);
    min-width: 0;
    line-height: 1;
  }

  .project-chip-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text, #e7e9ee);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
  }

  .project-chip-dir {
    font-size: 11px;
    color: var(--color-text-subtle, #5d636e);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
    flex-shrink: 1;
  }

  .project-chip-git {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--color-surface-hover, #1b1d22);
    font-size: 11px;
    color: var(--color-text-muted, #9aa0aa);
    flex-shrink: 0;
  }

  .project-chip-sep {
    align-self: stretch;
    width: 1px;
    background: var(--color-border, #ffffff14);
    flex-shrink: 0;
  }

  .project-chip-action {
    display: flex;
    flex-shrink: 0;
    width: 20px;
    justify-content: center;
  }

  @media (max-width: 500px) {
    .project-chip-dir {
      display: none;
    }
  }

  @media (max-width: 400px) {
    .project-chip-git {
      display: none;
    }
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    justify-self: end;
    gap: 2px;
    flex-shrink: 0;
  }
</style>
