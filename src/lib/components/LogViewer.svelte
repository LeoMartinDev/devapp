<script lang="ts">
  import { MAX_LOG_LINES_PER_PROCESS } from "$lib/stores/runtime.svelte";
  import type { ProcessLogPayload } from "$lib/types";

  type Props = {
    logs: ProcessLogPayload[];
    processName: string | null;
    truncatedCount: number;
    onClear: () => void;
  };

  let { logs, processName, truncatedCount, onClear }: Props = $props();

  let query = $state("");
  let autoScroll = $state(true);
  let paused = $state(false);
  let pausedLogs = $state<ProcessLogPayload[] | null>(null);
  let viewport = $state<HTMLDivElement | null>(null);
  let searchInput = $state<HTMLInputElement | null>(null);
  let activeProcessName = $state<string | null>(null);
  let copied = $state(false);
  let copyTimer = $state<number | null>(null);

  const visibleLogs = $derived(paused ? (pausedLogs ?? logs) : logs);
  const totalVisibleCount = $derived(visibleLogs.length);

  let filteredLogs = $derived.by(() =>
    visibleLogs.filter((entry) =>
      query.trim().length === 0
        ? true
        : `${entry.stream} ${entry.line}`.toLowerCase().includes(query.toLowerCase()),
    ),
  );

  function togglePaused() {
    paused = !paused;
    pausedLogs = paused ? [...logs] : null;
  }

  function clearLogs() {
    onClear();
    if (paused) {
      pausedLogs = [];
    }
  }

  async function copyLogs() {
    const text = filteredLogs
      .map(
        (entry) =>
          `${new Date(entry.timestamp).toLocaleTimeString()} ${entry.stream} ${entry.line}`,
      )
      .join("\n");
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      if (copyTimer !== null) {
        clearTimeout(copyTimer);
      }
      copyTimer = window.setTimeout(() => {
        copied = false;
        copyTimer = null;
      }, 1400);
    } catch {
      // clipboard unavailable — fail silently
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    const typing =
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      target?.isContentEditable;

    // `/` focuses search (terminal/zed convention); Esc clears + blurs.
    if (!typing && event.key === "/" && searchInput) {
      event.preventDefault();
      searchInput.focus();
      return;
    }
    if (typing && event.key === "Escape" && target === searchInput) {
      if (query.length > 0) {
        query = "";
      } else {
        searchInput.blur();
      }
    }
  }

  $effect(() => {
    if (processName === activeProcessName) {
      return;
    }
    activeProcessName = processName;
    paused = false;
    pausedLogs = null;
  });

  $effect(() => {
    filteredLogs.length;
    autoScroll;
    paused;
    if (autoScroll && !paused && viewport) {
      requestAnimationFrame(() => {
        viewport?.scrollTo({ top: viewport.scrollHeight });
      });
    }
  });

  $effect(() => {
    return () => {
      if (copyTimer !== null) {
        clearTimeout(copyTimer);
      }
    };
  });

  const toneByStream: Record<string, string> = {
    stdout: "text-text",
    stderr: "text-danger",
    system: "text-accent",
  };
</script>

<svelte:window onkeydown={handleKeydown} />

<section class="flex h-full min-h-0 flex-col bg-canvas">
  <!-- compact sub-toolbar: search + actions, one row -->
  <div class="flex items-center gap-2 border-b border-border px-3 py-2">
    <div class="relative min-w-0 flex-1">
      <svg
        class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-text-subtle"
        width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor"
        stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input
        bind:this={searchInput}
        bind:value={query}
        type="text"
        placeholder="Search logs"
        spellcheck="false"
        class="h-8 w-full rounded-md border border-border bg-surface-raised pl-8 pr-12 text-[13px] text-text outline-none transition-colors placeholder:text-text-subtle focus:border-accent"
      />
      <kbd
        class="pointer-events-none absolute right-2.5 top-1/2 hidden -translate-y-1/2 rounded border border-border bg-surface px-1.5 py-0.5 text-[10px] text-text-subtle sm:inline-block"
      >
        /
      </kbd>
    </div>

    <div class="flex shrink-0 items-center gap-0.5">
      <span class="mr-1.5 hidden whitespace-nowrap text-[11px] text-text-subtle md:inline">
        {filteredLogs.length}/{totalVisibleCount}
      </span>

      <button
        type="button"
        class={`grid h-8 w-8 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-hover hover:text-text ${
          autoScroll ? "text-accent hover:text-accent" : ""
        }`}
        onclick={() => (autoScroll = !autoScroll)}
        aria-pressed={autoScroll}
        aria-label={autoScroll ? "Auto-scroll on" : "Auto-scroll off"}
        title={autoScroll ? "Auto-scroll: on" : "Auto-scroll: off"}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>

      <button
        type="button"
        class="grid h-8 w-8 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-hover hover:text-text"
        onclick={togglePaused}
        aria-pressed={paused}
        aria-label={paused ? "Resume live log view" : "Pause live log view"}
        title={paused ? "Resume" : "Pause"}
      >
        {#if paused}
          <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M8 5v14l11-7z" />
          </svg>
        {:else}
          <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="5" width="4" height="14" rx="1" />
            <rect x="14" y="5" width="4" height="14" rx="1" />
          </svg>
        {/if}
      </button>

      <button
        type="button"
        class="grid h-8 w-8 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-hover hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
        onclick={copyLogs}
        disabled={filteredLogs.length === 0}
        aria-label="Copy logs to clipboard"
        title="Copy logs"
      >
        {#if copied}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="text-success" aria-hidden="true">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        {:else}
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
        {/if}
      </button>

      <button
        type="button"
        class="grid h-8 w-8 place-items-center rounded-md text-text-subtle transition-colors hover:bg-danger/10 hover:text-danger disabled:cursor-not-allowed disabled:opacity-40"
        onclick={clearLogs}
        disabled={totalVisibleCount === 0}
        aria-label="Clear logs"
        title="Clear logs"
      >
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="3 6 5 6 21 6" />
          <path d="M19 6l-2 14a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2L5 6" />
          <path d="M10 11v6M14 11v6" />
          <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
        </svg>
      </button>
    </div>
  </div>

  {#if paused}
    <div class="flex items-center gap-2 border-b border-border bg-warning/5 px-3 py-1.5 text-[11px] text-warning">
      <span class="relative inline-flex h-1.5 w-1.5">
        <span class="absolute inline-flex h-1.5 w-1.5 animate-ping rounded-full bg-warning opacity-60"></span>
        <span class="relative inline-flex h-1.5 w-1.5 rounded-full bg-warning"></span>
      </span>
      Paused — log collection continues in the background.
    </div>
  {/if}

  {#if truncatedCount > 0}
    <div class="border-b border-border px-3 py-1 text-[11px] text-text-subtle">
      {truncatedCount} older line{truncatedCount === 1 ? "" : "s"} hidden · latest {Math.min(totalVisibleCount, MAX_LOG_LINES_PER_PROCESS)} shown
    </div>
  {/if}

  <div bind:this={viewport} class="min-h-0 flex-1 overflow-auto px-3 py-2 font-mono text-[12px] leading-[1.45]">
    {#if filteredLogs.length === 0}
      <div class="px-1 py-1 text-text-subtle">{query ? "No matching lines" : "No log line"}</div>
    {:else}
      {#each filteredLogs as entry, index (`${entry.timestamp}-${index}`)}
        <div class="group flex gap-3 rounded px-1 py-px hover:bg-surface-hover/40">
          <span class="shrink-0 select-none text-text-subtle">
            {new Date(entry.timestamp).toLocaleTimeString()}
          </span>
          <span class="w-12 shrink-0 select-none uppercase text-text-subtle">{entry.stream}</span>
          <span class={`whitespace-pre-wrap break-words [overflow-wrap:anywhere] ${toneByStream[entry.stream] ?? "text-text"}`}>
            {entry.line}
          </span>
        </div>
      {/each}
    {/if}
  </div>
</section>
