<script lang="ts">
  import type { ProcessForm } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";

  type Props = {
    projectName: string;
    processes: ProcessForm[];
    selectedProcess: ProcessForm | null;
    loading: boolean;
    disabled: boolean;
    onAdd: () => void;
    onSelect: (id: string) => void;
  };

  let {
    projectName,
    processes,
    selectedProcess,
    loading,
    disabled,
    onAdd,
    onSelect,
  }: Props = $props();
</script>

<aside class="flex min-h-0 flex-col border-b border-border bg-surface lg:border-b-0 lg:border-r">
  <div class="flex items-center justify-between gap-3 border-b border-border px-4 py-3 lg:block lg:py-4">
    <div class="min-w-0">
      <div class="text-sm font-semibold text-text">Project Settings</div>
      <div class="mt-1 truncate text-[11px] text-text-subtle">{projectName}</div>
    </div>
    <Button class="shrink-0 lg:hidden" size="sm" onclick={onAdd} disabled={loading || disabled}>
      Add process
    </Button>
  </div>

  <div class="hidden border-b border-border p-3 lg:block">
    <Button class="w-full" onclick={onAdd} disabled={loading || disabled}>
      Add process
    </Button>
  </div>

  <div class="flex min-h-0 gap-2 overflow-x-auto p-2 lg:block lg:flex-1 lg:overflow-y-auto">
    {#each processes as process (process.id)}
      <button
        type="button"
        class={`flex min-w-48 items-center justify-between gap-2 rounded-md px-3 py-2 text-left text-[13px] transition-colors duration-75 lg:mb-1 lg:w-full lg:min-w-0 ${
          process.id === selectedProcess?.id
            ? "bg-surface-raised text-text"
            : "text-text-subtle hover:bg-surface-hover hover:text-text"
        }`}
        aria-current={process.id === selectedProcess?.id ? "true" : undefined}
        onclick={() => onSelect(process.id)}
      >
        <span class="min-w-0 truncate">{process.name || "Unnamed process"}</span>
        <span class="shrink-0 text-[11px] text-text-subtle">{process.kind}</span>
      </button>
    {/each}
  </div>
</aside>
