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

  type SupportedReadyType = "http" | "log" | "delay" | "command";

  let { process, readyIssue, onFieldBlur }: Props = $props();

  let readyEnabled = $state(false);
  let readyType = $state<ProcessForm["readyType"]>("http");
  let httpUrl = $state("");
  let logPattern = $state("");
  let logRegex = $state(false);
  let delayDurationMs = $state<ProcessForm["delayDurationMs"]>(1000);
  let commandCmd = $state("");
  let intervalMs = $state<ProcessForm["intervalMs"]>(null);
  let timeoutMs = $state<ProcessForm["timeoutMs"]>(null);

  const supportedReadyTypeOptions: Array<{ value: SupportedReadyType; label: string }> = [
    { value: "http", label: "HTTP" },
    { value: "log", label: "Log" },
    { value: "delay", label: "Delay" },
    { value: "command", label: "Command" },
  ];

  $effect(() => {
    readyEnabled = process.readyEnabled;
    readyType = process.readyType;
    httpUrl = process.httpUrl;
    logPattern = process.logPattern;
    logRegex = process.logRegex;
    delayDurationMs = process.delayDurationMs;
    commandCmd = process.commandCmd;
    intervalMs = process.intervalMs;
    timeoutMs = process.timeoutMs;
  });

  function sanitizeIdPart(value: string): string {
    return value.replace(/[^a-zA-Z0-9_-]+/g, "-");
  }

  function readyErrorId(field: string): string {
    return `process-${sanitizeIdPart(process.id)}-ready-${field}-error`;
  }

  function readyFieldState(field: string): {
    message: string | null;
    describedBy?: string;
    invalid?: "true";
    className?: string;
  } {
    const message = readyIssue(process, field);

    return {
      message,
      describedBy: message ? readyErrorId(field) : undefined,
      invalid: message ? "true" : undefined,
      className: message ? "border-danger focus:border-danger" : undefined,
    };
  }

  const unknownReadyTypeMessage = "Unknown readiness type. Choose a supported type.";

  function isSupportedReadyType(value: string): value is SupportedReadyType {
    return supportedReadyTypeOptions.some((option) => option.value === value);
  }

  function readyTypeOptions(): Array<{ value: string; label: string }> {
    if (isSupportedReadyType(readyType)) {
      return supportedReadyTypeOptions;
    }

    return [
      { value: readyType, label: `Unsupported (${readyType})` },
      ...supportedReadyTypeOptions,
    ];
  }

  function readyTypeState(): {
    message: string | null;
    describedBy?: string;
    invalid?: "true";
    className?: string;
  } {
    const message = isSupportedReadyType(readyType) ? null : unknownReadyTypeMessage;

    return {
      message,
      describedBy: message ? readyErrorId("type") : undefined,
      invalid: message ? "true" : undefined,
      className: message ? "border-danger focus:border-danger" : undefined,
    };
  }

  function syncReadyEnabled() {
    process.readyEnabled = readyEnabled;
  }

  function syncReadyType() {
    process.readyType = readyType;
  }

  function syncHttpUrl() {
    process.httpUrl = httpUrl;
  }

  function syncLogPattern() {
    process.logPattern = logPattern;
  }

  function syncLogRegex() {
    process.logRegex = logRegex;
  }

  function syncDelayDurationMs() {
    process.delayDurationMs = delayDurationMs;
  }

  function syncCommandCmd() {
    process.commandCmd = commandCmd;
  }

  function syncIntervalMs() {
    process.intervalMs = intervalMs;
  }

  function syncTimeoutMs() {
    process.timeoutMs = timeoutMs;
  }
</script>

<section data-settings-surface="flat" class="grid gap-3">
  <div>
    <h3 class="text-sm font-semibold text-text">Readiness</h3>
    <p class="mt-1 text-xs leading-5 text-text-subtle">Optional startup probe for services that need an explicit ready signal.</p>
  </div>

  <CheckboxField
    label="Enable readiness check"
    class="text-[12px] text-text-muted"
    bind:checked={readyEnabled}
    onchange={syncReadyEnabled}
    onblur={() => onFieldBlur?.(`process.${process.id}.ready.enabled`)}
  />

  {#if readyEnabled}
    {@const typeState = readyTypeState()}
    <div class="grid grid-cols-1 gap-3 md:grid-cols-[180px_minmax(0,1fr)]">
      <SelectField
        label="Type"
        density="compact"
        options={readyTypeOptions()}
        class={typeState.className}
        aria-invalid={typeState.invalid}
        aria-describedby={typeState.describedBy}
        bind:value={readyType}
        onchange={syncReadyType}
        onblur={() => onFieldBlur?.(`process.${process.id}.ready.type`)}
      />

      {#if readyType === "http"}
        {@const httpUrlState = readyFieldState("httpUrl")}
        <div>
          <TextField
            label="URL"
            density="compact"
            class={httpUrlState.className}
            aria-invalid={httpUrlState.invalid}
            aria-describedby={httpUrlState.describedBy}
            bind:value={httpUrl}
            oninput={syncHttpUrl}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.httpUrl`)}
          />
          {#if httpUrlState.message}
            <span id={readyErrorId("httpUrl")} class="text-xs text-danger">{httpUrlState.message}</span>
          {/if}
        </div>
      {:else if readyType === "log"}
        {@const logPatternState = readyFieldState("logPattern")}
        <div class="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,1fr)_120px]">
          <div>
            <TextField
              label="Pattern"
              density="compact"
              class={logPatternState.className}
              aria-invalid={logPatternState.invalid}
              aria-describedby={logPatternState.describedBy}
              bind:value={logPattern}
              oninput={syncLogPattern}
              onblur={() => onFieldBlur?.(`process.${process.id}.ready.logPattern`)}
            />
            {#if logPatternState.message}
              <span id={readyErrorId("logPattern")} class="text-xs text-danger">{logPatternState.message}</span>
            {/if}
          </div>
          <CheckboxField
            label="Regex"
            class="mt-7 text-[12px] text-text-muted"
            bind:checked={logRegex}
            onchange={syncLogRegex}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.logRegex`)}
          />
        </div>
      {:else if readyType === "delay"}
        {@const delayDurationState = readyFieldState("delayDurationMs")}
        <div>
          <TextField
            label="Duration ms"
            density="compact"
            type="number"
            min="0"
            class={delayDurationState.className}
            aria-invalid={delayDurationState.invalid}
            aria-describedby={delayDurationState.describedBy}
            bind:value={delayDurationMs}
            oninput={syncDelayDurationMs}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.delayDurationMs`)}
          />
          {#if delayDurationState.message}
            <span id={readyErrorId("delayDurationMs")} class="text-xs text-danger">{delayDurationState.message}</span>
          {/if}
        </div>
      {:else if readyType === "command"}
        {@const commandState = readyFieldState("commandCmd")}
        <div>
          <TextField
            label="Command"
            density="compact"
            monospace
            class={commandState.className}
            aria-invalid={commandState.invalid}
            aria-describedby={commandState.describedBy}
            bind:value={commandCmd}
            oninput={syncCommandCmd}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.commandCmd`)}
          />
          {#if commandState.message}
            <span id={readyErrorId("commandCmd")} class="text-xs text-danger">{commandState.message}</span>
          {/if}
        </div>
      {:else}
        <p id={readyErrorId("type")} class="pt-2 text-xs text-danger">{typeState.message}</p>
      {/if}
    </div>

    {#if readyType === "http" || readyType === "log" || readyType === "command"}
      {@const timeoutState = readyFieldState("timeoutMs")}
      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        {#if readyType === "http" || readyType === "command"}
          {@const intervalState = readyFieldState("intervalMs")}
          <div>
            <TextField
              label="Interval ms"
              density="compact"
              type="number"
              min="0"
              class={intervalState.className}
              aria-invalid={intervalState.invalid}
              aria-describedby={intervalState.describedBy}
              bind:value={intervalMs}
              oninput={syncIntervalMs}
              onblur={() => onFieldBlur?.(`process.${process.id}.ready.intervalMs`)}
            />
            {#if intervalState.message}
              <span id={readyErrorId("intervalMs")} class="text-xs text-danger">{intervalState.message}</span>
            {/if}
          </div>
        {/if}
        <div>
          <TextField
            label="Timeout ms"
            density="compact"
            type="number"
            min="0"
            class={timeoutState.className}
            aria-invalid={timeoutState.invalid}
            aria-describedby={timeoutState.describedBy}
            bind:value={timeoutMs}
            oninput={syncTimeoutMs}
            onblur={() => onFieldBlur?.(`process.${process.id}.ready.timeoutMs`)}
          />
          {#if timeoutState.message}
            <span id={readyErrorId("timeoutMs")} class="text-xs text-danger">{timeoutState.message}</span>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</section>
