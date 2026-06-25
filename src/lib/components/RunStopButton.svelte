<script lang="ts">
  type Props = {
    active: boolean;
    busy?: boolean;
    disabled?: boolean;
    compact?: boolean;
    onRun: () => void;
    onStop: () => void;
  };

  let { active, busy = false, disabled = false, compact = false, onRun, onStop }: Props = $props();
</script>

{#if active}
  <button
    type="button"
    class="inline-flex items-center justify-center gap-1.5 border border-danger/30 bg-danger/10 font-medium text-danger transition-colors hover:bg-danger/20 disabled:cursor-not-allowed disabled:opacity-55 {compact ? 'h-5 rounded-md px-3 text-xs' : 'h-8 flex-1 rounded-md px-2 text-xs'}"
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
    class="inline-flex items-center justify-center gap-1.5 bg-accent font-medium text-canvas transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:bg-surface-hover disabled:text-text-subtle {compact ? 'h-5 rounded-md px-3 text-xs' : 'h-8 flex-1 rounded-md px-2 text-xs'}"
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
