<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  type Props = Omit<HTMLInputAttributes, "value"> & {
    label?: string;
    value?: string | number | null;
    error?: string | null;
    monospace?: boolean;
    density?: "default" | "compact";
  };

  let {
    label,
    value = $bindable(""),
    placeholder,
    error = null,
    type = "text",
    autocomplete,
    monospace = false,
    density = "default",
    class: className = "",
    ...rest
  }: Props = $props();

  const inputClass = $derived(
    `w-full rounded-md border bg-surface-raised text-sm text-text outline-none transition-colors duration-75 placeholder:text-text-subtle ${
      error ? "border-danger focus:border-danger" : "border-border focus:border-accent"
    } ${density === "compact" ? "h-6 px-1.5 text-[12px]" : "h-9 px-3"} ${monospace ? "font-mono text-[13px]" : ""} ${className}`,
  );
</script>

<label class="grid gap-1.5 text-sm">
  {#if label}
    <span class="text-text-subtle">{label}</span>
  {/if}
  <input {...rest} {type} {placeholder} {autocomplete} class={inputClass} bind:value />
  {#if error}
    <span class="text-xs text-danger">{error}</span>
  {/if}
</label>
