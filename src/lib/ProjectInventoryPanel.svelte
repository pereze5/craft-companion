<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let projectId: number;

  type InventoryItem = {
    id: number;
    name: string;
    supply_type: string;
    brand: string;
    color_name: string;
    color_code: string;
    quantity: number;
    unit: string;
    notes: string;
    created_at: string;
    updated_at: string;
  };

  type ProjectInventoryItem = {
    id: number;
    project_id: number;
    inventory_item_id: number;
    quantity_allocated: number;
    notes: string;
    item_name: string;
    supply_type: string;
    brand: string;
    color_name: string;
    color_code: string;
    unit: string;
  };

  let inventoryItems: InventoryItem[] = [];
  let projectItems: ProjectInventoryItem[] = [];
  let selectedItemId = "";
  let quantityAllocated = 1;
  let notes = "";
  let error = "";
  let editingLinkId: number | null = null;
  let editQuantityAllocated = 1;
  let editNotes = "";

  async function loadData() {
    inventoryItems = await invoke<InventoryItem[]>("list_inventory_items");
    projectItems = await invoke<ProjectInventoryItem[]>("list_project_inventory", {
      projectId
    });
  }

  async function linkItem() {
    error = "";

    if (!selectedItemId) {
      error = "Select an inventory item.";
      return;
    }

    try {
      await invoke("link_inventory_to_project", {
        projectId,
        inventoryItemId: Number(selectedItemId),
        quantityAllocated: Number(quantityAllocated),
        notes
      });

      selectedItemId = "";
      quantityAllocated = 1;
      notes = "";

      await loadData();
    } catch (e) {
      console.error(e);
      error = String(e);
    }
  }

  async function removeLink(projectInventoryId: number) {
    error = "";

    try {
      await invoke("remove_project_inventory_link", {
        projectInventoryId
      });

      await loadData();
    } catch (e) {
      console.error(e);
      error = String(e);
    }
  }


  function startEditingLink(item: ProjectInventoryItem) {
  editingLinkId = item.id;
  editQuantityAllocated = item.quantity_allocated;
  editNotes = item.notes;
}

function cancelEditingLink() {
  editingLinkId = null;
  editQuantityAllocated = 1;
  editNotes = "";
}

async function saveLinkEdit(projectInventoryId: number) {
  error = "";

  try {
    await invoke("update_project_inventory_link", {
      projectInventoryId,
      quantityAllocated: Number(editQuantityAllocated),
      notes: editNotes
    });

    cancelEditingLink();
    await loadData();
  } catch (e) {
    console.error(e);
    error = String(e);
  }
}

  onMount(() => {
  loadData();

  window.addEventListener(DATA_CHANGED_EVENT, loadData);

  return () => {
    window.removeEventListener(DATA_CHANGED_EVENT, loadData);
  };
});
</script>

<section>
  <h3>Project supplies</h3>

  <form on:submit|preventDefault={linkItem}>
    <select bind:value={selectedItemId}>
      <option value="">Select inventory item</option>
      {#each inventoryItems as item}
        <option value={item.id}>
          {item.name} ({item.supply_type}) — {item.quantity} {item.unit}
        </option>
      {/each}
    </select>

    <input
      type="number"
      min="0"
      step="0.01"
      bind:value={quantityAllocated}
    />

    <input bind:value={notes} placeholder="Allocation notes" />

    <button type="submit">Link supply to project</button>
  </form>

  {#if error}
    <p>{error}</p>
  {/if}

  {#if projectItems.length === 0}
    <p>No supplies linked to this project yet.</p>
  {:else}
    <ul>
      {#each projectItems as item}
  <li>
    {#if editingLinkId === item.id}
      <strong>{item.item_name}</strong>
      <br />

      <input
        type="number"
        min="0"
        step="0.01"
        bind:value={editQuantityAllocated}
      />

      <input bind:value={editNotes} placeholder="Allocation notes" />

      <button on:click={() => saveLinkEdit(item.id)}>Save</button>
      <button on:click={cancelEditingLink}>Cancel</button>
    {:else}
      <strong>{item.item_name}</strong>
      ({item.supply_type})
      <br />
      {item.brand}
      {#if item.color_name}
        — {item.color_name}
      {/if}
      {#if item.color_code}
        [{item.color_code}]
      {/if}
      <br />
      Allocated: {item.quantity_allocated} {item.unit}
      {#if item.notes}
        <br />
        Notes: {item.notes}
      {/if}
      <br />
      <button on:click={() => startEditingLink(item)}>Edit</button>
      <button on:click={() => removeLink(item.id)}>Remove link</button>
    {/if}
  </li>
{/each}
    </ul>
  {/if}
</section>