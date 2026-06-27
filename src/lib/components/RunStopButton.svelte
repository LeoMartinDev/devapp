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

  const runClass =
    "inline-flex items-center justify-center border border-emerald-500/50 bg-emerald-500/10 text-emerald-500 transition-colors hover:bg-emerald-500/20 disabled:cursor-not-allowed disabled:border-border disabled:text-text-subtle disabled:bg-transparent";
  const stopClass =
    "inline-flex items-center justify-center border border-danger/30 bg-danger/10 text-danger transition-colors hover:bg-danger/20 disabled:cursor-not-allowed disabled:opacity-55";
  const busyClass =
    "inline-flex items-center justify-center border border-warning/30 bg-warning/10 text-warning cursor-not-allowed";
  const compactClass = "h-4 w-4 rounded-[4px] p-0";
</script>

{#if busy}
  <span
    class="{busyClass} {compact ? compactClass : 'h-8 rounded-md px-2 text-xs'}"
    aria-label="Busy"
    title="Operation in progress"
  >
    <svg class="animate-spin" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
      <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
      <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
    </svg>
    {#if !compact}<span>Busy</span>{/if}
  </span>
{:else if active}
  <button
    type="button"
    class="{stopClass} {compact ? compactClass : 'h-8 rounded-md px-2 text-xs'}"
    onclick={onStop}
    disabled={busy}
    aria-label="Stop current run"
    title="Stop current run"
  >
    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <rect x="6" y="6" width="12" height="12" rx="1.5" />
    </svg>
    {#if !compact}<span>Stop</span>{/if}
  </button>
{:else}
  <button
    type="button"
    class="{runClass} {compact ? compactClass : 'h-8 rounded-md px-2 text-xs'}"
    onclick={onRun}
    disabled={disabled || busy}
    aria-label="Run project"
    title="Run project"
  >
    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M8 5v14l11-7z" />
    </svg>
    {#if !compact}<span>Run</span>{/if}
  </button>
{/if}
