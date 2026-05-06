
<script lang="ts">
// ----- STYLES -----
    import "$lib/css/global.css"

// ----- COMPONENTS -----
    import Video from '$lib/components/video.svelte';
    import MaintenanceTasks from "$lib/components/maintenance_tasks.svelte";
    import ArmViz from "$lib/components/arm_viz.svelte";
    import ArmFeedback from "$lib/components/arm_feedback.svelte";

// ----- IMPORTS -----
    import { armCamera, depthCamera } from "../../state.svelte";
</script>

<main class="grid" style="grid-template-rows: 1fr; grid-template-columns: 2fr 1fr;">

    <div class="grid-nest" style="grid template-rows: 1fr 1fr; grid-template-columns: 3fr 1fr;">
        <div class="grid-item" style="padding-right: 0">
            <Video camera={armCamera}/>
        </div>
        <div class="grid-item" style="padding-left: 0">
            <ArmFeedback />
        </div>
        <div class="grid-item" style="padding-right: 0">
            <Video camera={depthCamera} mode="detect"/>
        </div>
        <div class="grid-item">
            <ArmViz />
        </div>
    </div>

    <div class="grid-item">
        <MaintenanceTasks />
    </div>
</main>