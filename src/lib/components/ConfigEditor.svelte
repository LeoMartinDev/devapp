<script lang="ts">
  import {
    buildConfig as buildConfigFromForm,
    createProcess,
    serializeConfig,
    toProcessForm,
    type ConfigFormState,
    type EnvRow,
    type ProcessForm as ProcessFormState,
  } from "$lib/config/editorModel";
  import {
    validateConfigForm,
    type ValidationIssue,
  } from "$lib/config/validation";
  import ConfigProcessList from "$lib/components/ConfigProcessList.svelte";
  import DependencyEditor from "$lib/components/DependencyEditor.svelte";
  import EnvEditor from "$lib/components/EnvEditor.svelte";
  import ProcessForm from "$lib/components/ProcessForm.svelte";
  import ReadyCheckEditor from "$lib/components/ReadyCheckEditor.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import type { ProjectRecord } from "$lib/types";

  type Props = {
    open: boolean;
    project: ProjectRecord | null;
    onClose: () => void;
  };

  let { open, project, onClose }: Props = $props();

  let version = $state(1);
  let envRows = $state<EnvRow[]>([]);
  let processes = $state<ProcessFormState[]>([]);
  let selectedProcessId = $state<string | null>(null);
  let loadedProjectId = $state<string | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let status = $state<string | null>(null);
  let loadError = $state<string | null>(null);
  let dialogWasOpen = $state(false);
  let mode = $state<"form" | "raw">("form");
  let rawYaml = $state("");
  let rawError = $state<string | null>(null);
  let validationIssues = $state<ValidationIssue[]>([]);
  let showPreview = $state(false);
  const modeOptions: { value: "form" | "raw"; label: string }[] = [
    { value: "form", label: "Form" },
    { value: "raw", label: "Raw YAML" },
  ];

  const selectedProcess = $derived(
    processes.find((process) => process.id === selectedProcessId) ?? processes[0] ?? null,
  );

  const formState = $derived<ConfigFormState>({
    version,
    envRows,
    processes,
  });

  const previewYaml = $derived(serializeConfig(buildConfigFromForm(formState)));
  const formIssueCount = $derived(validationIssues.length);
  const dialogDescription = $derived(
    project
      ? `Base directory: ${project.baseDir}. Config: ${project.configSource} - ${project.configPath}.`
      : "Select a project first.",
  );

  function nextId(prefix: string) {
    return `${prefix}-${crypto.randomUUID()}`;
  }

  function newProcess(name = "api") {
    return createProcess(name, nextId);
  }

  function resetEmpty() {
    version = 1;
    envRows = [];
    processes = [newProcess("api")];
    selectedProcessId = processes[0].id;
    rawYaml = serializeConfig(buildConfigFromForm({ version, envRows, processes }));
  }

  function resetUnloaded() {
    version = 1;
    envRows = [];
    processes = [];
    selectedProcessId = null;
    rawYaml = "";
  }

  function errorMessage(error: unknown) {
    if (error instanceof Error) {
      return error.message;
    }
    if (typeof error === "string") {
      return error;
    }
    return "Unable to load the project configuration.";
  }

  async function load(projectId: string) {
    loading = true;
    status = null;
    loadError = null;
    rawError = null;
    validationIssues = [];
    showPreview = false;
    try {
      const document = await runtimeStore.loadConfig(projectId);
      const config = document?.config;
      if (!config) {
        resetEmpty();
        loadedProjectId = projectId;
        return;
      }
      version = config.version;
      envRows = Object.entries(config.env ?? {}).map(([key, value]) => ({
        id: nextId("env"),
        key,
        value,
      }));
      processes = Object.entries(config.processes ?? {}).map(([name, process]) =>
        toProcessForm(name, process, nextId),
      );
      if (processes.length === 0) {
        processes = [newProcess("api")];
      }
      selectedProcessId = processes[0].id;
      rawYaml = document.yaml;
      loadedProjectId = projectId;
    } catch (error) {
      loadError = errorMessage(error);
      loadedProjectId = null;
      resetUnloaded();
    } finally {
      loading = false;
    }
  }

  function addEnvRow() {
    envRows = [...envRows, { id: nextId("env"), key: "", value: "" }];
  }

  function removeEnvRow(id: string) {
    envRows = envRows.filter((row) => row.id !== id);
  }

  function addProcess() {
    const name = uniqueProcessName("process");
    const process = newProcess(name);
    processes = [...processes, process];
    selectedProcessId = process.id;
  }

  function removeProcess(id: string) {
    const removedName = processes.find((process) => process.id === id)?.name;
    processes = processes.filter((process) => process.id !== id);
    if (removedName) {
      for (const process of processes) {
        process.dependencies = process.dependencies.filter(
          (dependency) => dependency.processName !== removedName,
        );
      }
    }
    selectedProcessId = processes[0]?.id ?? null;
  }

  function uniqueProcessName(base: string) {
    const used = new Set(processes.map((process) => process.name));
    if (!used.has(base)) {
      return base;
    }
    let index = 2;
    while (used.has(`${base}-${index}`)) {
      index += 1;
    }
    return `${base}-${index}`;
  }

  function addDependency(process: ProcessFormState) {
    const target = processes.find((candidate) => candidate.id !== process.id);
    process.dependencies = [
      ...process.dependencies,
      {
        id: nextId("dependency"),
        processName: target?.name ?? "",
        condition: "ready",
      },
    ];
  }

  function removeDependency(process: ProcessFormState, dependencyId: string) {
    process.dependencies = process.dependencies.filter((dependency) => dependency.id !== dependencyId);
  }

  async function save() {
    if (!project || loadError) {
      return;
    }
    saving = true;
    status = null;
    rawError = null;
    try {
      if (mode === "raw") {
        await runtimeStore.loadConfig(project.id, rawYaml);
        await runtimeStore.saveConfig(rawYaml, project.id);
        status = "Settings saved";
        onClose();
        return;
      }

      const validation = validateConfigForm(formState);
      validationIssues = validation.issues;
      if (!validation.valid) {
        return;
      }

      const yaml = previewYaml;
      await runtimeStore.saveConfig(yaml, project.id);
      status = "Settings saved";
      onClose();
    } catch (error) {
      if (mode === "raw") {
        rawError = errorMessage(error);
      } else {
        status = errorMessage(error);
      }
    } finally {
      saving = false;
    }
  }

  function issueFor(key: string) {
    return validationIssues.find((issue) => issue.key === key)?.message ?? null;
  }

  function processIssue(process: ProcessFormState, field: string) {
    return issueFor(`process.${process.id}.${field}`);
  }

  function readyIssue(process: ProcessFormState, field: string) {
    return issueFor(`process.${process.id}.ready.${field}`);
  }

  function dependencyIssue(process: ProcessFormState, dependencyId: string) {
    return issueFor(`process.${process.id}.dependency.${dependencyId}`);
  }

  function rawInputClass(error: string | null, extra = "") {
    const border = error ? "border-danger focus:border-danger" : "border-border focus:border-accent";
    return `rounded-md border ${border} bg-surface-raised px-3 outline-none transition-colors ${extra}`;
  }

  $effect(() => {
    const projectId = project?.id ?? null;
    if (!open) {
      dialogWasOpen = false;
      return;
    }
    if (!projectId) {
      return;
    }
    const openedNow = !dialogWasOpen;
    dialogWasOpen = true;

    if (openedNow || projectId !== loadedProjectId) {
      void load(projectId);
    }
  });
</script>

<Dialog
  {open}
  title="Runtime configuration"
  description={dialogDescription}
  size="xl"
  {onClose}
  closeOnOverlay={!saving}
>
  <div class="grid h-[min(760px,calc(100vh-160px))] min-h-0 grid-cols-1 grid-rows-[auto_minmax(0,1fr)] overflow-hidden lg:grid-cols-[260px_minmax(0,1fr)] lg:grid-rows-none">
    <ConfigProcessList
      projectName={project?.name ?? "No project"}
      {processes}
      {selectedProcess}
      {loading}
      disabled={loadError !== null}
      onAdd={addProcess}
      onSelect={(id) => (selectedProcessId = id)}
    />

    <div class="flex min-h-0 flex-col">
      <div class="flex justify-end border-b border-border px-5 py-3">
        <SegmentedControl
          value={mode}
          options={modeOptions}
          onChange={(value) => (mode = value as "form" | "raw")}
        />
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto px-5 py-5">
        {#if loading}
          <div class="text-sm text-text-subtle">Loading settings...</div>
        {:else if loadError}
          <div class="grid gap-3 rounded-md border border-danger/40 bg-danger/10 p-4 text-sm">
            <div class="font-medium text-danger">Configuration could not be loaded</div>
            <div class="text-danger/80">{loadError}</div>
            <div class="text-text-subtle">
              The settings form is disabled so an empty generated configuration cannot overwrite the project YAML.
            </div>
          </div>
        {:else if mode === "raw"}
          <div class="grid gap-3">
            <div class="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-warning/90">
              Raw YAML preserves comments and unsupported fields, but it is saved exactly as written after backend validation.
            </div>
            {#if rawError}
              <div class="rounded-md border border-danger/40 bg-danger/10 px-3 py-2 text-sm text-danger">
                {rawError}
              </div>
            {/if}
            <label class="grid gap-1.5 text-sm">
              <span class="text-text-subtle">YAML</span>
              <textarea
                class={rawInputClass(rawError, "min-h-[480px] resize-y py-3 font-mono text-[13px] leading-5 text-text")}
                spellcheck="false"
                bind:value={rawYaml}
              ></textarea>
            </label>
          </div>
        {:else}
          <div class="grid gap-5">
            <div class="rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-warning/90">
              Saving rewrites the generated YAML and may remove comments or unsupported fields.
            </div>
            <EnvEditor
              bind:rows={envRows}
              {issueFor}
              onAdd={addEnvRow}
              onRemove={removeEnvRow}
            />

            {#if selectedProcess}
              <section class="grid gap-4">
                <ProcessForm
                  process={selectedProcess}
                  processCount={processes.length}
                  {processIssue}
                  onRemove={removeProcess}
                />

                <div class="grid grid-cols-1 gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
                  <div class="hidden lg:block"></div>
                  <DependencyEditor
                    process={selectedProcess}
                    {processes}
                    {dependencyIssue}
                    onAdd={addDependency}
                    onRemove={removeDependency}
                  />
                </div>

                <ReadyCheckEditor process={selectedProcess} {readyIssue} />
              </section>
            {/if}

            <section class="grid gap-2 border-t border-border pt-5">
              <div class="flex items-center justify-between">
                <h2 class="text-sm font-semibold text-text">YAML preview</h2>
                <Button size="sm" onclick={() => (showPreview = !showPreview)}>
                  {showPreview ? "Hide preview" : "Preview YAML"}
                </Button>
              </div>
              {#if showPreview}
                <pre class="max-h-72 overflow-auto rounded-md border border-border bg-surface-raised/70 p-3 font-mono text-xs leading-5 text-text-muted">{previewYaml}</pre>
              {/if}
            </section>
          </div>
        {/if}
      </div>
    </div>
  </div>

  {#snippet footer()}
    <div class="flex items-center justify-between gap-3">
      <div class="text-xs text-text-subtle">
        {#if formIssueCount > 0 && mode === "form"}
          <span class="text-danger">{formIssueCount} validation issue{formIssueCount === 1 ? "" : "s"} must be fixed before saving.</span>
        {:else}
          {status ?? "Changes are saved to the project YAML."}
        {/if}
      </div>
      <div class="flex gap-2">
        <Button onclick={onClose} disabled={saving}>
          Cancel
        </Button>
        <Button variant="primary" onclick={save} disabled={!project || saving || loadError !== null}>
          Save settings
        </Button>
      </div>
    </div>
  {/snippet}
</Dialog>
