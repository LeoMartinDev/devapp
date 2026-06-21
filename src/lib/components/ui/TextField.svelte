<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  type Props = Omit<HTMLInputAttributes, "value"> & {
    label?: string;
    value?: string | number | null;
    error?: string | null;
    monospace?: boolean;
  };

  let {
    label,
    value = $bindable(""),
    placeholder,
    error = null,
    type = "text",
    autocomplete,
    monospace = false,
    class: className = "",
    ...rest
  }: Props = $props();

  const inputClass = $derived(
    `h-9 w-full rounded-md border bg-surface-raised px-3 text-sm text-text outline-none transition-colors placeholder:text-text-subtle ${
      error ? "border-danger focus:border-danger" : "border-border focus:border-accent"
    } ${monospace ? "font-mono text-[13px]" : ""} ${className}`,
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
