<script lang="ts">
  import { runtimeStore } from "$lib/stores/runtime.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  const recentProjects = $derived(
    runtimeStore.projects.filter((project) => project.id !== runtimeStore.projectId),
  );

  function openCreate() {
    document.dispatchEvent(new CustomEvent("devapp:open-create-dialog"));
  }
</script>

<div class="grid h-full place-items-center px-6 text-center">
  <div class="max-w-sm">
    <div class="mb-4 text-4xl">&#9654;</div>
    <h2 class="text-base font-semibold text-text">Welcome to Devapp</h2>
    <p class="mt-2 text-sm text-text-muted">
      Register a project to start supervising your dev environment.
    </p>
    <div class="mt-6">
      <Button variant="primary" onclick={openCreate}>Register project</Button>
    </div>
    {#if recentProjects.length > 0}
      <div class="mt-8 border-t border-border pt-6 text-left">
        <div class="mb-3 text-[11px] font-semibold uppercase tracking-wider text-text-subtle">
          Recent projects
        </div>
        <div class="grid gap-1">
          {#each recentProjects as project}
            <button
              type="button"
              class="w-full rounded-md px-3 py-2 text-left text-[13px] transition-colors duration-75 hover:bg-surface-hover"
              onclick={() => {
                document.dispatchEvent(
                  new CustomEvent("devapp:open-edit-dialog", { detail: project }),
                );
              }}
            >
              <span class="text-text">{project.name}</span>
              <span class="ml-2 text-xs text-text-subtle">{project.baseDir}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
