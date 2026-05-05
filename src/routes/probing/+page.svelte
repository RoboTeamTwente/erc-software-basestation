<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { invoke } from '@tauri-apps/api/core';

// ----- SVELTE -----
    import { onMount } from "svelte";
    import {probingLocations, hoveredProbingId} from '../../stores/map';

// ----- COMPONENTS -----
    import Map from '$lib/components/map.svelte';
    import DoubleVideo from '$lib/components/double_video.svelte';
    import Imu from '$lib/components/imu.svelte';
    import InterestLocations from '$lib/components/interest_locations.svelte';
    import Probes from '$lib/components/probes.svelte';

// ----- IMPORTS ------
    import { depthCamera, frontCamera, armCamera } from '../../state.svelte';


// ----- STATES -----
    let pickupMode = $state(false);
    let cam1 = $state(depthCamera);
    let cam2 = frontCamera;


// ----- ROVER MODES LOGIC -----
    async function togglePickup() {
        pickupMode = !pickupMode;
        await invoke("set_state", {stateType: "Pickup", value: pickupMode});
    }
    async function getPickupMode() {
        pickupMode = await invoke("get_state", {stateType: "Pickup"});
    }


// ===============================
// LIFECYCLE
// ===============================
    onMount(async () => {
        await getPickupMode();
    });

</script>

<main class="grid">
    <div class="grid-item" style="padding-right: 0">
        <Map mode={'probing'}/>
    </div>

    <div class="grid-nest" style="grid-template-columns: 1fr 2fr">
        <div class="grid-item">
            <InterestLocations locations={probingLocations} hoveredId={hoveredProbingId}/>
        </div>
        <div class="grid-item">
            <Probes />
        </div>
    </div>

    <div class="grid-item" style="padding-right: 0">
        <DoubleVideo/>
    </div>

    <div class="grid-nest" style="grid-template-columns: 1fr 2fr; padding-left: 0">

        <div class="grid-item" style="padding-left: 0">
            <div class="attached-container">
                <button class="button" style="margin: 10px" onclick={togglePickup}>
                    {#if pickupMode}
                        Go to Drive Mode
                    {:else}
                        Go to Pick-up Mode
                    {/if}
                </button>
            </div>
        </div>

        <div class="grid-item">
            <Imu />
        </div>

    </div>
</main>