<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";

  import {
    DATA_CHANGED_EVENT,
    notifyDataChanged
  } from "$lib/appEvents";

  export let projectId: number;

  type Counter = {
    id: number;
    label: string;
    value: number;
    counter_type: string;
  };

  type Pattern = {
    id: number;
    file_path: string;
  };

  type Session = {
    id: number;
    ended_at: string | null;
  };

  let counters: Counter[] = [];
  let patterns: Pattern[] = [];
  let sessions: Session[] = [];

  async function loadData() {
    counters = await invoke<Counter[]>("list_counters", {
      projectId
    });

    patterns = await invoke<Pattern[]>("list_patterns", {
      projectId
    });

    sessions = await invoke<Session[]>("list_sessions", {
      projectId
    });
  }

  function getMainRowCounter() {
    return counters.find((c) => c.counter_type === "row");
  }

  function getActiveSession() {
    return sessions.find((s) => s.ended_at === null);
  }

  async function incrementRow(delta: number) {
    const rowCounter = getMainRowCounter();

    if (!rowCounter) return;

    await invoke("increment_counter", {
      counterId: rowCounter.id,
      delta
    });

    notifyDataChanged();

    await loadData();
  }

  async function openPrimaryPattern() {
    if (patterns.length === 0) return;

    await openPath(patterns[0].file_path);
  }

  async function startSession() {
    await invoke("start_session", {
      projectId
    });

    notifyDataChanged();

    await loadData();
  }

  async function endSession() {
    const session = getActiveSession();

    if (!session) return;

    await invoke("end_session", {
      sessionId: session.id,
      notes: "",
      currentPage: 1,
      positionNote: "",
      patternId: null,
      counterSnapshotJson: JSON.stringify(counters)
    });

    notifyDataChanged();

    await loadData();
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
  <h3>Quick actions</h3>

  {#if getMainRowCounter()}
    <div class="toolbar-group">
      <strong>
        {getMainRowCounter()?.label}:
        {getMainRowCounter()?.value}
      </strong>

      <div>
        <button on:click={() => incrementRow(-1)}>
          -1
        </button>

        <button on:click={() => incrementRow(1)}>
          +1
        </button>
      </div>
    </div>
  {/if}

  <div class="toolbar-group">
    <button on:click={openPrimaryPattern}>
      Open pattern
    </button>

    {#if getActiveSession()}
      <button on:click={endSession}>
        End session
      </button>
    {:else}
      <button on:click={startSession}>
        Start session
      </button>
    {/if}
  </div>
</section>

<style>
  .toolbar-group {
    margin-bottom: 1rem;
  }

  button {
    margin-right: 0.5rem;
    margin-top: 0.5rem;
  }
</style>