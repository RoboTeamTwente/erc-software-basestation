<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import '../../global.css';

    let { style } = $props();

    let fileBrowser = $state(false);
    let baseDir = $state("");
    let files: { name: string; is_dir: boolean }[] = $state([]);
    let currentDirectory = $state("");
    let selectedFile: string | null = $state(null);

  async function import_file() {
    const selected = await open({
        multiple: false,
        directory: false,
        filters: [
            {
                name: "Map Files",
                extensions: ["json", "geojson", "txt", "jpeg"] // adjust
            }
        ]
    });

    if (!selected || Array.isArray(selected)) return;

    // selected is FULL FILE PATH ---------------------------TODO:WHAT DOES THIS DO
    await invoke("import_map_file", { directory: selected });
  }

</script>

<div class="attached-container">
    <div class="grid-nest" style="grid-template-rows: 1fr 10fr 2fr padding: 0">
        <div class="grid-item" style="padding: 0;">
            <h1 class="heading">Navigation plan</h1>
        </div>

        <div class="grid-item">
            <div class="grid-nest" style="grid-template-rows: repeat(6, 1fr)">
                <div class="start">
                    Starting Point
                </div>
                <div class="waypoint">
                    Waypoint 1
                </div>
                <div class="waypoint">
                    Waypoint 2
                </div>
                <div class="waypoint">
                    Waypoint 3
                </div>
                <div class="waypoint">
                    Waypoint 4
                </div>
                <div class="start">
                    End Point
                </div>
            </div>
        </div>

        <div class="grid-item">
            <button class="button" onclick={import_file}>
                + Add Map File
            </button>
            <button class="button">
                Plan Route
            </button>
        </div>
    </div>
</div>



<style>
    .start {
        background-color: white;
        border-radius: 16px;
        display: flex;
        padding-bottom: 0.4rem;
        padding-top: 0.4rem;
        padding-left: 0.8rem;
        margin: 0.25rem;
        font-size: 1rem;
        font-weight: bold;
        height: fit-content;
        overflow: hidden;
    }

    .waypoint {
        background-color: white;
        border-radius: 16px;
        display: flex;
        padding-bottom: 0.2rem;
        padding-top: 0.2rem;
        padding-left: 0.8rem;
        margin: 0.25rem;
        font-size: 1rem;
        width: 80%;
        height: fit-content;
        overflow: hidden;
    }

    /* Overlay stays mostly the same */
    .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    }

    .path {
    font-size: 0.85rem;
    color: #444;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    }

</style>
