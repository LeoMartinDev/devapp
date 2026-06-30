<script lang="ts">
  export type SegmentOption<T extends string = string> = {
    value: T;
    label: string;
  };

  type Props<T extends string = string> = {
    value: T;
    options: SegmentOption<T>[];
    onChange: (value: T) => void;
    label?: string;
  };

  let { value, options, onChange, label = "View" }: Props = $props();
</script>

<div class="inline-flex rounded-md border border-border bg-surface-raised p-0.5" role="group" aria-label={label}>
  {#each options as option (option.value)}
    <button
      type="button"
      class={`h-7 rounded px-2.5 text-[13px] transition-colors duration-75 ${
        value === option.value ? "bg-surface-hover text-text" : "text-text-subtle hover:text-text-muted"
      }`}
      aria-pressed={value === option.value}
      title={option.label}
      onclick={() => onChange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>
