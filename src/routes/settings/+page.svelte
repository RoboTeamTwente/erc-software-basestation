<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  
  async function ping() {
    await invoke("ping");
  }

  let taskFiles: string[] = [];
  let selectedFile: string | null = null;
  let fileContents = "";

  async function listFiles(directory: string) {
    const result = await invoke<string[]>("list_task_files", {directory});
    if (result.length > 0) {
      taskFiles = result;
    } else {
      taskFiles = ["Nothing found"];
    }
  }

  async function clearAllFiles(directory: string) {
    if (confirm("Are you sure you want to delete all task files?")) {
      await invoke("delete_all_task_files", {
        directory});
        taskFiles = [];
        selectedFile = null;
        fileContents = "";
    }
  }

  async function openTaskFile(file: string) {
    try {
      const data = await invoke<number[]>("read_task_file", {
        fileName: file
      });

      const uint8Array = new Uint8Array(data);
      const decoder = new TextDecoder("utf-8");

      fileContents = decoder.decode(uint8Array);
      selectedFile = file;
    } catch (e) {
      console.error("Failed to read file:", e);
    }
  }

  async function saveSnapshot() {
    await invoke("save_snapshot", {port:"5000"});
  }

</script>

<div>
  <button class="button" style="margin: 10px;" onclick={ping}>
    Ping Rust
  </button>

  <button class="button" style="margin: 10px;" onclick={() => listFiles("tasks")}>
    List task files
  </button>

  <button class="button" style="margin: 10px;" onclick={() => listFiles("images")}>
    List image files
  </button>

  <button class="button" style="margin: 10px;" onclick={() => listFiles("maps")}>
    List map files
  </button>

  <button class="button" style="margin: 10px;" onclick={() => saveSnapshot()}>
    Save an image file
  </button>

  <button class="button" style="margin: 10px;" onclick={() => clearAllFiles("tasks")}>
    Delete all task files
  </button>

  <button class="button" style="margin: 10px;" onclick={() => clearAllFiles("images")}>
    Delete all image files
  </button>

  <button class="button" style="margin: 10px;" onclick={() => clearAllFiles("maps")}>
    Delete all map files
  </button>

  <div>
    {#if taskFiles.length > 0}
      <div style="margin: 10px;">
        <h3>Task Files:</h3>
          <ul>
            {#each taskFiles as file}
              <li>
                <button
                  class="link-button"
                  onclick={() => openTaskFile(file)}
                >
                  {file}
                </button>
              </li>
            {/each}
          </ul>
      </div>
    {/if}
  </div>

  {#if selectedFile}
    <div style="margin: 10px;">
      <h3>Contents of {selectedFile}</h3>
      <pre>{fileContents}</pre>
    </div>
  {/if}

</div>