<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { depthCamera, frontCamera, armCamera } from '../../state.svelte';

// ----- STATES -----
let pickupMode = $state(false);
let swapped = $state(false);

let modeCam = $derived(pickupMode ? armCamera : frontCamera);

let activeCam = $derived(swapped ? modeCam : depthCamera);
let secondaryCam = $derived(swapped ? depthCamera : modeCam);

$effect(() => {
    const interval = setInterval(getPickupMode, 250);
    return () => clearInterval(interval);
});

async function getPickupMode() {
    pickupMode = await invoke("get_state", { stateType: "Pickup" });
}

function toggleVideo() {
    swapped = !swapped;
}
</script>

<div class="frame">
    <h1 class="heading">{activeCam.name}</h1>
    <img src={activeCam.port} class="video-img" alt="Video feed from port {activeCam.name}" />

    {#if activeCam.stale}
        <div class="stale-overlay">
            <span class="stale-text">⚠ SIGNAL LOST</span>
        </div>
    {/if}

    <button
        class="frame secondary"
        style="height: 30%; width: 25%; align-self: flex-end; margin: 10px; position: absolute; bottom: 0; cursor: pointer"
        onclick={() => toggleVideo()}
    >
        <h1 class="heading" style="z-index: 3;">{secondaryCam.name}</h1>
        <img src={secondaryCam.port} class="video-img" style="z-index: 2;" alt="Video feed from port {secondaryCam.name}" />

        {#if secondaryCam.stale}
            <div class="stale-overlay">
                <span class="stale-text small">⚠ SIGNAL LOST</span>
            </div>
        {/if}
    </button>
</div>