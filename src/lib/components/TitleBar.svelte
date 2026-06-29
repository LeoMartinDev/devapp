<script lang="ts">
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import ProjectMenu from "$lib/components/ProjectMenu.svelte";
  import RunStopButton from "$lib/components/RunStopButton.svelte";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import { startWindowDrag } from "$lib/tauri/window";

  const isMac = navigator.platform.toLowerCase().includes("mac");

  const project = $derived(runtimeStore.project);
  const session = $derived(runtimeStore.session);
  const sessionActive = $derived(!!session && !session.stoppedAt);
  const selection = $derived(runtimeStore.selection);
  const selectedProcess = $derived(runtimeStore.selectedProcess);
  const selectedTerminal = $derived(runtimeStore.selectedTerminal);
  const gitInfo = $derived(runtimeStore.gitInfo);

  const logActions = $derived(runtimeStore.logActions);
  const projectPath = $derived(project?.baseDir ?? gitInfo?.displayPath ?? null);
  const projectPathTitle = $derived(projectPath);

  function onDragStart(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-tauri-no-drag]")) return;
    void startWindowDrag();
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
      {#if projectPath}
        <span class="project-chip-dir-wrap">
          <span class="project-chip-dir" title={projectPathTitle ?? undefined}>{projectPath}</span>
        </span>
      {/if}
      {#if gitInfo?.branch || gitInfo?.worktree}
        <span class="project-chip-git">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M6 6v12M18 6v12" />
          </svg>
          <span class="project-chip-git-label">{gitInfo.worktree ?? gitInfo.branch}</span>
        </span>
      {/if}
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
    height: 38px;
    min-height: 38px;
    background: var(--color-canvas, #08090b);
    box-shadow: inset 0 -1px 0 var(--color-border, #ffffff14);
    user-select: none;
    -webkit-user-select: none;
    gap: 8px;
  }

  .titlebar-left {
    min-width: 0;
  }

  .titlebar-center {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-width: 0;
    overflow: hidden;
  }

  .project-chip {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    align-self: center;
    column-gap: 5px;
    position: relative;
    width: min(360px, calc(100vw - 240px));
    max-width: 360px;
    height: 24px;
    padding: 0 5px 0 8px;
    border-radius: 5px;
    background: var(--color-surface, #0e0f12);
    border: 1px solid var(--color-border, #ffffff14);
    min-width: 0;
    line-height: 1;
    transform: translateY(-1px);
  }

  .project-chip-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text, #e7e9ee);
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-chip-dir-wrap {
    display: flex;
    justify-content: center;
    position: absolute;
    left: 50%;
    top: 50%;
    z-index: 0;
    width: min(10.5rem, calc(100% - 12.5rem));
    min-width: 6rem;
    overflow: hidden;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .project-chip-dir {
    display: block;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    color: var(--color-text-muted, #9aa0aa);
    letter-spacing: 0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: center;
    unicode-bidi: plaintext;
  }

  .project-chip-git {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    min-width: 0;
    max-width: 6.5rem;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--color-surface-hover, #1b1d22);
    font-size: 10px;
    color: var(--color-text-muted, #9aa0aa);
    flex-shrink: 0;
    z-index: 1;
  }

  .project-chip-git-label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-chip-action {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding-left: 4px;
    border-left: 1px solid var(--color-border, #ffffff14);
    z-index: 1;
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
