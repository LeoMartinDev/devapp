<script lang="ts">
  import StatusDot from "$lib/components/ui/StatusDot.svelte";
  import type { ProcessRuntimeId, ProcessSnapshot, ProcessStatus } from "$lib/types";

  type Props = {
    processes: ProcessSnapshot[];
    selectedRuntimeId: ProcessRuntimeId | null;
    busy?: boolean;
    onSelect: (runtimeId: ProcessRuntimeId) => void;
    onRestart: (processName: string) => void;
    onStop: (processName: string) => void;
  };

  let { processes, selectedRuntimeId, busy = false, onSelect, onRestart, onStop }: Props = $props();

  const restartDisabledStatuses = new Set<ProcessStatus>([
    "pending",
    "blocked",
    "starting",
    "stopping",
  ]);
  const stopDisabledStatuses = new Set<ProcessStatus>(["succeeded", "failed", "stopped"]);

  function canRestart(status: ProcessStatus) {
    return !busy && !restartDisabledStatuses.has(status);
  }

  function canStop(status: ProcessStatus) {
    return !busy && !stopDisabledStatuses.has(status);
  }
</script>

<div class="grid gap-0.5">
  {#if processes.length === 0}
    <div
      class="rounded-lg border border-dashed border-border px-3 py-6 text-center text-xs leading-5 text-text-subtle"
    >
      No process loaded
    </div>
  {:else}
    {#each processes as process (process.runtimeId)}
      {@const selected = process.runtimeId === selectedRuntimeId}
      <div
        class={`group relative flex items-center gap-2.5 rounded-md px-3 py-1.5 transition-colors ${
          selected ? "bg-surface-raised" : "hover:bg-surface-hover"
        }`}
      >
        <!-- active indicator — accent left bar -->
        <span
          class={`absolute left-0 top-1/2 h-4 -translate-y-1/2 rounded-r-full transition-all ${
            selected ? "w-0.5 bg-accent opacity-100" : "w-0 opacity-0"
          }`}
        ></span>

        <button
          type="button"
          class="grid min-w-0 flex-1 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2.5 text-left"
          aria-current={selected ? "true" : undefined}
          onclick={() => onSelect(process.runtimeId)}
        >
          <StatusDot status={process.status} />
          <span class="min-w-0">
            <span class="block truncate text-[13px] font-medium text-text">{process.name}</span>
            <span class="block truncate text-[11px] text-text-subtle">
              {process.status}{process.exitCode !== undefined ? ` · exit ${process.exitCode}` : ""}
            </span>
          </span>
          <span
            class="shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-text-subtle"
          >
            {process.kind}
          </span>
        </button>

        <!-- hover actions -->
        <div
          class="flex shrink-0 items-center gap-0.5 opacity-100 transition md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
        >
          <button
            type="button"
            class="grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover hover:text-text disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-transparent"
            disabled={!canRestart(process.status)}
            aria-label={`Restart ${process.name}`}
            title={`Restart ${process.name}`}
            onclick={(event) => {
              event.stopPropagation();
              onRestart(process.name);
            }}
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
              <path d="M3 3v5h5" />
            </svg>
          </button>
          <button
            type="button"
            class="grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-danger/10 hover:text-danger disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-text-subtle"
            disabled={!canStop(process.status)}
            aria-label={`Stop ${process.name}`}
            title={`Stop ${process.name}`}
            onclick={(event) => {
              event.stopPropagation();
              onStop(process.name);
            }}
          >
            <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <rect x="6" y="6" width="12" height="12" rx="1.5" />
            </svg>
          </button>
        </div>
      </div>
    {/each}
  {/if}
</div>
