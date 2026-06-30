<script lang="ts" module>
  export type MenuIcon =
    | "edit"
    | "config"
    | "restart"
    | "stop"
    | "copy"
    | "clear"
    | "close"
    | "terminal";

  export type MenuItem = {
    label: string;
    onSelect: () => void;
    disabled?: boolean;
    danger?: boolean;
    dividerAfter?: boolean;
    icon?: MenuIcon;
  };
</script>

<script lang="ts">
  import { onMount, tick } from "svelte";

  type Props = {
    label: string;
    items: MenuItem[];
    disabled?: boolean;
  };

  let { label, items, disabled = false }: Props = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement | null>(null);

  function enabledItems() {
    if (!root) {
      return [];
    }
    return Array.from(root.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')).filter(
      (item) => !item.disabled,
    );
  }

  async function focusFirstItem() {
    await tick();
    enabledItems()[0]?.focus();
  }

  function onPointerDown(event: MouseEvent) {
    if (!root) {
      return;
    }
    if (!root.contains(event.target as Node)) {
      open = false;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (!open) {
      return;
    }

    if (event.key === "Escape") {
      open = false;
      return;
    }

    const items = enabledItems();
    if (items.length === 0) {
      return;
    }

    const activeIndex = items.indexOf(document.activeElement as HTMLButtonElement);
    const currentIndex = activeIndex === -1 ? 0 : activeIndex;

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const nextIndex = event.key === "ArrowDown"
        ? (currentIndex + 1) % items.length
        : (currentIndex - 1 + items.length) % items.length;
      items[nextIndex]?.focus();
      return;
    }

    if (event.key === "Home") {
      event.preventDefault();
      items[0]?.focus();
      return;
    }

    if (event.key === "End") {
      event.preventDefault();
      items[items.length - 1]?.focus();
    }
  }

  onMount(() => {
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("keydown", onKeydown);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("keydown", onKeydown);
    };
  });

  function choose(item: MenuItem) {
    if (item.disabled) {
      return;
    }
    open = false;
    item.onSelect();
  }

  $effect(() => {
    if (!open) {
      return;
    }

    void focusFirstItem();
  });
</script>

<div bind:this={root} class="relative">
  <button
    type="button"
    {disabled}
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label={label}
    title={label}
    class="grid h-7 w-7 place-items-center rounded-md text-text-muted transition-colors duration-75 hover:bg-surface-hover hover:text-text disabled:cursor-not-allowed disabled:opacity-55"
    onclick={() => (open = !open)}
  >
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  </button>

  {#if open}
    <div
      role="menu"
      aria-label={label}
      class="absolute right-0 top-full z-30 mt-1 min-w-[180px] overflow-hidden rounded-md border border-border bg-surface-raised py-1 shadow-2xl select-none"
    >
      {#each items as item, i (i)}
        <button
          type="button"
          role="menuitem"
          disabled={item.disabled}
          class={`flex w-full items-center px-2.5 py-1.5 text-left text-[13px] transition-colors duration-75 disabled:cursor-not-allowed disabled:opacity-40 ${
            item.disabled
              ? ""
              : item.danger
                ? "text-danger hover:bg-danger/10"
                : "text-text-muted hover:bg-surface-hover hover:text-text"
          }`}
          onclick={() => choose(item)}
        >
          <span class="mr-2 inline-flex h-4 w-4 shrink-0 items-center justify-center opacity-80" aria-hidden="true">
            {#if item.icon === "edit"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9" /><path d="M16.5 3.5a2.12 2.12 0 1 1 3 3L7 19l-4 1 1-4z" /></svg>
            {:else if item.icon === "config"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
            {:else if item.icon === "restart"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" /></svg>
            {:else if item.icon === "stop"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="1.5" /></svg>
            {:else if item.icon === "copy"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
            {:else if item.icon === "clear"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /></svg>
            {:else if item.icon === "close"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
            {:else if item.icon === "terminal"}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5" /><line x1="12" y1="19" x2="20" y2="19" /></svg>
            {/if}
          </span>
          {item.label}
        </button>
        {#if item.dividerAfter}
          <div class="my-1 h-px bg-border"></div>
        {/if}
      {/each}
    </div>
  {/if}
</div>
