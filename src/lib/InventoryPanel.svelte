<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  import {
    DATA_CHANGED_EVENT,
    notifyDataChanged
  } from "$lib/appEvents";

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

  let items: InventoryItem[] = [];
  let error = "";

  let name = "";
  let supplyType = "yarn";
  let brand = "";
  let colorName = "";
  let colorCode = "";
  let quantity = 1;
  let unit = "skein";
  let notes = "";

  let editingItemId: number | null = null;
  let editName = "";
  let editSupplyType = "yarn";
  let editBrand = "";
  let editColorName = "";
  let editColorCode = "";
  let editQuantity = 1;
  let editUnit = "";
  let editNotes = "";

  async function loadItems() {
    items = await invoke<InventoryItem[]>("list_inventory_items");
  }

  async function addItem() {
    error = "";

    if (!name.trim()) {
      error = "Item name is required.";
      return;
    }

    try {
      await invoke<InventoryItem>("add_inventory_item", {
        name: name.trim(),
        supplyType,
        brand,
        colorName,
        colorCode,
        quantity: Number(quantity),
        unit,
        notes
      });

      notifyDataChanged();

      name = "";
      brand = "";
      colorName = "";
      colorCode = "";
      quantity = 1;
      unit = "skein";
      notes = "";

      await loadItems();
    } catch (e) {
      console.error("Failed to add inventory item:", e);
      error = String(e);
    }
  }

  async function deleteItem(itemId: number) {
    error = "";

    try {
      await invoke("delete_inventory_item", { itemId });

      notifyDataChanged();

      await loadItems();
    } catch (e) {
      console.error("Failed to delete inventory item:", e);
      error = String(e);
    }
  }

  function startEditingItem(item: InventoryItem) {
    editingItemId = item.id;
    editName = item.name;
    editSupplyType = item.supply_type;
    editBrand = item.brand;
    editColorName = item.color_name;
    editColorCode = item.color_code;
    editQuantity = item.quantity;
    editUnit = item.unit;
    editNotes = item.notes;
  }

  function cancelEditingItem() {
    editingItemId = null;
    editName = "";
    editSupplyType = "yarn";
    editBrand = "";
    editColorName = "";
    editColorCode = "";
    editQuantity = 1;
    editUnit = "";
    editNotes = "";
  }

  async function saveInventoryEdit(itemId: number) {
    error = "";

    if (!editName.trim()) {
      error = "Item name is required.";
      return;
    }

    try {
      await invoke<InventoryItem>("update_inventory_item", {
        itemId,
        name: editName.trim(),
        supplyType: editSupplyType,
        brand: editBrand,
        colorName: editColorName,
        colorCode: editColorCode,
        quantity: Number(editQuantity),
        unit: editUnit,
        notes: editNotes
      });

      notifyDataChanged();

      cancelEditingItem();
      await loadItems();
    } catch (e) {
      console.error("Failed to update inventory item:", e);
      error = String(e);
    }
  }

  onMount(() => {
    loadItems();

    window.addEventListener(DATA_CHANGED_EVENT, loadItems);

    return () => {
      window.removeEventListener(DATA_CHANGED_EVENT, loadItems);
    };
  });
</script>

<section>
  <h3>Inventory</h3>

  <form on:submit|preventDefault={addItem}>
    <input bind:value={name} placeholder="Item name" />

    <select bind:value={supplyType}>
      <option value="yarn">Yarn</option>
      <option value="floss">Floss</option>
      <option value="fabric">Fabric</option>
      <option value="needle">Needle</option>
      <option value="notion">Notion</option>
      <option value="other">Other</option>
    </select>

    <input bind:value={brand} placeholder="Brand" />
    <input bind:value={colorName} placeholder="Color name" />
    <input bind:value={colorCode} placeholder="Color code" />

    <input type="number" min="0" step="0.01" bind:value={quantity} />

    <input bind:value={unit} placeholder="Unit, e.g. skein, metre, piece" />

    <input bind:value={notes} placeholder="Notes" />

    <button type="submit">Add inventory item</button>
  </form>

  {#if error}
    <p>{error}</p>
  {/if}

  {#if items.length === 0}
    <p>No inventory items yet.</p>
  {:else}
    <ul>
      {#each items as item}
        <li>
          {#if editingItemId === item.id}
            <input bind:value={editName} />

            <select bind:value={editSupplyType}>
              <option value="yarn">Yarn</option>
              <option value="floss">Floss</option>
              <option value="fabric">Fabric</option>
              <option value="needle">Needle</option>
              <option value="notion">Notion</option>
              <option value="other">Other</option>
            </select>

            <input bind:value={editBrand} placeholder="Brand" />
            <input bind:value={editColorName} placeholder="Color name" />
            <input bind:value={editColorCode} placeholder="Color code" />
            <input type="number" min="0" step="0.01" bind:value={editQuantity} />
            <input bind:value={editUnit} placeholder="Unit" />
            <input bind:value={editNotes} placeholder="Notes" />

            <button on:click={() => saveInventoryEdit(item.id)}>Save</button>
            <button on:click={cancelEditingItem}>Cancel</button>
          {:else}
            <strong>{item.name}</strong>
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
            Quantity: {item.quantity} {item.unit}

            {#if item.notes}
              <br />
              Notes: {item.notes}
            {/if}

            <br />

            <button on:click={() => startEditingItem(item)}>Edit</button>
            <button on:click={() => deleteItem(item.id)}>Remove</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>