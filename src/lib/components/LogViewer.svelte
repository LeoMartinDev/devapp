<script lang="ts">
  import { MAX_LOG_LINES_PER_PROCESS } from "$lib/stores/runtime.svelte";
  import type { ProcessLogPayload } from "$lib/types";

  type Props = {
    logs: ProcessLogPayload[];
    processName: string | null;
    truncatedCount: number;
    onClear: () => void;
    onActions?: (actions: { copy: () => void; clear: () => void }) => void;
  };

  let { logs, processName, truncatedCount, onClear, onActions }: Props = $props();

  const ROW_HEIGHT = 22;
  const OVERSCAN = 10;

  let query = $state("");
  let autoScroll = $state(true);
  let paused = $state(false);
  let pausedLogs = $state<ProcessLogPayload[] | null>(null);
  let viewport = $state<HTMLDivElement | null>(null);
  let searchInput = $state<HTMLInputElement | null>(null);
  let activeProcessName = $state<string | null>(null);
  let copied = $state(false);
  let copyTimer = $state<number | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  const visibleLogs = $derived(paused ? (pausedLogs ?? logs) : logs);
  const totalVisibleCount = $derived(visibleLogs.length);

  let filteredLogs = $derived.by(() =>
    visibleLogs.filter((entry) =>
      query.trim().length === 0
        ? true
        : `${entry.stream} ${entry.line}`.toLowerCase().includes(query.toLowerCase()),
    ),
  );

  const totalHeight = $derived(filteredLogs.length * ROW_HEIGHT);

  const startIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN));
  const endIndex = $derived(Math.min(
    filteredLogs.length,
    Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + OVERSCAN,
  ));

  const visibleItems = $derived(filteredLogs.slice(startIndex, endIndex));

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

  $effect(() => {
    onActions?.({ copy: copyLogs, clear: clearLogs });
  });

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

  function handleScroll() {
    if (!viewport) return;
    scrollTop = viewport.scrollTop;
    viewportHeight = viewport.clientHeight;
    const threshold = 4;
    const atBottom =
      scrollTop + viewportHeight >=
      viewport.scrollHeight - threshold;
    autoScroll = atBottom;
  }

  $effect(() => {
    const el = viewport;
    if (!el) return;
    const ro = new ResizeObserver(() => {
      if (viewport) viewportHeight = viewport.clientHeight;
    });
    ro.observe(el);
    viewportHeight = el.clientHeight;
    return () => ro.disconnect();
  });

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

  const borderByStream: Record<string, string> = {
    stdout: "border-l-[#e7e9ee30]",
    stderr: "border-l-danger",
    system: "border-l-[#5b8def]",
  };

  const borderCornerClass = $derived((indexInLogs: number): string => {
    const entry = filteredLogs[indexInLogs];
    if (!entry) return "";
    const prev = indexInLogs > 0 ? filteredLogs[indexInLogs - 1] : null;
    const next = indexInLogs < filteredLogs.length - 1 ? filteredLogs[indexInLogs + 1] : null;
    const sameAsPrev = prev !== null && prev.stream === entry.stream;
    const sameAsNext = next !== null && next.stream === entry.stream;

    if (!sameAsPrev && !sameAsNext) return "rounded-tl rounded-bl";
    if (!sameAsPrev) return "rounded-tl";
    if (!sameAsNext) return "rounded-bl";
    return "";
  });

  function escapeRegExp(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }

  type TextSegment = { text: string; match: boolean };

  function highlightLine(text: string, q: string): TextSegment[] {
    const trimmed = q.trim();
    if (trimmed.length === 0) return [{ text, match: false }];

    const regex = new RegExp(`(${escapeRegExp(trimmed)})`, "gi");
    const segments: TextSegment[] = [];
    let lastIndex = 0;
    let m: RegExpExecArray | null;

    while ((m = regex.exec(text)) !== null) {
      if (m.index > lastIndex) {
        segments.push({ text: text.slice(lastIndex, m.index), match: false });
      }
      segments.push({ text: m[0], match: true });
      lastIndex = regex.lastIndex;
    }

    if (lastIndex < text.length) {
      segments.push({ text: text.slice(lastIndex), match: false });
    }

    return segments;
  }

  const matchCount = $derived(
    (() => {
      const trimmed = query.trim();
      if (trimmed.length === 0) return null;
      const regex = new RegExp(escapeRegExp(trimmed), "gi");
      let count = 0;
      for (const entry of filteredLogs) {
        count += (entry.line.match(regex) ?? []).length;
      }
      return count;
    })(),
  );
</script>

<svelte:window onkeydown={handleKeydown} />

<section class="flex h-full min-h-0 flex-col bg-canvas">
  <div class="flex items-center gap-1.5 border-b border-border px-2 pt-[4px] pb-[5px]">
    <div class="relative min-w-0 flex-1">
      <svg
        class="pointer-events-none absolute left-1.5 top-1/2 -translate-y-1/2 text-text-subtle"
        width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor"
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
        class="h-6 w-full rounded-md border border-border bg-surface-raised pl-6 pr-2 text-[11px] text-text outline-none transition-colors duration-75 placeholder:text-[10px] placeholder:text-text-subtle focus:border-accent"
      />
      {#if matchCount !== null}
        <span class="pointer-events-none absolute right-1.5 top-1/2 -translate-y-1/2 text-[10px] text-text-subtle">
          {matchCount}
        </span>
      {/if}
    </div>

    <div
      class="flex shrink-0 items-center gap-0.5"
      role="toolbar"
      tabindex="0"
      aria-label="Log actions"
      onfocus={(e: FocusEvent) => {
        const buttons = Array.from(
          (e.currentTarget as HTMLElement).querySelectorAll<HTMLElement>("button"),
        );
        if (buttons.length === 0) return;
        const active = buttons.find((b) => b.getAttribute("aria-pressed") === "true") ?? buttons[0];
        active.focus();
      }}
      onkeydown={(e: KeyboardEvent) => {
        const target = e.currentTarget as HTMLElement | null;
        if (!target) return;
        if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
          e.preventDefault();
          const buttons = Array.from(
            target.querySelectorAll<HTMLElement>("button:not([disabled])"),
          );
          const idx = buttons.indexOf(document.activeElement as HTMLElement);
          const next = e.key === "ArrowRight"
            ? (idx + 1) % buttons.length
            : (idx - 1 + buttons.length) % buttons.length;
          buttons[next]?.focus();
        }
      }}
    >
      <button
        type="button"
        tabindex="-1"
        class="grid h-6 w-6 place-items-center rounded-md transition-colors duration-75 {autoScroll ? 'bg-accent/15 text-accent' : 'text-text-subtle hover:bg-surface-hover hover:text-text'}"
        onclick={() => (autoScroll = !autoScroll)}
        aria-pressed={autoScroll}
        aria-label={autoScroll ? "Auto-scroll on" : "Auto-scroll off"}
        title={autoScroll ? "Auto-scroll: on" : "Auto-scroll: off"}
      >
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <line x1="4" y1="21" x2="20" y2="21" />
          <line x1="12" y1="3" x2="12" y2="18" />
          <polyline points="8 14 12 18 16 14" />
        </svg>
      </button>

      <button
        type="button"
        tabindex="-1"
        class="grid h-6 w-6 place-items-center rounded-md text-text-subtle transition-colors duration-75 hover:bg-surface-hover hover:text-text"
        onclick={togglePaused}
        aria-pressed={paused}
        aria-label={paused ? "Resume live log view" : "Pause live log view"}
        title={paused ? "Resume" : "Pause"}
      >
        {#if paused}
          <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M8 5v14l11-7z" />
          </svg>
        {:else}
          <svg width="9" height="9" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="5" width="4" height="14" rx="1" />
            <rect x="14" y="5" width="4" height="14" rx="1" />
          </svg>
        {/if}
      </button>

      <button
        type="button"
        tabindex="-1"
        class="grid h-6 w-6 place-items-center rounded-md text-text-subtle transition-colors duration-75 hover:bg-surface-hover hover:text-danger"
        onclick={clearLogs}
        aria-label="Clear logs"
        title="Clear logs"
      >
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="3 6 5 6 21 6" />
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          <line x1="10" y1="11" x2="10" y2="17" />
          <line x1="14" y1="11" x2="14" y2="17" />
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

  <div bind:this={viewport} onscroll={handleScroll} data-native-selectable="logs" class="min-h-0 flex-1 overflow-auto font-mono text-[12px] leading-[1.45]">
    {#if filteredLogs.length === 0}
      <div class="px-3 py-2 text-text-subtle">{query ? "No matching lines" : "No log line"}</div>
    {:else}
      <div style="height: {totalHeight}px; position: relative;">
        {#each visibleItems as entry, index (`${entry.timestamp}-${startIndex + index}`)}
          <div
            style="position: absolute; top: {(startIndex + index) * ROW_HEIGHT}px; left: 0; right: 0; height: {ROW_HEIGHT}px;"
            class="group flex items-center gap-3 px-3 hover:bg-surface-hover/40 border-l-[3px] {borderByStream[entry.stream] ?? 'border-l-transparent'} {borderCornerClass(startIndex + index)}"
          >
            <span class="shrink-0 select-none text-text-subtle">
              {new Date(entry.timestamp).toLocaleTimeString()}
            </span>
            <span class="w-12 shrink-0 select-none uppercase text-text-subtle">{entry.stream}</span>
            <span class={`truncate ${toneByStream[entry.stream] ?? "text-text"}`}>
              {#if entry.stream === "system" && /ready|listening/i.test(entry.line)}
                <span class="mr-1">&#9679;</span>
              {/if}
              {#each highlightLine(entry.line, query) as seg}
                {#if seg.match}
                  <mark class="bg-warning/30 text-text rounded-[2px]">{seg.text}</mark>
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</section>
