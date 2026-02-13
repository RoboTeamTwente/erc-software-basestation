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
    <div class="grid-nest" style="grid-template-rows: auto 1fr auto; padding: 0;">
        <div class="grid-item" style="padding: 0;">
            <h1 class="heading">Navigation plan</h1>
        </div>

        <div class="grid-item">
            <div class="route-list">
                <div class="route-item start">
                    Starting Point
                </div>

                <div class="route-item waypoint">
                    Waypoint 1
                </div>

                <div class="route-item waypoint">
                    Waypoint 2
                </div>

                <div class="route-item waypoint">
                    Waypoint 3
                </div>

                <div class="route-item waypoint">
                    Waypoint 4
                </div>

                <div class="route-item end">
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
            <button class="button">
                ▶︎ Go to Next Location
            </button>
        </div>
    </div>
</div>



<style>
    /* Route list container */
    .route-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;            /* clean even spacing */
        padding: 0.5rem 1rem;
        width: 100%;
        box-sizing: border-box;

        overflow-y: auto;   /* makes it scroll */
        min-height: 0;   
    }

    /* Base style for all route items */
    .route-item {
        background-color: white;
        border-radius: 16px;
        padding: 0.6rem 1rem;
        font-size: 1rem;
        display: flex;
        align-items: center;
        width: 100%;            /* ensures perfect alignment */
        box-sizing: border-box;
    }

    /* Start & End slightly stronger */
    .start,
    .end {
        font-weight: 600;
        border: 2px solid var(--color-rtpurple);
    }

    /* Waypoints slightly inset look */
    .waypoint {
        background-color: #fff;
        height: 2rem;
    }

</style>
