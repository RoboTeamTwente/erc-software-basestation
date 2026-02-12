<script lang="ts">
    import '../../global.css';
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from 'svelte';

    let mapFiles = $state<string[]>([]);
    let mapCount = $state(0);
    let selectedMap = $state<string | null>(null);
    let openedMap = $state<string | null>(null);

    let mouseX = $state(0);
    let mouseY = $state(0);

	onMount(() => {
		loadMap();
	});

    async function loadMap() {
        openedMap = await invoke("selected_map_from_backend");
        if (openedMap == ""){
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
        await invoke("selected_map_to_backend", {fileName: openedMap});
    }

    function updateCoords(e: MouseEvent) {
        mouseX = e.clientX;
        mouseY = e.clientY;
    }
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
                                <span class="icon">🗺️</span>
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


<style>
    .header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .coords {
        position: absolute;
        bottom: 8px;
        right: 8px;
        font-size: 11px;
        color: #888;
    }

    .muted {
        color: #888;
    }

</style>