<script lang="ts">
  import type { ProcessForm as ProcessFormState } from "$lib/config/editorModel";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import SelectField from "$lib/components/ui/SelectField.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";

  type Props = {
    process: ProcessFormState;
    processCount: number;
    processIssue: (process: ProcessFormState, field: string) => string | null;
    onRemove: (id: string) => void;
    onFieldBlur?: (key: string) => void;
  };

  let { process, processCount, processIssue, onRemove, onFieldBlur }: Props = $props();
</script>

<section class="grid gap-4 border-t border-border/70 pt-5">
  <div class="flex items-start justify-between gap-3">
    <div>
      <h3 class="text-sm font-semibold text-text">Process</h3>
      <p class="mt-1 text-xs leading-5 text-text-subtle">Name, command, and kind for the selected runtime node.</p>
    </div>
    <IconButton
      label="Remove process"
      variant="ghost"
      size="sm"
      onclick={() => onRemove(process.id)}
      disabled={processCount <= 1}
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

  <div class="grid grid-cols-1 gap-3 md:grid-cols-[220px_minmax(0,1fr)]">
  <TextField
    label="Name"
    error={processIssue(process, "name")}
    bind:value={process.name}
    onblur={() => onFieldBlur?.(`process.${process.id}.name`)}
  />
  <TextField
    label="Command"
    placeholder="deno task dev"
    monospace
    error={processIssue(process, "cmd")}
    bind:value={process.cmd}
    onblur={() => onFieldBlur?.(`process.${process.id}.cmd`)}
  />
  </div>

  <div class="grid grid-cols-1 gap-3 md:grid-cols-[220px_minmax(0,1fr)]">
    <SelectField
      label="Kind"
      options={[
        { value: "service", label: "Service" },
        { value: "task", label: "Task" },
      ]}
      bind:value={process.kind}
      onblur={() => onFieldBlur?.(`process.${process.id}.kind`)}
    />
  </div>
</section>
