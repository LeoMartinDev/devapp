<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    processList: Snippet;
    children: Snippet;
  };

  let {
    processList,
    children,
  }: Props = $props();
</script>

<main class="h-screen min-h-0 overflow-hidden bg-canvas text-text" style="padding-top: 40px">
  <div
    class="grid h-full min-h-0 grid-rows-[minmax(260px,42vh)_minmax(0,1fr)] overflow-hidden lg:grid-cols-[260px_minmax(0,1fr)] lg:grid-rows-none"
  >
    <aside
      class="grid min-h-0 grid-rows-[minmax(0,1fr)] border-b border-border bg-surface lg:border-b-0 lg:border-r"
    >
      {@render processList()}
    </aside>

    <section class="grid min-h-0 grid-rows-[minmax(0,1fr)] bg-canvas">
      <div class="min-h-0">
        {@render children()}
      </div>
    </section>
  </div>
</main>
