<script lang="ts">
  import type { ProcessForm as ProcessFormState } from "$lib/config/editorModel";
  import Button from "$lib/components/ui/Button.svelte";
  import SelectField from "$lib/components/ui/SelectField.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";

  type Props = {
    process: ProcessFormState;
    processCount: number;
    processIssue: (process: ProcessFormState, field: string) => string | null;
    onRemove: (id: string) => void;
  };

  let { process, processCount, processIssue, onRemove }: Props = $props();
</script>

<div class="flex items-center justify-between border-t border-border pt-5">
  <h2 class="text-sm font-semibold text-text">Process</h2>
  <Button
    variant="danger"
    size="sm"
    onclick={() => onRemove(process.id)}
    disabled={processCount <= 1}
  >
    Remove process
  </Button>
</div>

<div class="grid grid-cols-1 gap-3 md:grid-cols-[220px_minmax(0,1fr)]">
  <TextField
    label="Name"
    class="h-10"
    error={processIssue(process, "name")}
    bind:value={process.name}
  />
  <TextField
    label="Command"
    class="h-10"
    placeholder="deno task dev"
    monospace
    error={processIssue(process, "cmd")}
    bind:value={process.cmd}
  />
</div>

<div class="grid grid-cols-1 gap-3 md:grid-cols-[220px_minmax(0,1fr)]">
  <SelectField
    label="Kind"
    class="h-10"
    options={[
      { value: "service", label: "Service" },
      { value: "task", label: "Task" },
    ]}
    bind:value={process.kind}
  />
</div>
