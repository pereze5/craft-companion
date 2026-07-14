<script lang="ts">
import { onMount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { DATA_CHANGED_EVENT } from "$lib/appEvents";
  

  export let projectId: number;

  type Counter = {
    id: number;
    label: string;
    value: number;
  };

  type Session = {
    id: number;
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
  metadata_json: string;
  };

  type PatternMetadata = {
    designer?: string;
    needle_size?: string;
    gauge?: string;
    yarn_requirement?: string;
    fabric_count?: string;
    size?: string;
  };

  type ProjectInventoryItem = {
    id: number;
    item_name: string;
    quantity_allocated: number;
    unit: string;
  };

  let counters: Counter[] = [];
  let sessions: Session[] = [];
  let patterns: Pattern[] = [];
  let supplies: ProjectInventoryItem[] = [];

  async function loadSummary() {
    counters = await invoke<Counter[]>("list_counters", { projectId });

    sessions = await invoke<Session[]>("list_sessions", {
      projectId
    });

    patterns = await invoke<Pattern[]>("list_patterns", {
      projectId
    });

    supplies = await invoke<ProjectInventoryItem[]>(
      "list_project_inventory",
      {
        projectId
      }
    );
  }

  function parseMetadata(pattern: Pattern): PatternMetadata {
    try {
      return JSON.parse(pattern.metadata_json || "{}");
    } catch {
      return {};
    }
  }

  function parseCounterSnapshot(session: Session): Counter[] {
    try {
      return JSON.parse(session.counter_snapshot_json || "[]");
    } catch {
      return [];
    }
  }

  $: latestEndedSession =
    sessions.find((s) => s.ended_at !== null) ?? null;

  $: latestPattern =
    patterns.find((p) => p.id === latestEndedSession?.pattern_id) ??
    patterns[0] ??
    null;

  $: if (projectId) {
    loadSummary();
  }

  onMount(() => {
  loadSummary();

  window.addEventListener(
    DATA_CHANGED_EVENT,
    loadSummary
  );

  return () => {
    window.removeEventListener(
      DATA_CHANGED_EVENT,
      loadSummary
    );
  };
});
</script>

<section>
  <h3>Where I stopped</h3>

  {#if latestEndedSession}
    <div class="stopping-card">
      {#if latestPattern}
        <div class="pattern-name">
          {latestPattern.display_name}
        </div>
      {/if}

      <div class="page">
        Page {latestEndedSession.current_page}
      </div>

      {#if latestEndedSession.position_note}
        <div class="line">
          {latestEndedSession.position_note}
        </div>
      {/if}

      {#if latestEndedSession.notes}
        <div class="session-notes">
          {latestEndedSession.notes}
        </div>
      {/if}

      <div class="session-time">
        {latestEndedSession.started_at}
        {#if latestEndedSession.ended_at}
          → {latestEndedSession.ended_at}
        {/if}
      </div>
    </div>

    {#if latestPattern}
      {@const metadata = parseMetadata(latestPattern)}

      <div class="metadata">
        {#if metadata.needle_size}
          <div>Needle: {metadata.needle_size}</div>
        {/if}

        {#if metadata.gauge}
          <div>Gauge: {metadata.gauge}</div>
        {/if}

        {#if metadata.yarn_requirement}
          <div>Yarn: {metadata.yarn_requirement}</div>
        {/if}

        {#if metadata.fabric_count}
          <div>Fabric count: {metadata.fabric_count}</div>
        {/if}

        {#if metadata.size}
          <div>Size: {metadata.size}</div>
        {/if}
      </div>
    {/if}

    <h4>Counter snapshot</h4>

    {@const snapshotCounters = parseCounterSnapshot(latestEndedSession)}

    {#if snapshotCounters.length === 0}
      <p>No counter snapshot.</p>
    {:else}
      <ul>
        {#each snapshotCounters as counter}
          <li>
            {counter.label}: {counter.value}
          </li>
        {/each}
      </ul>
    {/if}
  {:else}
    <p>No completed sessions yet.</p>
    {/if}


  <h4>Live counters</h4>

  {#if counters.length === 0}
    <p>No counters yet.</p>
  {:else}
    <ul>
      {#each counters as counter}
        <li>
          {counter.label}: {counter.value}
        </li>
      {/each}
    </ul>
  {/if}

  <h4>Project supplies</h4>

  {#if supplies.length === 0}
    <p>No supplies linked.</p>
  {:else}
    <ul>
      {#each supplies as supply}
        <li>
          {supply.item_name}: {supply.quantity_allocated}
          {supply.unit}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .stopping-card {
    border: 2px solid currentColor;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .pattern-name {
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .page {
    font-size: 1.5rem;
    font-weight: 800;
    margin-bottom: 0.5rem;
  }

  .line {
    font-size: 1.2rem;
    font-weight: 700;
    line-height: 1.4;
    margin-bottom: 0.75rem;
  }

  .session-notes {
    margin-bottom: 0.75rem;
    opacity: 0.85;
  }

  .session-time {
    font-size: 0.9rem;
    opacity: 0.7;
  }

  .metadata {
    font-size: 0.9rem;
    opacity: 0.75;
    margin-bottom: 1rem;
  }
</style>