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

  const runBase =
    "inline-flex items-center justify-center border border-emerald-500/50 bg-emerald-500/10 text-emerald-500 transition-colors hover:bg-emerald-500/20 disabled:cursor-not-allowed";
  const stopBase =
    "inline-flex items-center justify-center border border-danger/30 bg-danger/10 text-danger transition-colors hover:bg-danger/20 disabled:cursor-not-allowed";
</script>

{#if compact}
  <span class="inline-flex items-center gap-1">
    <button
      type="button"
      class="{runBase} h-5 w-5 rounded p-px {active ? 'opacity-30' : ''}"
      onclick={onRun}
      disabled={disabled || busy || active}
      aria-label="Run project"
      title="Run project"
    >
      <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M8 5v14l11-7z" />
      </svg>
    </button>
    <button
      type="button"
      class="{stopBase} h-5 w-5 rounded p-px {active ? '' : 'opacity-30'}"
      onclick={onStop}
      disabled={busy || !active}
      aria-label="Stop current run"
      title="Stop current run"
    >
      <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <rect x="6" y="6" width="12" height="12" rx="1.5" />
      </svg>
    </button>
  </span>
{:else if active}
  <button
    type="button"
    class="{stopBase} h-8 rounded-md px-2 text-xs disabled:opacity-55"
    onclick={onStop}
    disabled={busy}
    aria-label="Stop current run"
    title="Stop current run"
  >
    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <rect x="6" y="6" width="12" height="12" rx="1.5" />
    </svg>
    <span>Stop</span>
  </button>
{:else}
  <button
    type="button"
    class="{runBase} h-8 rounded-md px-2 text-xs disabled:border-border disabled:text-text-subtle disabled:bg-transparent"
    onclick={onRun}
    disabled={disabled || busy}
    aria-label="Run project"
    title="Run project"
  >
    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M8 5v14l11-7z" />
    </svg>
    <span>Run</span>
  </button>
{/if}
