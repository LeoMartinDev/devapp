<script lang="ts">
  import { validateProjectDetails, type ValidationIssue } from "$lib/config/validation";
  import Button from "$lib/components/ui/Button.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import SelectField from "$lib/components/ui/SelectField.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";
  import type { ProjectRecord, ProjectSource } from "$lib/types";
  import type { SaveProjectInput } from "$lib/tauri/client";

  type Props = {
    open: boolean;
    project: ProjectRecord | null;
    onClose: () => void;
    onSave: (input: SaveProjectInput) => Promise<void>;
    onRemove: (project: ProjectRecord) => Promise<void>;
    launchLocked: boolean;
  };

  let { open, project, onClose, onSave, onRemove, launchLocked }: Props = $props();

  let name = $state("");
  let baseDir = $state("");
  let configSource = $state<ProjectSource | "">("");
  let saving = $state(false);
  let removing = $state(false);
  let confirmRemoveOpen = $state(false);
  let validationIssues = $state<ValidationIssue[]>([]);
  const configSourceOptions = $derived([
    ...(!project ? [{ value: "", label: "Auto-detect devapp.yml" }] : []),
    { value: "projectFile", label: "Project file" },
    { value: "appConfigFile", label: "App config file" },
  ]);

  $effect(() => {
    if (!open) {
      return;
    }
    name = project?.name ?? "";
    baseDir = project?.baseDir ?? "";
    configSource = project?.configSource ?? "";
    validationIssues = [];
    confirmRemoveOpen = false;
  });

  async function submit() {
    const validation = validateProjectDetails({ name, baseDir, configSource });
    validationIssues = validation.issues;
    if (!validation.valid) {
      return;
    }
    saving = true;
    try {
      await onSave({
        id: project?.id,
        name,
        baseDir,
        ...(configSource ? { configSource } : {}),
      });
      onClose();
    } finally {
      saving = false;
    }
  }

  function issueFor(key: string) {
    return validationIssues.find((issue) => issue.key === key)?.message ?? null;
  }

  function closeDialog() {
    if (!confirmRemoveOpen) {
      onClose();
    }
  }

  async function removeProject() {
    if (!project) {
      return;
    }
    removing = true;
    try {
      await onRemove(project);
      confirmRemoveOpen = false;
      onClose();
    } finally {
      removing = false;
    }
  }
</script>

<Dialog
  {open}
  title={project ? "Edit project" : "Register project"}
  size="sm"
  variant="panel"
  onClose={closeDialog}
  closeOnOverlay={!saving && !removing}
>
  <div class="space-y-4 overflow-y-auto px-4 py-4">
    <TextField
      label="Name"
      density="compact"
      error={issueFor("project.name")}
      bind:value={name}
    />

    <TextField
      label="Base directory"
      placeholder="/path/to/project"
      density="compact"
      error={issueFor("project.baseDir")}
      bind:value={baseDir}
    />

    <label class="block space-y-1.5 text-sm">
      <SelectField
        label="Configuration source"
        density="compact"
        options={configSourceOptions}
        error={issueFor("project.configSource")}
        bind:value={configSource}
      />
      <span class="block text-xs leading-5 text-text-subtle">
        {configSource === ""
          ? "If devapp.yml exists in the base directory, the backend will use it."
          : configSource === "projectFile"
            ? "Use devapp.yml inside the project directory."
            : "Store this project's config in the app data directory."}
      </span>
    </label>
  </div>

  {#snippet footer()}
    <div class="flex items-center justify-between gap-3">
      <div>
        {#if project && !launchLocked}
          <Button size="sm" variant="danger" onclick={() => (confirmRemoveOpen = true)} disabled={saving || removing}>
            Remove project
          </Button>
        {/if}
      </div>
      <div class="flex justify-end gap-2">
        <Button size="sm" onclick={onClose} disabled={saving || removing}>
          Cancel
        </Button>
        <Button size="sm" variant="primary" onclick={submit} disabled={saving || removing}>
          Save project
        </Button>
      </div>
    </div>
  {/snippet}
</Dialog>

<ConfirmDialog
  open={confirmRemoveOpen}
  title="Remove project"
  message={`Remove "${project?.name ?? "this project"}" from devapp? This does not delete project files.`}
  confirmLabel="Remove project"
  busy={removing}
  onConfirm={removeProject}
  onClose={() => (confirmRemoveOpen = false)}
/>
