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
import { runtimeStore } from "$lib/stores/runtime.svelte";
  import type { ProjectRecord } from "$lib/types";
  import { untrack } from "svelte";

  type Props = {
    open: boolean;
    project: ProjectRecord | null;
    onClose: () => void;
  };

  let { open, project, onClose }: Props = $props();

  let version = $state(1);
  let globalEnvRows = $state<EnvRow[]>([]);
  let processes = $state<ProcessFormState[]>([]);
  let selectedProcessId = $state<string | null>(null);
  let loadedProjectId = $state<string | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let status = $state<string | null>(null);
  let loadError = $state<string | null>(null);
  let dialogWasOpen = $state(false);
  let validationIssues = $state<ValidationIssue[]>([]);
  let showPreview = $state(false);
  let isDirty = $state(false);
  let suppressDirty = $state(false);

  const selectedProcess = $derived(
    processes.find((process) => process.id === selectedProcessId) ?? processes[0] ?? null,
  );

  const formState = $derived<ConfigFormState>({
    version,
    globalEnvRows,
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
    globalEnvRows = [];
    processes = [newProcess("api")];
    selectedProcessId = processes[0].id;
  }

  function resetUnloaded() {
    version = 1;
    globalEnvRows = [];
    processes = [];
    selectedProcessId = null;
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
    validationIssues = [];
    showPreview = false;
    try {
      const document = await runtimeStore.loadConfig(projectId);
      const config = document?.config;
      if (!config) {
        suppressDirty = true;
        resetEmpty();
        suppressDirty = false;
        isDirty = false;
        loadedProjectId = projectId;
        return;
      }
      suppressDirty = true;
      version = config.version;
      globalEnvRows = Object.entries(config.env ?? {}).map(([key, value]) => ({
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
      suppressDirty = false;
      isDirty = false;
      loadedProjectId = projectId;
    } catch (error) {
      loadError = errorMessage(error);
      loadedProjectId = null;
      resetUnloaded();
    } finally {
      loading = false;
    }
  }

  function addEnvRow(process: ProcessFormState) {
    process.envRows = [...process.envRows, { id: nextId("env"), key: "", value: "" }];
  }

  function removeEnvRow(process: ProcessFormState, rowId: string) {
    process.envRows = process.envRows.filter((row) => row.id !== rowId);
  }

  function addGlobalEnvRow() {
    globalEnvRows = [...globalEnvRows, { id: nextId("env"), key: "", value: "" }];
  }

  function removeGlobalEnvRow(rowId: string) {
    globalEnvRows = globalEnvRows.filter((row) => row.id !== rowId);
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
    try {
      const validation = validateConfigForm(formState);
      validationIssues = validation.issues;
      if (!validation.valid) {
        return;
      }

      const yaml = previewYaml;
      await runtimeStore.saveConfig(yaml, project.id);
      suppressDirty = true;
      isDirty = false;
      suppressDirty = false;
      status = "Settings saved";
      onClose();
    } catch (error) {
      status = errorMessage(error);
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

  function envRowIssue(process: ProcessFormState, rowId: string) {
    return issueFor(`process.${process.id}.env.${rowId}.key`);
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

  $effect(() => {
    void version;
    void globalEnvRows;
    void processes;
    if (untrack(() => suppressDirty)) return;
    isDirty = true;
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
        {:else}
          <div class="grid gap-5">
            <EnvEditor
              bind:rows={globalEnvRows}
              {issueFor}
              onAdd={addGlobalEnvRow}
              onRemove={removeGlobalEnvRow}
            />
            {#if selectedProcess}
              <section class="grid gap-4">
                <ProcessForm
                  process={selectedProcess}
                  processCount={processes.length}
                  {processIssue}
                  onRemove={removeProcess}
                />

                <EnvEditor
                  bind:rows={selectedProcess.envRows}
                  processId={selectedProcess.id}
                  {issueFor}
                  onAdd={() => addEnvRow(selectedProcess)}
                  onRemove={(id) => removeEnvRow(selectedProcess, id)}
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
        {#if formIssueCount > 0}
          <span class="text-danger">{formIssueCount} validation issue{formIssueCount === 1 ? "" : "s"} must be fixed before saving.</span>
        {:else}
          {status ?? "Changes are saved to the project YAML."}
        {/if}
        {#if isDirty}
          <span class="text-warning"> Unsaved changes</span>
        {/if}
      </div>
      <div class="flex gap-2">
        <Button onclick={onClose} disabled={saving}>
          Cancel
        </Button>
        <Button variant="primary" onclick={save} disabled={!project || saving || loadError !== null}>
          Save{isDirty ? " *" : " settings"}
        </Button>
      </div>
    </div>
  {/snippet}
</Dialog>
