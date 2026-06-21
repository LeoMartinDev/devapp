<script lang="ts">
  import type { DependencyCondition } from "$lib/types";
  import type { ProcessForm } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";

  type Props = {
    process: ProcessForm;
    processes: ProcessForm[];
    dependencyIssue: (process: ProcessForm, dependencyId: string) => string | null;
    onAdd: (process: ProcessForm) => void;
    onRemove: (process: ProcessForm, dependencyId: string) => void;
  };

  let {
    process,
    processes,
    dependencyIssue,
    onAdd,
    onRemove,
  }: Props = $props();
</script>

<div class="grid gap-2">
  <div class="flex items-center justify-between">
    <span class="text-sm text-text-muted">Dependencies</span>
    <Button size="sm" onclick={() => onAdd(process)}>Add dependency</Button>
  </div>
  {#if process.dependencies.length === 0}
    <div class="rounded-md border border-dashed border-border px-3 py-2 text-sm text-text-subtle">
      Starts without dependencies.
    </div>
  {:else}
    {#each process.dependencies as dependency (dependency.id)}
      {@const depError = dependencyIssue(process, dependency.id)}
      <div class="grid gap-1">
        <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_150px_auto]">
          <select
            class={`h-9 rounded-md border px-3 text-sm outline-none transition-colors ${
              depError
                ? "border-danger focus:border-danger"
                : "border-border bg-surface-raised focus:border-accent"
            }`}
            bind:value={dependency.processName}
          >
            <option value="">Select process</option>
            {#each processes.filter((candidate) => candidate.id !== process.id) as candidate}
              <option value={candidate.name}>{candidate.name}</option>
            {/each}
          </select>
          <select
            class="h-9 rounded-md border border-border bg-surface-raised px-3 text-sm outline-none transition-colors focus:border-accent"
            bind:value={dependency.condition}
          >
            <option value={"ready" satisfies DependencyCondition}>Ready</option>
            <option value={"success" satisfies DependencyCondition}>Success</option>
          </select>
          <Button variant="danger" onclick={() => onRemove(process, dependency.id)}>
            Remove
          </Button>
        </div>
        {#if depError}
          <span class="text-xs text-danger">{depError}</span>
        {/if}
      </div>
    {/each}
  {/if}
</div>
