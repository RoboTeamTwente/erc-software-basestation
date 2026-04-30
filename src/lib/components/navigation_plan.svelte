<script lang="ts">
// ----- TAURI / EXTERNAL -----
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { dndzone } from "svelte-dnd-action";
    import { confirm } from '@tauri-apps/plugin-dialog';

// ----- STYLES -----
    import '../../global.css';
    import '../css/map.css'
    import { waypoints, startPoint, endPoint, pinnedCoords } from "../../stores/map";
    import type { Waypoint, PinnedCoord } from "../../stores/map";

    let { style } = $props();

// ── File management ───────────────────────────────────────────────────────────

    async function import_file() {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [{ name: "Map Files", extensions: ["json","geojson","txt","jpeg","obj","las","laz","e57"] }]
        });
        if (!selected || Array.isArray(selected)) return;
        await invoke("import_map_file", { directory: selected });
    }

// ── Waypoint management ───────────────────────────────────────────────────────

    function addWaypointFromPin(pin: PinnedCoord) {
        waypoints.update(w => [
            ...w,
            { id: crypto.randomUUID(), x: pin.x, y: pin.y }
        ]);
        // Remove the pin so it's clear it's been consumed
        pinnedCoords.update(pins => pins.filter(p => p.id !== pin.id));
    }

    function setStartFromPin(pin: PinnedCoord) {
        startPoint.set({ id: "start", x: pin.x, y: pin.y });
        pinnedCoords.update(pins => pins.filter(p => p.id !== pin.id));
    }

    function setEndFromPin(pin: PinnedCoord) {
        endPoint.set({ id: "end", x: pin.x, y: pin.y });
        pinnedCoords.update(pins => pins.filter(p => p.id !== pin.id));
    }

    async function removeWaypoint(index: number) {
        const confirmed = await confirm("Are you sure you want to clear the navigation plan?", "Confirm Clear");

        if (!confirmed) return;
        waypoints.update(w => w.filter((_, i) => i !== index));
    }

    async function clearAll() {
        const confirmed = await confirm("Are you sure you want to clear the navigation plan?", "Confirm Clear");

        if (!confirmed) return;

        waypoints.set([]);
        startPoint.set(null);
        endPoint.set(null);
    }

    function handleDnd(e: CustomEvent) {
        waypoints.set(e.detail.items);
    }

    function formatCoord(w: Waypoint | null): string {
        if (!w) return "Not set";
        return `${w.x.toFixed(1)} m, ${w.y.toFixed(1)} m`;
    }
</script>



<div class="attached-container">
    <div class="grid-nest" style="grid-template-rows: auto 1fr auto">

        <!-- Header -->
        <div class="grid-item">
            <h1 class="heading" style="padding-bottom: 0">Navigation plan</h1>
        </div>

        <!-- Waypoint list -->
        <div class="grid-item" style="flex-direction: column; overflow: hidden;">
            <div class="task-list">

                <!-- START -->
                <div class="task-card start">
                    <div class="task-info">
                        <span class="navplan-label">▶ Start</span>
                        <span class="navplan-coord">{formatCoord($startPoint)}</span>
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
                                <span class="navplan-label">⬡ Waypoint {i + 1}</span>
                                <span class="navplan-coord">{waypoint.x.toFixed(1)} m, {waypoint.y.toFixed(1)} m</span>
                                <button class="delete-button" onclick={() => removeWaypoint(i)}>
                                    <img class="right-icon" src="/smallDelete.svg" alt="Delete" />
                                </button>
                            </div>
                        </div>
                    {/each}
                </div>

                <!-- END -->
                <div class="task-card end">
                    <div class="task-info">
                        <span class="navplan-label">⏹ End</span>
                        <span class="navplan-coord">{formatCoord($endPoint)}</span>
                    </div>
                </div>

                <!-- PINNED COORDS (from map) -->
                {#if $pinnedCoords.length > 0}
                    <div class="pin-section-header">📍 Pinned from map</div>
                    {#each $pinnedCoords as pin, i}
                        <div class="task-card pin-card">
                            <div class="task-info">
                                <span class="navplan-coord">#{i+1} — {pin.x.toFixed(2)} m, {pin.y.toFixed(2)} m</span>
                            </div>
                            <div class="pin-actions-row">
                                <button class="button secondary small" onclick={() => setStartFromPin(pin)}>
                                    Set Start
                                </button>
                                <button class="button secondary small" onclick={() => addWaypointFromPin(pin)}>
                                    + Waypoint
                                </button>
                                <button class="button secondary small" onclick={() => setEndFromPin(pin)}>
                                    Set End
                                </button>
                            </div>
                        </div>
                    {/each}
                {/if}

            </div>
        </div>

        <!-- Footer buttons -->
        <div class="grid-item">
            <button class="button" onclick={import_file}>+ Add Map File</button>
            <button class="button">▶︎ Plan Route</button>
            {#if $waypoints.length > 0 || $startPoint || $endPoint}
                <button class="button" onclick={clearAll}>✕ Clear</button>
            {/if}
        </div>

    </div>
</div>