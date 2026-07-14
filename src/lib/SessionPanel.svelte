<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let projectId: number;

  type Session = {
    id: number;
    project_id: number;
    started_at: string;
    ended_at: string | null;
    notes: string;
  };

  let sessions: Session[] = [];
  let activeSession: Session | null = null;
  let notes = "";

  async function loadSessions() {
    sessions = await invoke<Session[]>("list_sessions", { projectId });
    activeSession = sessions.find((s) => s.ended_at === null) ?? null;
  }

  async function startSession() {
    activeSession = await invoke<Session>("start_session", { projectId });
    await loadSessions();
  }

  async function endSession() {
    if (!activeSession) return;

    await invoke<Session>("end_session", {
      sessionId: activeSession.id,
      notes
    });

    notes = "";
    activeSession = null;
    await loadSessions();
  }

  $: if (projectId) {
    loadSessions();
  }

  onMount(loadSessions);
</script>

<section>
  <h3>Sessions</h3>

  {#if activeSession}
    <p>Session started: {activeSession.started_at}</p>

    <textarea bind:value={notes} placeholder="Session notes"></textarea>
    <br />

    <button on:click={endSession}>End session</button>
  {:else}
    <button on:click={startSession}>Start session</button>
  {/if}

  <h4>Session history</h4>

  {#if sessions.length === 0}
    <p>No sessions yet.</p>
  {:else}
    <ul>
      {#each sessions as session}
        <li>
          <strong>{session.started_at}</strong>
          {#if session.ended_at}
            → {session.ended_at}
          {:else}
            → active
          {/if}

          {#if session.notes}
            <br />
            {session.notes}
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>