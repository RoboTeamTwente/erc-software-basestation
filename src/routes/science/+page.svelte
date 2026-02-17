<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { invoke } from '@tauri-apps/api/core';

// ----- SVELTE -----
    import { onMount } from "svelte";

// ----- COMPONENTS -----
    import DoubleVideo from '$lib/components/double_video.svelte';
    import Map from '$lib/components/map.svelte';

// ----- IMPORTS ------
    import { depthCamera, frontCamera, armCamera } from '../../state.svelte';


// ----- STATES -----
    let pickupMode = $state(false);
    let cam1 = $state(depthCamera);
    let cam2 = frontCamera;


// ----- ROVER MODES LOGIC -----
    async function togglePickup() {
        pickupMode = !pickupMode;
        setCameras();
        await invoke("pickup_mode_to_backend", {pickupMode});
    }
    async function getPickupMode() {
        pickupMode = await invoke("pickup_mode_from_backend");
    }
    function setCameras(){
        if (pickupMode){
            cam1 = armCamera;
        } else {
            cam1 = depthCamera;
        }
    }


// ===============================
// LIFECYCLE
// ===============================
    onMount(async () => {
        await getPickupMode();
        setCameras();
    });

</script>

<main class="grid" style="grid-template-rows: 1fr; grid-template-columns: 5fr 3fr;">
    <div class="grid-nest" style="grid-template-rows: 1fr 1fr; grid-template-columns: 4fr 1fr;">
        <div class="grid-item" style="padding-right: 0">
        <Map/>
        </div>
        <div class="grid-item" style="padding-left: 0;">
        <div class="container" style="border-top-left-radius: 0; border-bottom-left-radius: 0;">
            Locations of interest
        </div>
        </div>
        <div class="grid-item" style="padding-right: 0">
        <DoubleVideo camera1={cam1} camera2={cam2}/>
        </div>
        <div class="grid-item" style="padding-left: 0">
        <div class="container" style="border-top-left-radius: 0; border-bottom-left-radius: 0;">
            <button class="button" style="margin: 10px" onclick={togglePickup}>
                {#if pickupMode}
                    Go to Drive Mode
                {:else}
                    Go to Pick-up Mode
                {/if}
            </button>
        </div>
        </div>
    </div>

    <div class="grid-item">
        <div class="container">
        Sampling Locations
        </div>
    </div>
</main>