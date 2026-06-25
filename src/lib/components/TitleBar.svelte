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

  let logActions = $state<{ copy: () => void; clear: () => void } | null>(null);

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
  style:padding-left={isMac ? "74px" : "12px"}
  style:padding-right={isMac ? "8px" : "0"}
  onmousedown={onDragStart}
>
  <!-- Left: Identity + Actions -->
  <div class="titlebar-left" data-tauri-no-drag>
    <div class="min-w-0">
      <div class="truncate text-sm font-semibold text-text">{project?.name ?? "devapp"}</div>
    </div>
    {#if gitInfo?.branch || gitInfo?.worktree}
      <span class="git-badge">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3" />
          <path d="M6 6v12M18 6v12" />
        </svg>
        {gitInfo.worktree ?? gitInfo.branch}
      </span>
    {/if}

    {#if !runtimeStore.launchLocked}
      <div class="titlebar-actions">
        <RunStopButton
          active={sessionActive}
          busy={runtimeStore.busy}
          disabled={!runtimeStore.projectId}
          onRun={() => runtimeStore.startCurrentProject()}
          onStop={() => runtimeStore.stopCurrentProject()}
        />
        <IconButton label="Register project" onclick={openCreateDialog} class="text-lg leading-none">
          +
        </IconButton>
      </div>
    {/if}
  </div>

  <!-- Center: Context -->
  <div class="titlebar-center" data-tauri-no-drag>
    {#if selection?.kind === "terminal" && selectedTerminal}
      <div class="truncate text-sm text-text-muted">{selectedTerminal.title}</div>
      <div class="truncate text-[10px] text-text-subtle">{selectedTerminal.cwd}</div>
    {:else if selectedProcess}
      <div class="truncate text-sm text-text-muted">▶ {selectedProcess.name}</div>
      <div class="truncate text-[10px] text-text-subtle">{selectedProcess.status}</div>
    {:else if project}
      <div class="truncate text-[10px] text-text-subtle">{project.baseDir}</div>
    {/if}
  </div>

  <!-- Right: Tool actions -->
  <div class="titlebar-right" data-tauri-no-drag>
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
    grid-template-columns: auto 1fr auto;
    align-items: center;
    height: 38px;
    min-height: 38px;
    background: var(--color-surface, #0e0f12);
    user-select: none;
    -webkit-user-select: none;
    gap: 8px;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex-shrink: 1;
  }

  .titlebar-center {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-width: 0;
    overflow: hidden;
  }

  @media (max-width: 600px) {
    .titlebar-center {
      display: none;
    }
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .git-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--color-surface-raised, #15171b);
    font-size: 11px;
    color: var(--color-text-muted, #9aa0aa);
    flex-shrink: 0;
  }

  @media (max-width: 450px) {
    .git-badge {
      display: none;
    }
  }

  .titlebar-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }
</style>
