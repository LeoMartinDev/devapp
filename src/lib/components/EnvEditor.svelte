<script lang="ts">
  import type { EnvRow } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";

  type Props = {
    rows: EnvRow[];
    processId?: string;
    issueFor: (key: string) => string | null;
    onAdd: () => void;
    onRemove: (id: string) => void;
    onFieldBlur?: (key: string) => void;
  };

  let { rows = $bindable(), processId, issueFor, onAdd, onRemove, onFieldBlur }: Props = $props();

  const errorFieldClass = "border-danger focus:border-danger";
  const envScopeLabel = (value: string | undefined) => (value ? "Process" : "Global");
</script>

<section class="grid gap-3">
  <div class="flex items-start justify-between gap-3">
    <div>
      <h3 class="text-sm font-semibold text-text">Environment variables</h3>
      {#if processId}
        <p class="mt-1 text-xs leading-5 text-text-subtle">Injected into this process.</p>
      {/if}
    </div>
  </div>

  <div class="grid gap-2.5">
    {#if rows.length === 0}
      <div class="flex items-center justify-between gap-3 px-1 text-sm text-text-subtle">
        <span>{processId ? "No process variables." : "No global variables."}</span>
        <Button size="sm" variant="ghost" onclick={onAdd}>New +</Button>
      </div>
    {:else}
      <div class="flex justify-end mb-1">
        <Button size="sm" variant="ghost" onclick={onAdd}>New +</Button>
      </div>
      {#each rows as row, index (row.id)}
        {@const envIssueKey = processId ? `process.${processId}.env.${row.id}.key` : `env.${row.id}.key`}
        {@const keyError = issueFor(envIssueKey)}
        {@const keyErrorId = `env-error-${row.id}-key`}
        {@const rowNumber = index + 1}
        {@const scopeLabel = envScopeLabel(processId)}
        <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)_auto] md:items-start">
          <div class="grid gap-1">
            <TextField
              density="compact"
              aria-label={`${scopeLabel} environment variable key ${rowNumber}`}
              aria-invalid={keyError ? "true" : undefined}
              aria-describedby={keyError ? keyErrorId : undefined}
              class={keyError ? errorFieldClass : ""}
              placeholder="KEY"
              bind:value={row.key}
              onblur={() => onFieldBlur?.(processId ? `process.${processId}.env.${row.id}.key` : `env.${row.id}.key`)}
            />
            {#if keyError}
              <span id={keyErrorId} class="text-xs text-danger">{keyError}</span>
            {/if}
          </div>
          <TextField
            density="compact"
            aria-label={`${scopeLabel} environment variable value ${rowNumber}`}
            placeholder="value"
            bind:value={row.value}
            onblur={() => onFieldBlur?.(processId ? `process.${processId}.env.${row.id}.value` : `env.${row.id}.value`)}
          />
          <IconButton
            label={`Remove ${scopeLabel.toLowerCase()} environment variable ${rowNumber}`}
            variant="ghost"
            size="sm"
            onclick={() => onRemove(row.id)}
            class="mt-1"
          >
            <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
              <path d="M6 2.75h4M3.75 4.5h8.5M6.5 6.5v5M9.5 6.5v5M5.5 2.75l.35-.6A.75.75 0 0 1 6.5 1.75h3a.75.75 0 0 1 .65.4l.35.6m-6 1.75V12a1.25 1.25 0 0 0 1.25 1.25h4.5A1.25 1.25 0 0 0 11.5 12V4.5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </IconButton>
        </div>
      {/each}
    {/if}
  </div>
</section>
