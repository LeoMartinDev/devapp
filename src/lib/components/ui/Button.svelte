<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  type Variant = "primary" | "secondary" | "danger" | "ghost";
  type Size = "sm" | "md";

  type Props = HTMLButtonAttributes & {
    variant?: Variant;
    size?: Size;
    children?: Snippet;
  };

  let {
    variant = "secondary",
    size = "md",
    disabled = false,
    type = "button",
    class: className = "",
    children,
    ...rest
  }: Props = $props();

  const variantClass: Record<Variant, string> = {
    primary:
      "bg-accent text-canvas font-medium hover:bg-accent-hover disabled:bg-surface-hover disabled:text-text-subtle",
    secondary:
      "border border-border bg-surface-raised text-text-muted hover:bg-surface-hover hover:text-text disabled:text-text-subtle",
    danger:
      "border border-danger/30 bg-danger/10 text-danger hover:bg-danger/20 hover:text-danger disabled:text-text-subtle",
    ghost:
      "text-text-muted hover:bg-surface-hover hover:text-text disabled:text-text-subtle",
  };

  const sizeClass: Record<Size, string> = {
    sm: "h-7 px-2.5 text-xs",
    md: "h-9 px-3.5 text-sm",
  };
</script>

<button
  {...rest}
  {type}
  {disabled}
  class={`inline-flex items-center justify-center gap-1.5 rounded-md transition-colors disabled:cursor-not-allowed ${variantClass[variant]} ${sizeClass[size]} ${className}`}
>
  {@render children?.()}
</button>
