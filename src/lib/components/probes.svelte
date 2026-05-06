<script lang="ts">
    import type { Probe } from '../../types';
    import { probes } from '$lib/stores/probes';

    import Video from './video.svelte';
    import { armCamera, depthCamera, frontCamera } from "../../state.svelte.js";
    const cameras = [armCamera, frontCamera, depthCamera];

// ----- STATES -----
    let overlay = $state(false);

// ----- PICKUP -----
    async function pick() {
    }
</script>

<div class="container">
    <div class="grid-nest" style="grid-template-columns: 1fr; grid-template-rows: 1fr 4fr 1fr;">

        <div class="grid-item">
            <h1 class="heading"> Probes found </h1>
        </div>

        <div class="grid-item">
            <div class="task-list">
                {#each $probes as probe (probe.id)}
                    <div class="task-card">
                        Probe
                    </div>
                {/each}
            </div>
        </div>

        <div class="grid-item" style="flex-direction: column;">
            <button class="button" style="align-self: flex-end;" onclick={() => overlay = true}>
                Pick up probe
            </button>
        </div>

    </div>
</div>

<!-- Image capture: click any camera feed to take a snapshot -->
{#if overlay}
    <div class="modal-overlay">
        <div class="modal" >
            <button class="close-button" onclick={() => overlay = false}>&times;</button>
            <h1>Pick up probe</h1>
            <div class="video-row">
                {#each cameras as cam}
                    <div 
                        class="clickable-video" 
                        role="button"
                        tabindex="0"
                        onclick={() => pick}
                        onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") pick() }}
                    >
                        <Video camera={cam} mode="pick"/>
                    </div>
                {/each}
            </div>
        </div>
    </div>
{/if}