<script lang="ts" module>
  export type MenuItem = {
    label: string;
    onSelect: () => void;
    disabled?: boolean;
    danger?: boolean;
    dividerAfter?: boolean;
  };
</script>

<script lang="ts">
  import { onMount } from "svelte";

  type Props = {
    label: string;
    items: MenuItem[];
    disabled?: boolean;
  };

  let { label, items, disabled = false }: Props = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement | null>(null);

  function onPointerDown(event: MouseEvent) {
    if (!root) {
      return;
    }
    if (!root.contains(event.target as Node)) {
      open = false;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (open && event.key === "Escape") {
      open = false;
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
</script>

<div bind:this={root} class="relative">
  <button
    type="button"
    {disabled}
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label={label}
    title={label}
    class="grid h-8 w-8 place-items-center rounded-md text-text-muted transition-colors duration-75 hover:bg-surface-hover hover:text-text disabled:cursor-not-allowed disabled:opacity-55"
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
          class={`flex w-full items-center px-3 py-1.5 text-left text-[13px] transition-colors duration-75 disabled:cursor-not-allowed disabled:opacity-40 ${
            item.disabled
              ? ""
              : item.danger
                ? "text-danger hover:bg-danger/10"
                : "text-text-muted hover:bg-surface-hover hover:text-text"
          }`}
          onclick={() => choose(item)}
        >
          {item.label}
        </button>
        {#if item.dividerAfter}
          <div class="my-1 h-px bg-border"></div>
        {/if}
      {/each}
    </div>
  {/if}
</div>
