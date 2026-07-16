<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import CounterPanel from "$lib/CounterPanel.svelte";
  import SessionPanel from "$lib/SessionPanel.svelte";
  import PatternPanel from "$lib/PatternPanel.svelte";
  import InventoryPanel from "$lib/InventoryPanel.svelte";
  import ProjectInventoryPanel from "$lib/ProjectInventoryPanel.svelte";
  import ProjectSummaryPanel from "$lib/ProjectSummaryPanel.svelte";
  import ShoppingListPanel from "$lib/ShoppingListPanel.svelte";
  import QuickActionsPanel from "$lib/QuickActionsPanel.svelte";
  import ActiveSessionBanner from "$lib/ActiveSessionBanner.svelte";
  import CollapsiblePanel from "$lib/CollapsiblePanel.svelte";

  import {
    DATA_CHANGED_EVENT,
    notifyDataChanged
  } from "$lib/appEvents";

  type Project = {
    id: number;
    name: string;
    craft_type: string;
    status: string;
    created_at: string;
  };

  let projects: Project[] = [];
  let activeProject: Project | null = null;

  let name = "";
  let craftType = "knitting";
  let error = "";

  let editingProjectId: number | null = null;
  let editName = "";
  let editCraftType = "knitting";
  let editStatus = "active";

  let projectSearch = "";
  let projectStatusFilter = "active";

  $: filteredProjects = projects.filter((project) => {
    const matchesSearch = project.name
      .toLowerCase()
      .includes(projectSearch.toLowerCase());

    const matchesStatus =
      projectStatusFilter === "all" || project.status === projectStatusFilter;

    return matchesSearch && matchesStatus;
  });

  async function loadProjects() {
    projects = await invoke<Project[]>("list_projects");
    activeProject = await invoke<Project | null>("get_active_project");
  }

  async function exportMobile() {
  error = "";

  try {
    const result = await invoke<string>("export_mobile_data");
    console.log(result);
    alert(result);
  } catch (e) {
    console.error("Failed to export mobile data:", e);
    error = String(e);
  }
}

  async function createProject() {
    error = "";

    if (!name.trim()) {
      error = "Project name is required.";
      return;
    }

    try {
      const project = await invoke<Project>("create_project", {
        name: name.trim(),
        craftType
      });

      await invoke("set_active_project", {
        projectId: project.id
      });

      notifyDataChanged();

      name = "";
      await loadProjects();
    } catch (e) {
      console.error("Failed to create project:", e);
      error = String(e);
    }
  }

  async function selectProject(project: Project) {
    await invoke("set_active_project", {
      projectId: project.id
    });

    notifyDataChanged();

    activeProject = project;
  }

  function startEditingProject(project: Project) {
    editingProjectId = project.id;
    editName = project.name;
    editCraftType = project.craft_type;
    editStatus = project.status;
  }

  function cancelEditingProject() {
    editingProjectId = null;
    editName = "";
    editCraftType = "knitting";
    editStatus = "active";
  }

  async function saveProjectEdit(projectId: number) {
    error = "";

    if (!editName.trim()) {
      error = "Project name is required.";
      return;
    }

      await invoke("update_project", {
    projectId,
    name: editName.trim(),
    craftType: editCraftType,
    status: editStatus
  });

  notifyDataChanged();

    cancelEditingProject();
    await loadProjects();
  }

  async function deleteProject(projectId: number) {
    error = "";

    const confirmed = confirm(
      "Delete this project? This will also remove its counters, sessions, patterns, and supply links."
    );

    if (!confirmed) return;

    await invoke("delete_project", { projectId });

  notifyDataChanged();
    await loadProjects();
  }

  async function setProjectStatus(project: Project, status: string) {
  await invoke("update_project", {
    projectId: project.id,
    name: project.name,
    craftType: project.craft_type,
    status
  });

  notifyDataChanged();

  await loadProjects();
}

onMount(() => {
  loadProjects();

  window.addEventListener(DATA_CHANGED_EVENT, loadProjects);

  return () => {
    window.removeEventListener(DATA_CHANGED_EVENT, loadProjects);
  };
});
</script>

<div class="dashboard">
  <aside class="sidebar">
    <div class="card">
  <h2>Craft Companion</h2>

  <button on:click={exportMobile}>
    Export Mobile Data
  </button>

  <CollapsiblePanel title="Projects" open={true}>
        <input bind:value={projectSearch} placeholder="Search projects..." />

        <select bind:value={projectStatusFilter}>
          <option value="active">Active</option>
          <option value="paused">Paused</option>
          <option value="finished">Finished</option>
          <option value="archived">Archived</option>
          <option value="all">All</option>
        </select>

        {#if filteredProjects.length === 0}
          <p>No matching projects.</p>
        {:else}
          <ul class="project-list">
            {#each filteredProjects as project}
              <li>
                {#if editingProjectId === project.id}
                  <input bind:value={editName} />

                  <select bind:value={editCraftType}>
                    <option value="knitting">Knitting</option>
                    <option value="crochet">Crochet</option>
                    <option value="cross_stitch">Cross stitch</option>
                    <option value="sewing">Sewing</option>
                    <option value="other">Other</option>
                  </select>

                  <select bind:value={editStatus}>
                    <option value="active">Active</option>
                    <option value="paused">Paused</option>
                    <option value="finished">Finished</option>
                    <option value="archived">Archived</option>
                  </select>

                  <button on:click={() => saveProjectEdit(project.id)}>
                    Save
                  </button>

                  <button on:click={cancelEditingProject}>
                    Cancel
                  </button>
                {:else}
                  <button
                    class:selected={activeProject?.id === project.id}
                    class="project-button"
                    on:click={() => selectProject(project)}
                  >
                    <strong>{project.name}</strong>
                    <br />

                    <small>
                      {project.craft_type} · {project.status}
                    </small>

                    {#if activeProject?.id === project.id}
                      ✓
                    {/if}
                  </button>

                  <div class="project-actions">
                    <button on:click={() => startEditingProject(project)}>
                      Edit
                    </button>

                    {#if project.status !== "active"}
                      <button on:click={() => setProjectStatus(project, "active")}>
                        Reactivate
                      </button>
                    {/if}

                    {#if project.status !== "paused"}
                      <button on:click={() => setProjectStatus(project, "paused")}>
                        Pause
                      </button>
                    {/if}

                    {#if project.status !== "finished"}
                      <button on:click={() => setProjectStatus(project, "finished")}>
                        Finish
                      </button>
                    {/if}

                    {#if project.status !== "archived"}
                      <button on:click={() => setProjectStatus(project, "archived")}>
                        Archive
                      </button>
                    {/if}

                    <button on:click={() => deleteProject(project.id)}>
                      Delete
                    </button>
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}

        <h3>Add project</h3>

        <form on:submit|preventDefault={createProject}>
          <input bind:value={name} placeholder="Project name" />

          <select bind:value={craftType}>
            <option value="knitting">Knitting</option>
            <option value="crochet">Crochet</option>
            <option value="cross_stitch">Cross stitch</option>
            <option value="sewing">Sewing</option>
            <option value="other">Other</option>
          </select>

          <button type="submit">Create project</button>
        </form>

        {#if error}
          <p class="error">{error}</p>
        {/if}
      </CollapsiblePanel>
    </div>

    <div class="card">
      <CollapsiblePanel title="Inventory">
        <InventoryPanel />
      </CollapsiblePanel>
    </div>

    <div class="card">
      <CollapsiblePanel title="Shopping list">
        <ShoppingListPanel />
      </CollapsiblePanel>
    </div>
  </aside>

  <main class="main-panel">
    {#if activeProject}
      <div class="card">
        <p>
          Active project:
          <strong>{activeProject.name}</strong>
          ({activeProject.craft_type})
        </p>
      </div>

      <div class="card">
        <ActiveSessionBanner projectId={activeProject.id} />
      </div>

      <div class="card">
        <QuickActionsPanel projectId={activeProject.id} />
      </div>

      <div class="card">
        <CounterPanel projectId={activeProject.id} />
      </div>

      <div class="card">
        <CollapsiblePanel title="Patterns" open={true}>
          <PatternPanel projectId={activeProject.id} />
        </CollapsiblePanel>
      </div>

      <div class="card">
        <CollapsiblePanel title="Sessions">
          <SessionPanel projectId={activeProject.id} />
        </CollapsiblePanel>
      </div>
    {:else}
      <div class="card">
        <p>No active project selected.</p>
      </div>
    {/if}
  </main>

  <aside class="right-rail">
    {#if activeProject}
      <div class="card">
        <ProjectSummaryPanel projectId={activeProject.id} />
      </div>

      <div class="card">
        <CollapsiblePanel title="Project supplies" open={true}>
          <ProjectInventoryPanel projectId={activeProject.id} />
        </CollapsiblePanel>
      </div>
    {/if}
  </aside>
</div>

<style>
 .dashboard {
  display: grid;
  grid-template-columns: 300px minmax(520px, 1fr) 360px;
  gap: 1rem;
  width: 100%;
  align-items: start;
}

.right-rail {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

  .main-panel {
    min-width: 0;
    width: 100%;
  }

  .sidebar,
  .main-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .card {
    padding: 1rem;
    overflow: hidden;
  }

  :global(section) {
    border: none;
    padding: 0;
    margin: 0;
  }

  .project-list {
    padding-left: 0;
    list-style: none;
  }

  .project-list li {
    margin-bottom: 0.75rem;
  }

  .project-button {
    width: 100%;
    text-align: left;
    margin-bottom: 0.25rem;
  }

  .project-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .selected {
    border: 2px solid #888;
  }

  input,
  select,
  textarea {
    display: block;
    margin: 0.25rem 0;
    padding: 0.35rem;
    max-width: 100%;
    box-sizing: border-box;
  }

  button {
    margin-top: 0.25rem;
    margin-right: 0.35rem;
  }

  .error {
    color: crimson;
  }

@media (max-width: 900px) {
  .dashboard {
    display: flex;
    flex-direction: column;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    gap: 0.65rem;
  }

  .sidebar,
  .main-panel,
  .right-rail {
    display: flex;
    flex-direction: column;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    gap: 0.65rem;
  }

  .card {
    width: 100%;
    max-width: 100%;
    min-width: 0;
    padding: 0.7rem;
    overflow: hidden;
  }

  .project-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.4rem;
  }

  .project-actions button {
    width: 100%;
    margin: 0;
    white-space: normal;
  }

  input,
  select,
  textarea,
  button {
    max-width: 100%;
    min-width: 0;
  }
}
</style>

@media (max-width: 800px) and (max-height: 520px) {
  .dashboard,
  .sidebar,
  .main-panel,
  .right-rail {
    gap: 0.45rem;
  }

  .card {
    padding: 0.55rem;
    border-radius: 12px;
  }

  h2 {
    font-size: 1.2rem;
    margin-bottom: 0.5rem;
  }

  h3 {
    font-size: 1rem;
    margin-bottom: 0.5rem;
  }

  .project-actions {
    grid-template-columns: 1fr;
  }

  button {
    padding: 0.65rem 0.75rem;
  }
}
