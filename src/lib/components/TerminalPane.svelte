<script lang="ts">
  import { onMount } from "svelte";
  import { FitAddon } from "@xterm/addon-fit";
  import { Terminal } from "xterm";
  import "xterm/css/xterm.css";

  type Props = {
    terminalId: string | null;
    output: string;
    onInput: (data: string) => void;
    onResize: (cols: number, rows: number) => void;
    onOpenTerminal: () => void;
  };

  let { terminalId, output, onInput, onResize, onOpenTerminal }: Props = $props();

  let host = $state<HTMLDivElement | null>(null);
  let xterm = $state<Terminal | null>(null);
  let fitAddon = $state<FitAddon | null>(null);
  let resizeObserver = $state<ResizeObserver | null>(null);
  let lastRenderedOutput = $state("");
  let activeTerminalId = $state<string | null>(null);
  let terminalOpened = $state(false);
  let pendingFitFrame = $state<number | null>(null);

  function fit() {
    fitAddon?.fit();
    if (xterm && terminalId) {
      onResize(xterm.cols, xterm.rows);
    }
  }

  function fitOnNextFrame() {
    if (pendingFitFrame !== null) {
      cancelAnimationFrame(pendingFitFrame);
    }
    pendingFitFrame = requestAnimationFrame(() => {
      pendingFitFrame = null;
      fit();
    });
  }

  onMount(() => {
    xterm = new Terminal({
      cursorBlink: true,
      fontFamily:
        'JetBrains Mono, ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace',
      fontSize: 12,
      theme: {
        background: "#08090b",
        foreground: "#e7e9ee",
        cursor: "#5b8def",
        cursorAccent: "#08090b",
        selectionBackground: "#5b8def33",
      },
    });
    fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);
    xterm.onData((data) => onInput(data));

    return () => {
      if (pendingFitFrame !== null) {
        cancelAnimationFrame(pendingFitFrame);
      }
      resizeObserver?.disconnect();
      xterm?.dispose();
    };
  });

  $effect(() => {
    if (!xterm || !host || terminalOpened) {
      return;
    }

    xterm.open(host);
    terminalOpened = true;
    fitOnNextFrame();
    resizeObserver = new ResizeObserver(() => fitOnNextFrame());
    resizeObserver.observe(host);
  });

  $effect(() => {
    if (!xterm) {
      return;
    }

    if (terminalId !== activeTerminalId) {
      xterm.reset();
      lastRenderedOutput = "";
      activeTerminalId = terminalId;
      if (terminalId && output.length > 0) {
        xterm.write(output);
        lastRenderedOutput = output;
      }
      fitOnNextFrame();
    }

    if (!terminalId) {
      return;
    }

    if (!output.startsWith(lastRenderedOutput)) {
      xterm.reset();
      xterm.write(output);
      lastRenderedOutput = output;
      return;
    }

    const delta = output.slice(lastRenderedOutput.length);
    if (delta.length > 0) {
      xterm.write(delta);
      lastRenderedOutput = output;
    }
  });
</script>

<section class="flex h-full min-h-0 flex-col bg-canvas">
  <div
    bind:this={host}
    class={`min-h-0 flex-1 overflow-hidden px-2 py-2 ${terminalId ? "" : "hidden"}`}
  ></div>
  {#if !terminalId}
    <div class="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 px-4 text-center">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor"
        stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-text-subtle" aria-hidden="true">
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </svg>
      <div class="text-sm text-text-subtle">Open a terminal from the project menu</div>
      <button
        type="button"
        class="rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-text transition-colors hover:bg-surface-hover hover:border-text-muted"
        onclick={onOpenTerminal}
      >
        Open terminal
      </button>
    </div>
  {/if}
</section>
