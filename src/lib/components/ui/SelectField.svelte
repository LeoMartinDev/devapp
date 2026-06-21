<script lang="ts">
  import type { HTMLSelectAttributes } from "svelte/elements";

  export type SelectOption = {
    value: string;
    label: string;
    disabled?: boolean;
  };

  type Props = Omit<HTMLSelectAttributes, "value"> & {
    label?: string;
    value?: string;
    options: SelectOption[];
    error?: string | null;
  };

  let {
    label,
    value = $bindable(""),
    options,
    error = null,
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<label class="grid gap-1.5 text-sm">
  {#if label}
    <span class="text-text-subtle">{label}</span>
  {/if}
  <select
    {...rest}
    class={`h-9 w-full rounded-md border bg-surface-raised px-3 text-sm text-text outline-none transition-colors ${
      error ? "border-danger focus:border-danger" : "border-border focus:border-accent"
    } ${className}`}
    bind:value
  >
    {#each options as option}
      <option value={option.value} disabled={option.disabled}>{option.label}</option>
    {/each}
  </select>
  {#if error}
    <span class="text-xs text-danger">{error}</span>
  {/if}
</label>
