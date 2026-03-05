<script lang="ts">
// ----- TAURI / EXTERNAL -----
    import { invoke } from "@tauri-apps/api/core";

// ----- SVELTE -----
    import { onMount } from 'svelte';
    import { get } from "svelte/store";

// ----- STYLES -----
    import '../../global.css';
    import { displayedMap } from "../../stores/map";


// ----- STATES -----
    let mapFiles = $state<string[]>([]);
    let mapCount = $state(0);
    let selectedMap = $state<string | null>(null);
    let openedMap = $state<string | null>(null);

    let mouseX = $state(0);
    let mouseY = $state(0);


// ===============================
// MAP MANAGEMENT
// ===============================
    async function loadMap() {
        openedMap = get(displayedMap);
        if (openedMap == "" || openedMap == null){
            openedMap = null;
            listMaps();
        }
    }

    async function listMaps() {
        const result = await invoke<string[]>("list_task_files", {directory: "maps"});
        console.log("Result:", result);
        if (result.length == 1) {
            mapFiles = result;
            selectedMap = mapFiles[0];
            confirmMapSelection();
        } if (result.length > 1) {
            mapFiles = result;
        } else {
            mapFiles = ["No maps found"];
        }
        mapCount = mapFiles.length;
    }

    async function reload() {
        listMaps();
        openedMap = null;
        selectedMap = null;
    }

    async function selectMap(file: string) {
        selectedMap = file;
    }

    async function confirmMapSelection() {
        openedMap = selectedMap;
        displayedMap.set(openedMap);
    }


// ===============================
// COORDINATES
// ===============================
    function updateCoords(e: MouseEvent) {
        mouseX = e.clientX;
        mouseY = e.clientY;
    }

    
// ===============================
// LIFECYCLE
// ===============================
	onMount(() => {
		loadMap();
	});
</script>


<div class="frame"   onmousemove={updateCoords} aria-hidden="true">
    <!-- Header -->
    <div class="header">
        <!-- Reload button -->
        <button class="reload-button" onclick={reload} title="Reload maps">
            ⟳
        </button>
    </div>

    <!-- Content -->
    {#if openedMap === null}
        <div class="file-modal" style="width: 100%; height: 100%">
            {#if mapFiles.length === 0}
                <p class="muted" style="padding-left: 1rem;">No maps found, please add a map file.</p>

            {:else}
                <div class="file-header">Select a map:</div>
                <div class="file-list">
                    {#each mapFiles as file}
                        <div class="file-row">
                            <button class="file-item {selectedMap === file ? 'selected': ''}" onclick={() => selectMap(file)}>
                                <span class="map-icon">🗺️</span>
                                <span class="name">{file}</span>
                            </button>
                        </div>
                    {/each}
                </div>
                <div class="file-footer">
                    <button class="button secondary" onclick={confirmMapSelection}>
                        Confirm selection
                    </button>
                </div> 
            {/if}
        </div>
    {:else}
        Displaying map {openedMap}
            <!-- Coordinates -->
        <div class="coords">
            x: {mouseX} y: {mouseY}
        </div>
    {/if}

</div>