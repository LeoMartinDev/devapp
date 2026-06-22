<script lang="ts">
  import StatusDot from "$lib/components/ui/StatusDot.svelte";
  import type {
    ProcessRuntimeId,
    ProcessSnapshot,
    TerminalSnapshot,
  } from "$lib/types";

  type Props = {
    processes: ProcessSnapshot[];
    terminals: TerminalSnapshot[];
    selectedProcessRuntimeId: ProcessRuntimeId | null;
    selectedTerminalId: string | null;
    busy?: boolean;
    onSelectProcess: (runtimeId: ProcessRuntimeId) => void;
    onSelectTerminal: (terminalId: string) => void;
    onStop: (processName: string) => void;
    onStart: (processName: string) => void;
    onRestart: (processName: string) => void;
    onCloseTerminal: (terminalId: string) => void;
  };

  let {
    processes,
    terminals,
    selectedProcessRuntimeId,
    selectedTerminalId,
    busy = false,
    onSelectProcess,
    onSelectTerminal,
    onStop,
    onStart,
    onRestart,
    onCloseTerminal,
  }: Props = $props();

  // A process row exposes a single contextual action that depends on its state:
  //   running/ready/starting -> Stop
  //   stopped/failed/succeeded -> Start
  //   pending/blocked/stopping -> none (button disabled)
  type RowAction = "stop" | "start" | null;

  function rowAction(process: ProcessSnapshot): RowAction {
    if (process.kind === "task") return null;
    switch (process.status) {
      case "running":
      case "ready":
      case "starting":
        return "stop";
      case "stopped":
      case "failed":
      case "succeeded":
        return "start";
      default:
        return null;
    }
  }

  function actionEnabled(process: ProcessSnapshot) {
    return !busy && rowAction(process) !== null;
  }

  function statusLabel(process: ProcessSnapshot) {
    return process.exitCode !== undefined && process.exitCode !== null
      ? `${process.status} · exit ${process.exitCode}`
      : process.status;
  }
</script>

<div class="grid gap-0.5">
  {#if processes.length === 0 && terminals.length === 0}
    <div
      class="rounded-lg border border-dashed border-border px-3 py-6 text-center text-xs leading-5 text-text-subtle"
    >
      No process loaded
    </div>
  {:else}
    {#each processes as process (process.runtimeId)}
      {@const selected = process.runtimeId === selectedProcessRuntimeId}
      {@const action = rowAction(process)}
      {@const restartable = !busy && (action === "stop" || action === "start")}
      <div
        class={`group relative flex items-center gap-2.5 rounded-md px-3 py-1.5 transition-colors ${
          selected ? "bg-surface-raised" : "hover:bg-surface-hover"
        }`}
      >
        <button
          type="button"
          class="grid min-w-0 flex-1 grid-cols-[auto_minmax(0,1fr)] items-center gap-2.5 text-left"
          aria-current={selected ? "true" : undefined}
          onclick={() => onSelectProcess(process.runtimeId)}
        >
          {#if process.kind === "task"}
            <svg class="h-3.5 w-3.5 shrink-0 text-text-subtle" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <path d="M13 2L3 14h8l-2 8 10-12h-8l2-8z" />
            </svg>
          {:else}
            <StatusDot status={process.status} />
          {/if}
          <span class="min-w-0">
            <span class="block truncate text-[13px] font-medium text-text">{process.name}</span>
            <span class="block truncate text-[11px] text-text-subtle">
              {statusLabel(process)}
            </span>
          </span>
        </button>

        <div
          class="flex shrink-0 items-center gap-0.5 transition md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
        >
          {#if action}
            {@const danger = action === "stop"}
            <button
              type="button"
              class={`grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-40 ${danger ? "hover:bg-danger/10 hover:text-danger" : "hover:text-text"}`}
              disabled={!actionEnabled(process)}
              aria-label={`${action === "stop" ? "Stop" : "Start"} ${process.name}`}
              title={`${action === "stop" ? "Stop" : "Start"} ${process.name}`}
              onclick={(event) => {
                event.stopPropagation();
                if (action === "stop") {
                  onStop(process.name);
                } else if (action === "start") {
                  onStart(process.name);
                }
              }}
            >
              {#if action === "stop"}
                <!-- Stop: filled square -->
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                  <rect x="6" y="6" width="12" height="12" rx="1.5" />
                </svg>
              {:else}
                <!-- Start: triangle -->
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                  <path d="M7 5l12 7-12 7z" />
                </svg>
              {/if}
            </button>
            <button
              type="button"
              class="grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!restartable}
              aria-label={`Restart ${process.name}`}
              title={`Restart ${process.name}`}
              onclick={(event) => {
                event.stopPropagation();
                onRestart(process.name);
              }}
            >
              <!-- Restart: circular arrow -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <polyline points="1 4 1 10 7 10" />
                <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
              </svg>
            </button>
          {:else}
            <button
              type="button"
              class="grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover disabled:cursor-not-allowed disabled:opacity-40"
              disabled
              aria-label={`${process.name}`}
              title={`${process.name}`}
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M7 5l12 7-12 7z" />
              </svg>
            </button>
          {/if}
        </div>
      </div>
    {/each}

    {#each terminals.filter((t) => t.isOpen) as terminal (terminal.terminalId)}
      {@const selected = terminal.terminalId === selectedTerminalId}
      <div
        class={`group relative flex items-center gap-2.5 rounded-md px-3 py-1.5 transition-colors ${
          selected ? "bg-surface-raised" : "hover:bg-surface-hover"
        }`}
      >
        <button
          type="button"
          class="grid min-w-0 flex-1 grid-cols-[auto_minmax(0,1fr)] items-center gap-2.5 text-left"
          aria-current={selected ? "true" : undefined}
          onclick={() => onSelectTerminal(terminal.terminalId)}
        >
          <svg class="h-3.5 w-3.5 shrink-0 text-text-subtle" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" y1="19" x2="20" y2="19" />
          </svg>
          <span class="min-w-0">
            <span class="block truncate text-[13px] font-medium text-text">{terminal.title}</span>
            <span class="block truncate text-[11px] text-text-subtle">{terminal.cwd}</span>
          </span>
        </button>

        <div
          class="flex shrink-0 items-center gap-0.5 transition md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
        >
          <button
            type="button"
            class="grid h-6 w-6 place-items-center rounded text-text-subtle transition-colors hover:bg-danger/10 hover:text-danger disabled:cursor-not-allowed disabled:opacity-40"
            aria-label={`Close ${terminal.title}`}
            title={`Close ${terminal.title}`}
            onclick={(event) => {
              event.stopPropagation();
              onCloseTerminal(terminal.terminalId);
            }}
          >
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>
    {/each}
  {/if}
</div>
