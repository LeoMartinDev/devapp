<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { canUseTauriApis } from "$lib/tauri/environment";

  let isMaximized = $state(false);
  let platform = $state<"win32" | "linux">("linux");
  const isTauri = canUseTauriApis();

  const appWindow = isTauri ? getCurrentWindow() : null;

  $effect(() => {
    if (!appWindow) {
      return;
    }
    // Detect platform from navigator
    const p = navigator.platform.toLowerCase();
    platform = p.includes("win") ? "win32" : "linux";

    // Check initial maximized state
    void appWindow.isMaximized().then((v) => (isMaximized = v));
  });
</script>

{#if appWindow}
  <div class="flex items-center" data-tauri-no-drag>
    <button
      aria-label="Minimize"
      title="Minimize"
      class="window-control"
      onclick={() => appWindow.minimize()}
    >
      {#if platform === "win32"}
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="0" y="4.5" width="10" height="1" fill="currentColor" />
        </svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="1" y="5" width="8" height="1" fill="currentColor" />
        </svg>
      {/if}
    </button>

    <button
      aria-label={isMaximized ? "Restore" : "Maximize"}
      title={isMaximized ? "Restore" : "Maximize"}
      class="window-control"
      onclick={() => {
        void appWindow.toggleMaximize();
        isMaximized = !isMaximized;
      }}
    >
      {#if platform === "win32"}
        {#if isMaximized}
          <!-- Win11 restore icon -->
          <svg width="10" height="10" viewBox="0 0 10 10">
            <rect x="2" y="0.5" width="7" height="7" rx="0.5" fill="none" stroke="currentColor" stroke-width="1" />
            <rect x="0.5" y="2" width="7" height="7" rx="0.5" fill="#0e0f12" stroke="currentColor" stroke-width="1" />
          </svg>
        {:else}
          <!-- Win11 maximize icon -->
          <svg width="10" height="10" viewBox="0 0 10 10">
            <rect x="0.5" y="0.5" width="9" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
        {/if}
      {:else}
        {#if isMaximized}
          <!-- Linux restore icon -->
          <svg width="10" height="10" viewBox="0 0 10 10">
            <rect x="2" y="0.5" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1" />
            <rect x="0.5" y="2" width="7" height="7" rx="1" fill="#0e0f12" stroke="currentColor" stroke-width="1" />
          </svg>
        {:else}
          <!-- Linux maximize icon -->
          <svg width="10" height="10" viewBox="0 0 10 10">
            <rect x="1" y="1" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
        {/if}
      {/if}
    </button>

    <button
      aria-label="Close"
      title="Close"
      class="window-control window-control-close"
      onclick={() => appWindow.close()}
    >
      <svg width="10" height="10" viewBox="0 0 10 10">
        <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
    </button>
  </div>
{/if}

<style>
  .window-control {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 38px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--color-text-muted, #9aa0aa);
    cursor: default;
    -webkit-app-region: no-drag;
  }
  .window-control:hover {
    background: var(--color-surface-hover, #1b1d22);
    color: var(--color-text, #e7e9ee);
  }
  .window-control-close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
