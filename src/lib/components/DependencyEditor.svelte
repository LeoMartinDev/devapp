<script lang="ts">
  import type { DependencyCondition } from "$lib/types";
  import type { ProcessForm } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import SelectField from "$lib/components/ui/SelectField.svelte";

  type Props = {
    process: ProcessForm;
    processes: ProcessForm[];
    dependencyIssue: (process: ProcessForm, dependencyId: string) => string | null;
    onAdd: (process: ProcessForm) => void;
    onRemove: (process: ProcessForm, dependencyId: string) => void;
    onFieldBlur?: (key: string) => void;
  };

  let {
    process,
    processes,
    dependencyIssue,
    onAdd,
    onRemove,
    onFieldBlur,
  }: Props = $props();

  const errorFieldClass = "border-danger focus:border-danger";
</script>

<section class="grid gap-3">
  <div class="flex items-start justify-between gap-3">
    <div>
      <h3 class="text-sm font-semibold text-text">Dependencies</h3>
      <p class="mt-1 text-xs leading-5 text-text-subtle">Process launch order for this node.</p>
    </div>
  </div>

  {#if process.dependencies.length === 0}
    <div class="flex items-center justify-between gap-3 px-1 text-sm text-text-subtle">
      <span>Starts without dependencies.</span>
      <Button size="sm" onclick={() => onAdd(process)}>Add dependency</Button>
    </div>
  {:else}
    <div class="flex justify-end mb-1">
      <Button size="sm" onclick={() => onAdd(process)}>Add dependency</Button>
    </div>
    <div class="grid gap-2.5">
      {#each process.dependencies as dependency, index (dependency.id)}
        {@const depError = dependencyIssue(process, dependency.id)}
        {@const depErrorId = `dependency-error-${process.id}-${dependency.id}`}
        {@const dependencyIndex = index + 1}
        <div class="grid gap-1">
          <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_150px_auto] md:items-start">
            <SelectField
              aria-label={`Dependency process ${dependencyIndex}`}
              density="compact"
              aria-invalid={depError ? "true" : undefined}
              aria-describedby={depError ? depErrorId : undefined}
              class={depError ? errorFieldClass : ""}
              options={[
                { value: "", label: "Select process" },
                ...processes.filter((candidate) => candidate.id !== process.id).map((candidate) => ({
                  value: candidate.name,
                  label: candidate.name,
                })),
              ]}
              bind:value={dependency.processName}
              onblur={() => onFieldBlur?.(`process.${process.id}.dependency.${dependency.id}`)}
            />
            <SelectField
              aria-label={`Dependency condition ${dependencyIndex}`}
              density="compact"
              options={[
                { value: "ready" satisfies DependencyCondition, label: "Ready" },
                { value: "success" satisfies DependencyCondition, label: "Success" },
              ]}
              bind:value={dependency.condition}
              onblur={() => onFieldBlur?.(`process.${process.id}.dependency.${dependency.id}`)}
            />
            <IconButton
              label={`Remove dependency ${dependencyIndex}`}
              variant="ghost"
              size="sm"
              onclick={() => onRemove(process, dependency.id)}
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
          {#if depError}
            <span id={depErrorId} class="text-xs text-danger">{depError}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</section>
