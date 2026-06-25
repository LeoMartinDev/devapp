<script lang="ts">
  import type { ProcessForm } from "$lib/config/editorModel";
  import CheckboxField from "$lib/components/ui/CheckboxField.svelte";
  import SelectField from "$lib/components/ui/SelectField.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";

  type Props = {
    process: ProcessForm;
    readyIssue: (process: ProcessForm, field: string) => string | null;
    onFieldBlur?: (key: string) => void;
  };

  let { process, readyIssue, onFieldBlur }: Props = $props();
</script>

<section class="grid gap-3 rounded-md border border-border bg-surface-raised/40 p-4">
  <CheckboxField
    label="Enable readiness check"
    class="text-sm text-text-muted"
    bind:checked={process.readyEnabled}
    onblur={() => onFieldBlur?.(`process.${process.id}.ready.enabled`)}
  />

  {#if process.readyEnabled}
    <div class="grid grid-cols-1 gap-3 md:grid-cols-[180px_minmax(0,1fr)]">
      <SelectField
        label="Type"
        class="h-10"
        options={[
          { value: "http", label: "HTTP" },
          { value: "log", label: "Log" },
          { value: "delay", label: "Delay" },
          { value: "command", label: "Command" },
        ]}
        bind:value={process.readyType}
        onblur={() => onFieldBlur?.(`process.${process.id}.ready.type`)}
      />

      {#if process.readyType === "http"}
        <TextField
          label="URL"
          class="h-10"
          error={readyIssue(process, "httpUrl")}
          bind:value={process.httpUrl}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.httpUrl`)}
        />
      {:else if process.readyType === "log"}
        <div class="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,1fr)_120px]">
          <TextField
            label="Pattern"
            class="h-10"
            error={readyIssue(process, "logPattern")}
            bind:value={process.logPattern}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.logPattern`)}
          />
          <CheckboxField
            label="Regex"
            class="mt-7 text-sm text-text-muted"
            bind:checked={process.logRegex}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.logRegex`)}
          />
        </div>
      {:else if process.readyType === "delay"}
        <TextField
          label="Duration ms"
          type="number"
          min="0"
          class="h-10"
          error={readyIssue(process, "delayDurationMs")}
          bind:value={process.delayDurationMs}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.delayDurationMs`)}
        />
      {:else}
        <TextField
          label="Command"
          class="h-10"
          monospace
          error={readyIssue(process, "commandCmd")}
          bind:value={process.commandCmd}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.commandCmd`)}
        />
      {/if}
    </div>

    {#if process.readyType !== "delay"}
      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        {#if process.readyType === "http" || process.readyType === "command"}
          <TextField
            label="Interval ms"
            type="number"
            min="0"
            class="h-10"
            error={readyIssue(process, "intervalMs")}
            bind:value={process.intervalMs}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.intervalMs`)}
          />
        {/if}
        <TextField
          label="Timeout ms"
          type="number"
          min="0"
          class="h-10"
          error={readyIssue(process, "timeoutMs")}
          bind:value={process.timeoutMs}
          onblur={() => onFieldBlur?.(`process.${process.id}.ready.timeoutMs`)}
        />
      </div>
    {/if}
  {/if}
</section>
