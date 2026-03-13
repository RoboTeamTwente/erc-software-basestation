<script lang="ts">
// ----- TAURI / EXTERNAL -----
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { dndzone } from "svelte-dnd-action";

// ----- STYLES -----
    import '../../global.css';
    import { waypoints, startPoint, endPoint } from "../../stores/map";

    let { style } = $props();

// ----- STATES -----


// ----- FILE MANAGEMENT -----
    async function import_file() {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [
                {
                    name: "Map Files",
                    extensions: ["json", "geojson", "txt", "jpeg"]
                }
            ]
        });

        if (!selected || Array.isArray(selected)) return;

        await invoke("import_map_file", { directory: selected });
    }

// ----- WAYPOINT MANAGEMENT -----
    function setStart(lat: number, lng: number) {
        startPoint.set({ id: "start", lat, lng });
        endPoint.set({ id: "end", lat, lng });
    }

    function addWaypoint() {
        waypoints.update(w => [
            ...w,
            { id: crypto.randomUUID(), lat: 0, lng: 0 }
        ]);
    }

    function removeWaypoint(index: number) {
        waypoints.update(w => w.filter((_, i) => i !== index));
    }

    // called during drag
    function handleDnd(e: CustomEvent) {
        waypoints.set(e.detail.items);
    }
</script>

<div class="attached-container">
    <div class="grid-nest" style="grid-template-rows: auto 1fr auto">

        <div class="grid-item">
            <h1 class="heading" style="padding-bottom: 0">Navigation plan</h1>
        </div>

        <div class="grid-item">
            <div class="task-list">

            <!-- START -->
            <div class="task-card start">
                <div class="task-info">
                    <span>Starting Point</span>
                </div>
            </div>

            <!-- WAYPOINTS -->
             <div                 
                use:dndzone={{ items: $waypoints, flipDurationMs: 200 }}
                onconsider={handleDnd}
                onfinalize={handleDnd}
            >
                {#each $waypoints as waypoint, i (waypoint.id)}
                <div class="task-card waypoint">
                    <div class="task-info">

                        <span>Waypoint {i + 1}</span>

                        <button
                            class="delete-button"
                            onclick={() => removeWaypoint(i)}
                        >
                            <img class="right-icon" src="/smallDelete.svg" alt="Delete" />
                        </button>

                    </div>
                </div>
                {/each}
            </div>

            <!-- END -->
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
                ▶︎ Plan Route
            </button>

            <button class="button" onclick={addWaypoint}>
                + Add Waypoint
            </button>
        </div>

    </div>
</div>