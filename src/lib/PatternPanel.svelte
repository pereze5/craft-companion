<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { openPath } from "@tauri-apps/plugin-opener";

  import {
    DATA_CHANGED_EVENT,
    notifyDataChanged
  } from "$lib/appEvents";

  export let projectId: number;

  type Pattern = {
    id: number;
    project_id: number;
    display_name: string;
    file_path: string;
    file_type: string;
  };

  let patterns: Pattern[] = [];
  let manualPatternPath = "";
  let error = "";

  async function loadPatterns() {
    patterns = await invoke<Pattern[]>("list_patterns", { projectId });
  }

  function isUrl(value: string) {
    return value.startsWith("http://") || value.startsWith("https://");
  }

  function inferFileType(path: string) {
    const lower = path.toLowerCase();

    if (isUrl(path)) {
      if (lower.includes("youtube.com") || lower.includes("youtu.be")) {
        return "youtube";
      }

      return "url";
    }

    if (lower.endsWith(".pdf")) return "pdf";
    if (lower.endsWith(".png")) return "image";
    if (lower.endsWith(".jpg")) return "image";
    if (lower.endsWith(".jpeg")) return "image";

    return "unknown";
  }

  function basename(path: string) {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function displayNameForUrl(url: string) {
    if (url.includes("youtube.com") || url.includes("youtu.be")) {
      return "YouTube tutorial";
    }

    return url;
  }

  async function choosePatternFile() {
    error = "";

    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Pattern files",
            extensions: ["pdf", "png", "jpg", "jpeg"]
          }
        ]
      });

      if (!selected || Array.isArray(selected)) return;

      const filePath = selected;
      const displayName = basename(filePath);
      const fileType = inferFileType(filePath);

      if (fileType === "unknown") {
        error = "Unsupported file type.";
        return;
      }

      await invoke<Pattern>("add_pattern", {
        projectId,
        displayName,
        filePath,
        fileType
      });

      notifyDataChanged();

      await loadPatterns();
    } catch (e) {
      console.error("Failed to add pattern:", e);
      error = String(e);
    }
  }

  async function addPatternFromUrl() {
    error = "";

    const url = manualPatternPath.trim();

    if (!url) {
      error = "URL is required.";
      return;
    }

    const fileType = inferFileType(url);

    if (fileType === "unknown") {
      error = "Unsupported pattern URL.";
      return;
    }

    try {
      await invoke<Pattern>("add_pattern", {
        projectId,
        displayName: displayNameForUrl(url),
        filePath: url,
        fileType
      });

      manualPatternPath = "";

      notifyDataChanged();

      await loadPatterns();
    } catch (e) {
      console.error("Failed to add pattern URL:", e);
      error = String(e);
    }
  }

  async function openPattern(filePath: string) {
    error = "";

    try {
      await openPath(filePath);
    } catch (e) {
      console.error("Failed to open pattern:", e);
      error = String(e);
    }
  }

  async function deletePattern(patternId: number) {
    error = "";

    try {
      await invoke("delete_pattern", { patternId });

      notifyDataChanged();

      await loadPatterns();
    } catch (e) {
      console.error("Failed to delete pattern:", e);
      error = String(e);
    }
  }

  async function importMobile() {
    error = "";

    try {
      const result =
        await invoke<string>("import_mobile_data");

      console.log(result);

      alert(result);

      notifyDataChanged();

      await loadPatterns();
    } catch (e) {
      console.error("Failed to import mobile data:", e);
      error = String(e);
    }
  }

  onMount(() => {
    loadPatterns();

    window.addEventListener(DATA_CHANGED_EVENT, loadPatterns);

    return () => {
      window.removeEventListener(DATA_CHANGED_EVENT, loadPatterns);
    };
  });
</script>

<section>
  <h3>Patterns</h3>

  <button on:click={choosePatternFile}>
    Add pattern file
  </button>

  <div>
    <input
      bind:value={manualPatternPath}
      placeholder="Paste YouTube or pattern URL"
    />

    <button on:click={addPatternFromUrl}>
      Add URL
    </button>
  </div>

  <button on:click={importMobile}>
    Import Mobile Data
  </button>

  {#if error}
    <p>{error}</p>
  {/if}

  {#if patterns.length === 0}
    <p>No pattern files linked yet.</p>
  {:else}
    <ul>
      {#each patterns as pattern}
        <li>
          <strong>{pattern.display_name}</strong>

          <br />

          Type: {pattern.file_type}

          <br />
          <br />

          <small>{pattern.file_path}</small>

          <br />

          <button
            on:click={() => openPattern(pattern.file_path)}
          >
            Open pattern
          </button>

          <button
            on:click={() => deletePattern(pattern.id)}
          >
            Remove pattern
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>