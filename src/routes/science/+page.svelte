<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { invoke } from '@tauri-apps/api/core';

// ----- SVELTE -----
    import { onMount } from "svelte";
    import { scienceLocations, hoveredScienceId } from '../../stores/map';

// ----- COMPONENTS -----
    import DoubleVideo from '$lib/components/double_video.svelte';
    import Map from '$lib/components/map.svelte';
    import SamplingLocations from '$lib/components/sampling_locations.svelte';
    import InterestLocations from '$lib/components/interest_locations.svelte';


// ----- STATES -----
    let pickupMode = $state(false);


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

<main class="grid" style="grid-template-rows: 1fr; grid-template-columns: 5fr 3fr;">
    <div class="grid-nest" style="grid-template-rows: 1fr 1fr; grid-template-columns: 4fr 1fr;">
        <div class="grid-item" style="padding-right: 0">
            <Map mode={'science'}/>
        </div>
        <div class="grid-item" style="padding-left: 0;">
        <div class="container" style="border-top-left-radius: 0; border-bottom-left-radius: 0;">
            <InterestLocations locations={scienceLocations} hoveredId={hoveredScienceId}/>
        </div>
        </div>
        <div class="grid-item" style="padding-right: 0">
            <DoubleVideo/>
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
        <SamplingLocations />
    </div>
</main>