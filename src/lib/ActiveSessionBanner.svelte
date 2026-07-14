<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  import {
    DATA_CHANGED_EVENT,
    notifyDataChanged
  } from "$lib/appEvents";

  export let projectId: number;

  type Session = {
    id: number;
    project_id: number;
    started_at: string;
    ended_at: string | null;
    notes: string;
    current_page: number;
    position_note: string;
    counter_snapshot_json: string;
    pattern_id: number | null;
  };

  type Pattern = {
    id: number;
    display_name: string;
    current_page: number;
    last_position_note: string;
  };

  type Counter = {
    id: number;
    label: string;
    value: number;
    counter_type: string;
  };

  let sessions: Session[] = [];
  let patterns: Pattern[] = [];
  let counters: Counter[] = [];

  let now = new Date();
  let notes = "";
  let currentPage = 1;
  let positionNote = "";
  let selectedPatternId = "";
  let loadedSessionId: number | null = null;
  let error = "";

  async function loadData() {
    sessions = await invoke<Session[]>("list_sessions", { projectId });
    patterns = await invoke<Pattern[]>("list_patterns", { projectId });
    counters = await invoke<Counter[]>("list_counters", { projectId });
  }

  $: activeSession = sessions.find((s) => s.ended_at === null) ?? null;

  $: if (activeSession && loadedSessionId !== activeSession.id) {
    loadedSessionId = activeSession.id;
    notes = activeSession.notes;
    currentPage = activeSession.current_page || 1;
    positionNote = activeSession.position_note || "";
    selectedPatternId = activeSession.pattern_id
      ? String(activeSession.pattern_id)
      : "";
  }

  $: if (!activeSession) {
    loadedSessionId = null;
  }

  $: elapsedText = activeSession
    ? formatElapsed(activeSession.started_at, now)
    : "";

  function formatElapsed(startedAt: string, currentTime: Date) {
    const start = new Date(startedAt.replace(" ", "T"));
    const diffMs = currentTime.getTime() - start.getTime();

    if (Number.isNaN(diffMs) || diffMs < 0) return "unknown";

    const totalMinutes = Math.floor(diffMs / 60000);
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;

    return `${hours}h ${minutes}m`;
  }

  async function saveSessionNotes() {
    if (!activeSession) return;

    error = "";

    try {
      await invoke("update_session_notes", {
        sessionId: activeSession.id,
        notes
      });

      notifyDataChanged();

      await loadData();
    } catch (e) {
      console.error(e);
      error = String(e);
    }
  }

  async function endActiveSession() {
    if (!activeSession) return;

    error = "";

    try {
      await invoke("end_session", {
        sessionId: activeSession.id,
        notes,
        currentPage: Number(currentPage),
        positionNote,
        patternId: selectedPatternId ? Number(selectedPatternId) : null,
        counterSnapshotJson: JSON.stringify(counters)
      });

      notes = "";
      currentPage = 1;
      positionNote = "";
      selectedPatternId = "";
      loadedSessionId = null;

      notifyDataChanged();

      await loadData();
    } catch (e) {
      console.error(e);
      error = String(e);
    }
  }

  async function changeCounter(counterId: number, delta: number) {
    error = "";

    try {
      await invoke("increment_counter", {
        counterId,
        delta
      });

      notifyDataChanged();

      await loadData();
    } catch (e) {
      console.error(e);
      error = String(e);
    }
  }

  onMount(() => {
    loadData();

    window.addEventListener(DATA_CHANGED_EVENT, loadData);

    const timer = setInterval(() => {
      now = new Date();
    }, 60000);

    return () => {
      window.removeEventListener(DATA_CHANGED_EVENT, loadData);
      clearInterval(timer);
    };
  });
</script>

{#if activeSession}
  <section class="active-session">
    <h3>Active session</h3>

    <p>
      Started: <strong>{activeSession.started_at}</strong>
      <br />
      Elapsed: <strong>{elapsedText}</strong>
    </p>

    {#if counters.length > 0}
      <div class="work-counters">
        {#each counters as counter}
          <div
            class="work-counter"
            class:primary-counter={counter.counter_type === "row"}
          >
            <div class="counter-label">{counter.label}</div>
            <div class="counter-value">{counter.value}</div>

            <div class="counter-buttons">
              <button
                class="counter-button secondary"
                on:click={() => changeCounter(counter.id, -1)}
              >
                −1
              </button>

              <button
                class="counter-button primary"
                on:click={() => changeCounter(counter.id, 1)}
              >
                +1
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <label>
      Pattern:
      <select bind:value={selectedPatternId}>
        <option value="">No pattern selected</option>
        {#each patterns as pattern}
          <option value={pattern.id}>{pattern.display_name}</option>
        {/each}
      </select>
    </label>

    <label>
      Current page:
      <input type="number" min="1" bind:value={currentPage} />
    </label>

    <label>
      Current position:
      <input
        bind:value={positionNote}
        placeholder="e.g. Repeat from * to * a total of 7 times"
      />
    </label>

    <label>
      Session notes:
      <textarea bind:value={notes} placeholder="Session notes"></textarea>
    </label>

    <button on:click={saveSessionNotes}>Save notes</button>
    <button on:click={endActiveSession}>End session</button>

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </section>
{/if}

<style>
  .active-session {
    border: 2px solid currentColor;
    border-radius: 8px;
    padding: 1rem;
  }

  label {
    display: block;
    margin-top: 0.75rem;
  }

  input,
  select,
  textarea {
    width: 100%;
    box-sizing: border-box;
  }

  textarea {
    min-height: 4rem;
  }

  .error {
    color: crimson;
  }

  .work-counters {
    display: grid;
    gap: 1rem;
    margin: 1rem 0;
  }

  .work-counter {
    padding: 1rem;
    border: 1px solid currentColor;
    border-radius: 12px;
    text-align: center;
  }

  .primary-counter {
    border-width: 2px;
  }

  .counter-label {
    font-size: 1rem;
    opacity: 0.8;
  }

  .counter-value {
    font-size: 2.5rem;
    font-weight: 800;
    line-height: 1;
    margin: 0.5rem 0 1rem;
  }

  .primary-counter .counter-value {
    font-size: 4rem;
  }

  .counter-buttons {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 0.75rem;
  }

  .counter-button {
    font-size: 1.25rem;
    font-weight: 700;
    padding: 0.85rem;
  }

  .primary-counter .counter-button.primary {
    font-size: 2rem;
  }
</style>