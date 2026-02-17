<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { invoke } from '@tauri-apps/api/core';

// ----- SVELTE -----
    import { onMount } from "svelte";

// ----- IMPORTS ------
    import { depthCamera, frontCamera, armCamera } from '../state.svelte';

// ----- COMPONENTS -----
    import Double_Video from '$lib/components/double_video.svelte';
    import Map from '$lib/components/map.svelte';
    import NavigationPlan from '$lib/components/navigation_plan.svelte';
    import TaskCompletion from '$lib/components/task_completion.svelte';


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

<div class="grid">

    <div class="grid-item" style="padding-right: 0">
        <Double_Video camera1={cam1} camera2={cam2} />
    </div>


    <div class="grid-nest" style="grid-template-columns: 1fr 2fr; grid-template-rows: 1fr 2fr;">

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
        <div class="container">
            <h1 class="heading">Short Instructions for controls</h1>
        </div>
        </div>

        <div class="grid-item">
        <div class="container">
            <h1 class="heading">Model</h1>
        </div>
        </div>

        <div class="grid-item">
        <TaskCompletion />
        </div>

    </div>


    <div class="grid-item" style="padding-right: 0">
        <Map />
    </div>


    <div class="grid-nest" style="grid-template-columns: 2fr 3fr;">

        <div class="grid-item" style="padding-left: 0">
        <NavigationPlan style="width: 100%; height: 100%;" />
        </div>

        <div class="grid-item">
        <div class="container">
            <h1 class="heading">IMU and Sensor Data</h1> 
        </div>
        </div>

    </div>

</div>