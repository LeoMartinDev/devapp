<script lang="ts">
  import type { EnvRow } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";

  type Props = {
    rows: EnvRow[];
    processId?: string;
    issueFor: (key: string) => string | null;
    onAdd: () => void;
    onRemove: (id: string) => void;
  };

  let { rows = $bindable(), processId, issueFor, onAdd, onRemove }: Props = $props();
</script>

<section class="grid gap-3">
  <div class="flex items-center justify-between">
    <h2 class="text-sm font-semibold text-text">Environment variables</h2>
    <Button size="sm" onclick={onAdd}>Add variable</Button>
  </div>
  <div class="grid gap-2">
    {#if rows.length === 0}
      <div class="rounded-md border border-dashed border-border px-3 py-3 text-sm text-text-subtle">
        No global variables.
      </div>
    {:else}
      {#each rows as row (row.id)}
        {@const envIssueKey = processId ? `process.${processId}.env.${row.id}.key` : `env.${row.id}.key`}
        {@const keyError = issueFor(envIssueKey)}
        <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)_auto]">
          <label class="grid gap-1">
            <input
              class={`h-9 rounded-md border px-3 text-sm outline-none transition-colors ${
                keyError
                  ? "border-danger focus:border-danger"
                  : "border-border bg-surface-raised focus:border-accent"
              }`}
              placeholder="KEY"
              bind:value={row.key}
            />
            {#if keyError}
              <span class="text-xs text-danger">{keyError}</span>
            {/if}
          </label>
          <input
            class="h-9 rounded-md border border-border bg-surface-raised px-3 text-sm outline-none transition-colors focus:border-accent"
            placeholder="value"
            bind:value={row.value}
          />
          <Button variant="danger" onclick={() => onRemove(row.id)}>
            Remove
          </Button>
        </div>
      {/each}
    {/if}
  </div>
</section>
