<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type ShoppingListItem = {
    inventory_item_id: number;
    item_name: string;
    supply_type: string;
    brand: string;
    color_name: string;
    color_code: string;
    inventory_quantity: number;
    allocated_quantity: number;
    shortage_quantity: number;
    unit: string;
  };

  let items: ShoppingListItem[] = [];
  let error = "";

  async function loadShoppingList() {
    error = "";

    try {
      items = await invoke<ShoppingListItem[]>("get_shopping_list");
    } catch (e) {
      console.error(e);
      error = String(e);
    }
  }

  onMount(loadShoppingList);
</script>

<section>
  <h3>Shopping list</h3>

  <button on:click={loadShoppingList}>Refresh shopping list</button>

  {#if error}
    <p>{error}</p>
  {/if}

  {#if items.length === 0}
    <p>No shortages based on current project allocations.</p>
  {:else}
    <ul>
      {#each items as item}
        <li>
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
          Have: {item.inventory_quantity} {item.unit}
          <br />
          Allocated: {item.allocated_quantity} {item.unit}
          <br />
          Need: {item.shortage_quantity} {item.unit}
        </li>
      {/each}
    </ul>
  {/if}
</section>