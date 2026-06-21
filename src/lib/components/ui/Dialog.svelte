<script lang="ts">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";

  type Size = "sm" | "md" | "lg" | "xl";

  type Props = {
    open: boolean;
    title: string;
    description?: string;
    size?: Size;
    onClose: () => void;
    closeOnOverlay?: boolean;
    children?: Snippet;
    footer?: Snippet;
  };

  let {
    open,
    title,
    description,
    size = "md",
    onClose,
    closeOnOverlay = true,
    children,
    footer,
  }: Props = $props();

  let panel = $state<HTMLElement | null>(null);
  let titleId = $state(`dialog-title-${crypto.randomUUID()}`);
  let descriptionId = $state(`dialog-description-${crypto.randomUUID()}`);
  let previouslyFocused = $state<HTMLElement | null>(null);
  let wasOpen = $state(false);

  const sizeClass: Record<Size, string> = {
    sm: "w-[min(440px,calc(100vw-32px))]",
    md: "w-[min(560px,calc(100vw-32px))]",
    lg: "w-[min(840px,calc(100vw-32px))]",
    xl: "w-[min(1040px,calc(100vw-32px))]",
  };

  const focusableSelector = [
    "a[href]",
    "button:not([disabled])",
    "textarea:not([disabled])",
    "input:not([disabled])",
    "select:not([disabled])",
    "[tabindex]:not([tabindex='-1'])",
  ].join(",");

  function focusableElements() {
    if (!panel) {
      return [];
    }
    return Array.from(panel.querySelectorAll<HTMLElement>(focusableSelector)).filter(
      (element) => !element.hasAttribute("disabled") && element.tabIndex !== -1,
    );
  }

  async function focusInitial() {
    await tick();
    const first = focusableElements()[0] ?? panel;
    first?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!open) {
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key !== "Tab") {
      return;
    }

    const elements = focusableElements();
    if (elements.length === 0) {
      event.preventDefault();
      panel?.focus();
      return;
    }

    const first = elements[0];
    const last = elements[elements.length - 1];
    const active = document.activeElement;
    if (event.shiftKey && active === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }

  $effect(() => {
    if (open && !wasOpen) {
      previouslyFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      wasOpen = true;
      void focusInitial();
      return;
    }
    if (!open && wasOpen) {
      wasOpen = false;
      previouslyFocused?.focus();
      previouslyFocused = null;
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <button
      type="button"
      class="absolute inset-0 bg-black/60 backdrop-blur-sm"
      aria-label="Close dialog"
      tabindex="-1"
      onclick={() => closeOnOverlay && onClose()}
    ></button>
    <div
      bind:this={panel}
      class={`relative z-10 flex max-h-[calc(100vh-48px)] min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-surface text-text shadow-2xl outline-none ${sizeClass[size]}`}
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={description ? descriptionId : undefined}
      tabindex="-1"
    >
      <header class="border-b border-border px-5 py-4">
        <h2 id={titleId} class="text-sm font-semibold">{title}</h2>
        {#if description}
          <p id={descriptionId} class="mt-1 text-xs leading-5 text-text-subtle">{description}</p>
        {/if}
      </header>

      <div class="min-h-0 flex-1 overflow-hidden">
        {@render children?.()}
      </div>

      {#if footer}
        <footer class="border-t border-border px-5 py-4">
          {@render footer()}
        </footer>
      {/if}
    </div>
  </div>
{/if}
