<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  type Variant = "primary" | "secondary" | "danger" | "ghost";

  type Props = HTMLButtonAttributes & {
    label: string;
    variant?: Variant;
    children?: Snippet;
  };

  let {
    label,
    variant = "secondary",
    disabled = false,
    type = "button",
    title,
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const variantClass: Record<Variant, string> = {
    primary: "bg-accent text-canvas hover:bg-accent-hover disabled:bg-surface-hover",
    secondary:
      "border border-border text-text-muted hover:bg-surface-hover hover:text-text disabled:text-text-subtle",
    danger: "border border-danger/30 text-danger hover:bg-danger/10 disabled:text-text-subtle",
    ghost: "text-text-muted hover:bg-surface-hover hover:text-text disabled:text-text-subtle",
  };
</script>

<button
  {...rest}
  {type}
  {disabled}
  aria-label={label}
  title={title ?? label}
  class={`grid h-8 w-8 place-items-center rounded-md transition-colors duration-75 disabled:cursor-not-allowed ${variantClass[variant]} ${className}`}
>
  {@render children?.()}
</button>
