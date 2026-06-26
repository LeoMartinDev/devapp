<script lang="ts" module>
  export type IconName =
    | "play"
    | "stop"
    | "pause"
    | "task"
    | "service"
    | "terminal"
    | "close"
    | "restart"
    | "search"
    | "scroll-down"
    | "trash"
    | "git-branch"
    | "settings"
    | "spinner"
    | "plus"
    | "check"
    | "info"
    | "error";
</script>

<script lang="ts">
  type Props = {
    name: IconName;
    size?: "xs" | "sm" | "md" | "lg";
    class?: string;
  };

  let { name, size = "sm", class: className = "" }: Props = $props();

  const sizePx: Record<string, number> = { xs: 12, sm: 14, md: 16, lg: 20 };
</script>

<svg
  width={sizePx[size]}
  height={sizePx[size]}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  aria-hidden="true"
>
  {#if name === "play"}
    <path d="M8 5v14l11-7z" fill="currentColor" stroke="none" />
  {:else if name === "stop"}
    <rect x="6" y="6" width="12" height="12" rx="1.5" fill="currentColor" stroke="none" />
  {:else if name === "pause"}
    <rect x="6" y="5" width="4" height="14" rx="1" fill="currentColor" stroke="none" />
    <rect x="14" y="5" width="4" height="14" rx="1" fill="currentColor" stroke="none" />
  {:else if name === "task"}
    <path d="M13 2L3 14h8l-2 8 10-12h-8l2-8z" fill="currentColor" stroke="none" />
  {:else if name === "service"}
    <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
  {:else if name === "terminal"}
    <polyline points="4 17 10 11 4 5" />
    <line x1="12" y1="19" x2="20" y2="19" />
  {:else if name === "close"}
    <line x1="18" y1="6" x2="6" y2="18" />
    <line x1="6" y1="6" x2="18" y2="18" />
  {:else if name === "restart"}
    <polyline points="1 4 1 10 7 10" />
    <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
  {:else if name === "search"}
    <circle cx="11" cy="11" r="8" />
    <line x1="21" y1="21" x2="16.65" y2="16.65" />
  {:else if name === "scroll-down"}
    <line x1="4" y1="21" x2="20" y2="21" />
    <line x1="12" y1="3" x2="12" y2="18" />
    <polyline points="8 14 12 18 16 14" />
  {:else if name === "trash"}
    <polyline points="3 6 5 6 21 6" />
    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    <line x1="10" y1="11" x2="10" y2="17" />
    <line x1="14" y1="11" x2="14" y2="17" />
  {:else if name === "git-branch"}
    <circle cx="12" cy="12" r="3" />
    <path d="M6 6v12M18 6v12" />
  {:else if name === "settings"}
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
  {:else if name === "spinner"}
    <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
    <path d="M12 2a10 10 0 0 1 10 10" />
  {:else if name === "plus"}
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  {:else if name === "check"}
    <polyline points="20 6 9 17 4 12" />
  {:else if name === "info"}
    <circle cx="12" cy="12" r="10" />
    <line x1="12" y1="16" x2="12" y2="12" />
    <line x1="12" y1="8" x2="12.01" y2="8" />
  {:else if name === "error"}
    <circle cx="12" cy="12" r="10" />
    <line x1="15" y1="9" x2="9" y2="15" />
    <line x1="9" y1="9" x2="15" y2="15" />
  {/if}
</svg>
