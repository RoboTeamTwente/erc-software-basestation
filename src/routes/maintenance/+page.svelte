
<script lang="ts">
// ----- STYLES -----
    import "../../global.css"

// ----- COMPONENTS -----
    import Video from '$lib/components/video.svelte';
    import MaintenanceTasks from "$lib/components/maintenance_tasks.svelte";

// ----- IMPORTS -----
    import { armCamera, depthCamera } from "../../state.svelte";
</script>

<main class="grid" style="grid-template-rows: 1fr; grid-template-columns: 5fr 3fr;">

    <div class="grid-nest" style="grid template-rows: 1fr 1fr; grid-template-columns: 4fr 1fr;">
        <div class="grid-item" style="padding-right: 0">
        <Video camera={armCamera} pixelMode={false} measure={false}/>
        </div>
        <div class="grid-item" style="padding-left: 0">
        <div class="container" style="border-top-left-radius: 0; border-bottom-left-radius: 0;">
            <h1 class="heading"> Arm feedback </h1>
        </div>
        </div>
        <div class="grid-item" style="padding-right: 0">
        <Video camera={depthCamera} pixelMode={false} measure={false}/>
        </div>
        <div class="grid-item">
        <div class="container">
            <h1 class="heading"> Arm position </h1>
        </div>
        </div>
    </div>

    <div class="grid-item">
        <MaintenanceTasks />
    </div>
</main>