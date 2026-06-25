<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import ProjectMenu from "$lib/components/ProjectMenu.svelte";
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
    grid-template-columns: 1fr auto;
    align-items: center;
    height: 36px;
    min-height: 36px;
    background: var(--color-canvas, #08090b);
    user-select: none;
    -webkit-user-select: none;
    gap: 8px;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
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
</style>
