<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  import {
  DATA_CHANGED_EVENT,
  notifyDataChanged
} from "$lib/appEvents";

  export let projectId: number;

  type Counter = {
    id: number;
    project_id: number;
    label: string;
    value: number;
    counter_type: string;
    sort_order: number;
    updated_at: string;
  };

  let counters: Counter[] = [];
  let label = "Row";
  let error = "";

  let editingCounterId: number | null = null;
  let editLabel = "";
  let editValue = 0;
  let editCounterType = "row";

  async function loadCounters() {
    counters = await invoke<Counter[]>("list_counters", {
      projectId
    });
  }

  async function createCounter() {
    error = "";

    if (!label.trim()) {
      error = "Counter label is required.";
      return;
    }

    await invoke<Counter>("create_counter", {
      projectId,
      label: label.trim(),
      counterType: "row"
    });

    notifyDataChanged();

    label = "Row";
    await loadCounters();
  }

  async function changeCounter(counterId: number, delta: number) {
    const updated = await invoke<Counter>("increment_counter", {
      counterId,
      delta
    });

    notifyDataChanged();

    counters = counters.map((counter) =>
      counter.id === updated.id ? updated : counter
    );
  }

  function startEditingCounter(counter: Counter) {
    editingCounterId = counter.id;
    editLabel = counter.label;
    editValue = counter.value;
    editCounterType = counter.counter_type;
  }

  function cancelEditingCounter() {
    editingCounterId = null;
    editLabel = "";
    editValue = 0;
    editCounterType = "row";
  }

  async function saveCounterEdit(counterId: number) {
    error = "";

    if (!editLabel.trim()) {
      error = "Counter label is required.";
      return;
    }

    await invoke<Counter>("update_counter", {
      counterId,
      label: editLabel.trim(),
      value: Number(editValue),
      counterType: editCounterType
    });

    notifyDataChanged();

    cancelEditingCounter();
    await loadCounters();
  }

  async function resetCounter(counter: Counter) {
    await invoke<Counter>("update_counter", {
      counterId: counter.id,
      label: counter.label,
      value: 0,
      counterType: counter.counter_type
    });

    notifyDataChanged();

    await loadCounters();
  }

  async function deleteCounter(counterId: number) {
    const confirmed = confirm("Delete this counter?");
    if (!confirmed) return;

    await invoke("delete_counter", { counterId });

    notifyDataChanged();

    await loadCounters();
  }

  onMount(() => {
  loadCounters();

  window.addEventListener(DATA_CHANGED_EVENT, loadCounters);

  return () => {
    window.removeEventListener(DATA_CHANGED_EVENT, loadCounters);
  };
});
</script>

<section>
  <h3>Counters</h3>

  <form on:submit|preventDefault={createCounter}>
    <input bind:value={label} placeholder="Counter label" />
    <button type="submit">Add counter</button>
  </form>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if counters.length === 0}
    <p>No counters yet.</p>
  {:else}
    <ul>
      {#each counters as counter}
        <li>
          {#if editingCounterId === counter.id}
            <input bind:value={editLabel} />
            <input type="number" bind:value={editValue} />

            <select bind:value={editCounterType}>
              <option value="row">Row</option>
              <option value="round">Round</option>
              <option value="repeat">Repeat</option>
              <option value="stitch">Stitch</option>
              <option value="custom">Custom</option>
            </select>

            <button on:click={() => saveCounterEdit(counter.id)}>Save</button>
            <button on:click={cancelEditingCounter}>Cancel</button>
          {:else}
            <strong>{counter.label}</strong>: {counter.value}

            <button on:click={() => changeCounter(counter.id, -1)}>-</button>
            <button on:click={() => changeCounter(counter.id, 1)}>+</button>
            <button on:click={() => startEditingCounter(counter)}>Edit</button>
            <button on:click={() => resetCounter(counter)}>Reset</button>
            <button on:click={() => deleteCounter(counter.id)}>Delete</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>