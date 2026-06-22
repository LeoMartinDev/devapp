<script lang="ts">
  type Props = {
    /** True when a session is running (show Stop), false otherwise (show Run). */
    active: boolean;
    busy?: boolean;
    disabled?: boolean;
    onRun: () => void;
    onStop: () => void;
  };

  let { active, busy = false, disabled = false, onRun, onStop }: Props = $props();
</script>

{#if active}
  <button
    type="button"
    class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-md border border-danger/30 bg-danger/10 px-2 text-xs font-medium text-danger transition-colors hover:bg-danger/20 disabled:cursor-not-allowed disabled:opacity-55"
    onclick={onStop}
    disabled={busy}
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
    onclick={onRun}
    disabled={disabled || busy}
    aria-label="Run project"
    title="Run project"
  >
    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M8 5v14l11-7z" />
    </svg>
    Run
  </button>
{/if}
