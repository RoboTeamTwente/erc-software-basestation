<script lang="ts">
// ----- TAURI / EXTERNAL -----
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";

// ----- STYLES -----
    import '../../global.css';

    let { style } = $props();


// ----- FILE MANAGEMENT -----
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

        // selected is FULL FILE PATH
        await invoke("import_map_file", { directory: selected });
    }

</script>

<div class="attached-container">
    <div class="grid-nest" style="grid-template-rows: auto 1fr auto">
        <div class="grid-item" >
            <h1 class="heading" style="padding-bottom: 0">Navigation plan</h1>
        </div>

        <div class="grid-item">
            <div class="task-list">
                <div class="task-card start">
                    <div class="task-info">
                        <span>Starting Point </span>
                    </div>
                </div>

                <div class="task-card waypoint">
                    <div class="task-info">
                        <span>Waypoint 1</span>
                    </div>
                </div>

                <div class="task-card waypoint">
                    <div class="task-info">
                        <span>Waypoint 2</span>
                    </div>
                </div>

                <div class="task-card waypoint">
                    <div class="task-info">
                        <span>Waypoint 3</span>
                    </div>
                </div>

                <div class="task-card waypoint">
                    <div class="task-info">
                        <span>Waypoint 4</span>
                    </div>
                </div>

                <div class="task-card end">
                    <div class="task-info">
                        <span>End point</span>
                    </div>
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
            <button class="button">
                ▶︎ Go to Next Location
            </button>
        </div>
    </div>
</div>