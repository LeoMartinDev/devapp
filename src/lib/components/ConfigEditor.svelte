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
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import type { ProjectRecord } from "$lib/types";
  import { untrack } from "svelte";

  type Mode = "panel" | "page";

  const sectionNavItems = [
    { id: "settings-general", label: "General" },
    { id: "settings-environment", label: "Environment" },
    { id: "settings-processes", label: "Processes" },
    { id: "settings-preview", label: "YAML preview" },
  ] as const;

  type SectionId = (typeof sectionNavItems)[number]["id"];

  type Props = {
    open: boolean;
    project: ProjectRecord | null;
    onClose: () => void;
    mode?: Mode;
  };

  let { open, project, onClose, mode = "panel" }: Props = $props();

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
  let touchedFields = $state(new Set<string>());
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let dirtyClosePending = $state(false);
  let activeSection = $state<SectionId>("settings-general");
  let pageScrollContainer = $state<HTMLElement | null>(null);
  let generalSection = $state<HTMLElement | null>(null);
  let environmentSection = $state<HTMLElement | null>(null);
  let processesSection = $state<HTMLElement | null>(null);
  let previewSection = $state<HTMLElement | null>(null);

  const prefersReducedMotion = typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window.matchMedia("(prefers-reduced-motion: reduce)").matches
    : false;

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
  const panelLayoutClass =
    "grid h-[min(760px,calc(100vh-160px))] min-h-0 grid-cols-1 grid-rows-[auto_minmax(0,1fr)] overflow-hidden lg:grid-cols-[260px_minmax(0,1fr)] lg:grid-rows-none";
  const projectSourceLabel = $derived(
    !project
      ? "Auto-detect"
      : project.configSource === "projectFile"
        ? "Project file"
        : "App config file",
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
    touchedFields = new Set();
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
        for (const issue of validationIssues) {
          touchedFields.add(issue.key);
        }
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

  function isTouched(key: string) {
    return touchedFields.has(key);
  }

  function issueFor(key: string) {
    const issue = validationIssues.find((issue) => issue.key === key);
    if (!issue) return null;
    if (!isTouched(key)) return null;
    return issue.message;
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

  function requestClose() {
    if (isDirty && !loadError) {
      dirtyClosePending = true;
    } else {
      onClose();
    }
  }

  function sectionElement(sectionId: SectionId) {
    return sectionEntries().find((section) => section.id === sectionId)?.element ?? null;
  }

  function sectionEntries() {
    return [
      { ...sectionNavItems[0], element: generalSection },
      { ...sectionNavItems[1], element: environmentSection },
      { ...sectionNavItems[2], element: processesSection },
      { ...sectionNavItems[3], element: previewSection },
    ] as const;
  }

  function settingsNavClass(sectionId: SectionId) {
    return activeSection === sectionId
      ? "bg-surface-raised/70 text-text"
      : "text-text-subtle hover:bg-surface-hover/70 hover:text-text";
  }

  function navigateToSection(event: MouseEvent, sectionId: SectionId) {
    event.preventDefault();
    activeSection = sectionId;
    const element = sectionElement(sectionId);
    if (typeof window !== "undefined") {
      window.history.replaceState(null, "", `#${sectionId}`);
    }
    element?.scrollIntoView?.({
      behavior: prefersReducedMotion ? "auto" : "smooth",
      block: "start",
      inline: "nearest",
    });
  }

  function syncActiveSectionFromScroll() {
    if (!pageScrollContainer) {
      return;
    }

    const containerTop = pageScrollContainer.getBoundingClientRect().top;
    let nextSection: SectionId = "settings-general";

    for (const section of sectionEntries()) {
      const { id, element } = section;
      if (!element) {
        continue;
      }
      if (element.getBoundingClientRect().top - containerTop <= 48) {
        nextSection = id;
      } else {
        break;
      }
    }

    activeSection = nextSection;
  }

  function processOptionId(processId: string) {
    return `settings-process-option-${processId}`;
  }

  function processPanelId(processId: string) {
    return `settings-process-panel-${processId}`;
  }

  function focusProcessOption(processId: string) {
    if (typeof document === "undefined") {
      return;
    }
    document.getElementById(processOptionId(processId))?.focus();
  }

  function selectProcessByIndex(index: number) {
    const process = processes[index];
    if (!process) {
      return;
    }
    selectedProcessId = process.id;
    focusProcessOption(process.id);
  }

  function handleProcessOptionKeydown(event: KeyboardEvent, processId: string) {
    const currentIndex = processes.findIndex((process) => process.id === processId);
    if (currentIndex === -1) {
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowRight") {
      event.preventDefault();
      selectProcessByIndex((currentIndex + 1) % processes.length);
      return;
    }

    if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
      event.preventDefault();
      selectProcessByIndex((currentIndex - 1 + processes.length) % processes.length);
      return;
    }

    if (event.key === "Home") {
      event.preventDefault();
      selectProcessByIndex(0);
      return;
    }

    if (event.key === "End") {
      event.preventDefault();
      selectProcessByIndex(processes.length - 1);
    }
  }

  function markTouched(key: string) {
    touchedFields.add(key);
  }

  $effect(() => {
    if (!open || mode !== "page") {
      return;
    }
    activeSection = "settings-general";
  });

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
    if (!open || loadedProjectId === null) return;
    if (untrack(() => suppressDirty)) return;
    isDirty = true;
  });

  $effect(() => {
    void version;
    void globalEnvRows;
    void processes;
    if (!open || !loadedProjectId) return;

    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      validationIssues = validateConfigForm(formState).issues;
    }, 300);

    return () => {
      if (debounceTimer !== null) {
        clearTimeout(debounceTimer);
      }
    };
  });
</script>

{#snippet panelBody()}
  <div class={panelLayoutClass}>
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
              onFieldBlur={markTouched}
            />
            {#if selectedProcess}
              <section class="grid gap-4">
                <ProcessForm
                  process={selectedProcess}
                  processCount={processes.length}
                  {processIssue}
                  onRemove={removeProcess}
                  onFieldBlur={markTouched}
                />

                <EnvEditor
                  bind:rows={selectedProcess.envRows}
                  processId={selectedProcess.id}
                  {issueFor}
                  onAdd={() => addEnvRow(selectedProcess)}
                  onRemove={(id) => removeEnvRow(selectedProcess, id)}
                  onFieldBlur={markTouched}
                />

                <div class="grid grid-cols-1 gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
                  <div class="hidden lg:block"></div>
                  <DependencyEditor
                    process={selectedProcess}
                    {processes}
                    {dependencyIssue}
                    onAdd={addDependency}
                    onRemove={removeDependency}
                    onFieldBlur={markTouched}
                  />
                </div>

                <ReadyCheckEditor process={selectedProcess} {readyIssue} onFieldBlur={markTouched} />
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
                <pre data-native-selectable="yaml-preview" class="max-h-72 overflow-auto rounded-md border border-border bg-surface-raised/70 p-3 font-mono text-xs leading-5 text-text-muted">{previewYaml}</pre>
              {/if}
            </section>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/snippet}

{#snippet pageBody()}
  <div class="grid h-full min-h-0 grid-cols-1 overflow-hidden lg:grid-cols-[248px_minmax(0,1fr)]">
    <aside class="min-h-0 border-b border-border bg-surface/90 lg:border-b-0 lg:border-r">
      <div class="flex h-full min-h-0 flex-col">
        <div class="border-b border-border px-4 py-4">
          <div class="text-[11px] font-semibold uppercase tracking-[0.18em] text-text-subtle">Settings</div>
          <div class="mt-3 text-sm font-semibold text-text">{project?.name ?? "No project selected"}</div>
          <div class="mt-1 text-xs leading-5 text-text-subtle">{project?.baseDir ?? "Open a project to edit its runtime configuration."}</div>
        </div>

        <nav aria-label="Settings sections" class="border-b border-border px-3 py-3">
          <div class="grid gap-1">
            {#each sectionNavItems as section (section.id)}
              <a
                class={`rounded-md px-3 py-1.5 text-[13px] transition-colors duration-75 ${settingsNavClass(section.id)}`}
                href={`#${section.id}`}
                aria-current={activeSection === section.id ? "page" : undefined}
                onclick={(event) => navigateToSection(event, section.id)}
              >{section.label}</a>
            {/each}
          </div>
        </nav>

        <div class="min-h-0 flex-1 px-3 py-3">
          <div class="mb-2 flex items-center justify-between px-1">
            <div class="text-[11px] font-semibold uppercase tracking-[0.18em] text-text-subtle">Processes</div>
            <Button size="sm" onclick={addProcess} disabled={loading || loadError !== null}>Add</Button>
          </div>

          <div role="listbox" aria-label="Processes" aria-orientation="vertical" class="flex min-h-0 gap-2 overflow-x-auto lg:block lg:h-full lg:overflow-y-auto">
            {#each processes as process (process.id)}
              <button
                id={processOptionId(process.id)}
                type="button"
                role="option"
                class={`flex min-w-44 items-center justify-between gap-2 rounded-md px-3 py-1.5 text-left text-[13px] transition-colors duration-75 lg:mb-1 lg:w-full lg:min-w-0 ${
                  process.id === selectedProcess?.id
                    ? "bg-surface-raised/70 text-text"
                    : "text-text-subtle hover:bg-surface-hover/70 hover:text-text"
                }`}
                aria-selected={process.id === selectedProcess?.id ? "true" : "false"}
                tabindex={process.id === selectedProcess?.id ? 0 : -1}
                onclick={() => (selectedProcessId = process.id)}
                onkeydown={(event) => handleProcessOptionKeydown(event, process.id)}
              >
                <span class="min-w-0 truncate">{process.name || "Unnamed process"}</span>
                <span class="shrink-0 text-[11px] text-text-subtle">{process.kind}</span>
              </button>
            {/each}
          </div>
        </div>
      </div>
    </aside>

    <div bind:this={pageScrollContainer} class="min-h-0 overflow-y-auto bg-canvas" onscroll={syncActiveSectionFromScroll}>
      <div class="mx-auto flex w-full max-w-245 flex-col gap-6 px-5 py-6 lg:px-8 lg:py-8">
        {#if loading}
          <div class="text-sm text-text-subtle">Loading settings...</div>
        {:else if loadError}
          <div class="grid gap-3 rounded-xl border border-danger/40 bg-danger/10 p-4 text-sm">
            <div class="font-medium text-danger">Configuration could not be loaded</div>
            <div class="text-danger/80">{loadError}</div>
            <div class="text-text-subtle">
              The settings form is disabled so an empty generated configuration cannot overwrite the project YAML.
            </div>
          </div>
        {:else}
          <section bind:this={generalSection} id="settings-general" class="grid gap-4">
            <div>
              <h2 class="text-base font-semibold text-text">General</h2>
              <p class="mt-1 text-sm leading-6 text-text-subtle">Project-scoped runtime settings for the current workspace.</p>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
              <div class="rounded-lg border border-border/70 bg-surface/40 px-4 py-3">
                <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-subtle">Project</div>
                <div class="mt-2 text-sm font-medium text-text">{project?.name ?? "No project selected"}</div>
                <div class="mt-1 wrap-break-word text-xs leading-5 text-text-subtle">{project?.baseDir ?? "Select a project to load its runtime config."}</div>
              </div>

              <div class="rounded-lg border border-border/70 bg-surface/40 px-4 py-3">
                <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-subtle">Config source</div>
                <div class="mt-2 text-sm font-medium text-text">{projectSourceLabel}</div>
                <div class="mt-1 wrap-break-word text-xs leading-5 text-text-subtle">{project?.configPath ?? "The backend will resolve devapp.yml when a project is selected."}</div>
              </div>
            </div>
          </section>

          <section bind:this={environmentSection} id="settings-environment" class="grid gap-4 border-t border-border/70 pt-6">
            <div>
              <h2 class="text-base font-semibold text-text">Environment</h2>
              <p class="mt-1 text-sm leading-6 text-text-subtle">Shared variables are injected into every configured process.</p>
            </div>

            <EnvEditor
              bind:rows={globalEnvRows}
              {issueFor}
              onAdd={addGlobalEnvRow}
              onRemove={removeGlobalEnvRow}
              onFieldBlur={markTouched}
            />
          </section>

          <section bind:this={processesSection} id="settings-processes" class="grid gap-5 border-t border-border/70 pt-6">
            <div class="flex items-start justify-between gap-3">
              <div>
                <h2 class="text-base font-semibold text-text">Processes</h2>
                <p class="mt-1 text-sm leading-6 text-text-subtle">Select a process in the left rail to edit command, env, dependencies, and readiness.</p>
              </div>
              <Button size="sm" onclick={addProcess} disabled={loading || loadError !== null}>Add process</Button>
            </div>

            {#if selectedProcess}
              <div
                id={processPanelId(selectedProcess.id)}
                role="region"
                aria-labelledby={processOptionId(selectedProcess.id)}
                class="grid gap-0"
              >
                <ProcessForm
                  process={selectedProcess}
                  processCount={processes.length}
                  {processIssue}
                  onRemove={removeProcess}
                  onFieldBlur={markTouched}
                />

                <EnvEditor
                  bind:rows={selectedProcess.envRows}
                  processId={selectedProcess.id}
                  {issueFor}
                  onAdd={() => addEnvRow(selectedProcess)}
                  onRemove={(id) => removeEnvRow(selectedProcess, id)}
                  onFieldBlur={markTouched}
                />

                <DependencyEditor
                  process={selectedProcess}
                  {processes}
                  {dependencyIssue}
                  onAdd={addDependency}
                  onRemove={removeDependency}
                  onFieldBlur={markTouched}
                />

                <ReadyCheckEditor process={selectedProcess} {readyIssue} onFieldBlur={markTouched} />
              </div>
            {:else}
              <div class="px-1 text-sm text-text-subtle">
                Add a process to start building the runtime graph.
              </div>
            {/if}
          </section>

          <section bind:this={previewSection} id="settings-preview" class="grid gap-4 border-t border-border/70 pt-6">
            <div class="flex items-center justify-between gap-3">
              <div>
                <h2 class="text-base font-semibold text-text">YAML preview</h2>
                <p class="mt-1 text-sm leading-6 text-text-subtle">Preview the generated devapp.yml before saving it back to the project.</p>
              </div>
              <Button size="sm" onclick={() => (showPreview = !showPreview)}>
                {showPreview ? "Hide preview" : "Preview YAML"}
              </Button>
            </div>
            {#if showPreview}
              <pre data-native-selectable="yaml-preview" class="max-h-72 overflow-auto rounded-md border border-border/70 bg-surface/40 p-3 font-mono text-xs leading-5 text-text-muted">{previewYaml}</pre>
            {/if}
          </section>
        {/if}
      </div>
    </div>
  </div>
{/snippet}

{#snippet editorFooter()}
  <div class="flex items-center justify-between gap-3">
    <div class="text-xs text-text-subtle">
      {#if formIssueCount > 0}
        <span class="text-danger">{formIssueCount} validation issue{formIssueCount === 1 ? "" : "s"} must be fixed before saving.</span>
      {:else if isDirty}
        Unsaved changes in project YAML
      {:else}
        {status ?? "Changes are saved to the project YAML."}
      {/if}
    </div>
    <div class="flex gap-2">
      <Button onclick={requestClose} disabled={saving}>
        Cancel
      </Button>
      <Button variant="primary" onclick={save} disabled={!project || saving || loadError !== null}>
        Save
      </Button>
    </div>
  </div>
{/snippet}

{#if open && mode === "page"}
  <section class="flex h-full min-h-0 flex-col bg-canvas text-text">
    <header class="border-b border-border bg-canvas/95 px-5 py-4">
      <button
        type="button"
        class="inline-flex items-center gap-2 rounded-md px-2 py-1.5 text-sm font-medium text-text-subtle transition-colors duration-75 hover:bg-surface-hover hover:text-text"
        onclick={requestClose}
      >
        <svg viewBox="0 0 16 16" class="h-4 w-4" aria-hidden="true">
          <path d="M9.5 3.5L5 8l4.5 4.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span>Go back</span>
      </button>

      <div class="mt-3 flex items-start justify-between gap-4">
        <div class="min-w-0">
          <h1 class="text-lg font-semibold text-text">Runtime configuration</h1>
          <p class="mt-1 text-sm leading-6 text-text-subtle">{dialogDescription}</p>
        </div>
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-hidden">
      {@render pageBody()}
    </div>

    <footer class="border-t border-border bg-surface px-5 py-4">
      {@render editorFooter()}
    </footer>
  </section>
{:else}
  <Dialog
    {open}
    title="Runtime configuration"
    description={dialogDescription}
    size="xl"
    variant="panel"
    onClose={requestClose}
    closeOnOverlay={!saving}
  >
    {@render panelBody()}

    {#snippet footer()}
      {@render editorFooter()}
    {/snippet}
  </Dialog>
{/if}

<ConfirmDialog
  open={dirtyClosePending}
  title="Unsaved changes"
  message="You have unsaved changes. Discard them?"
  confirmLabel="Discard"
  cancelLabel="Keep editing"
  onConfirm={() => {
    dirtyClosePending = false;
    onClose();
  }}
  onClose={() => {
    dirtyClosePending = false;
  }}
/>