
<script>
// ----- STYLES -----
    import "../../global.css"

// ----- COMPONENTS -----
    import DoubleVideo from '$lib/components/double_video.svelte';
    import Map from '$lib/components/map.svelte';
    import NavigationPlan from '$lib/components/navigation_plan.svelte';
    import Costmap from '$lib/components/costmap.svelte';
    import { depthCamera, frontCamera } from "../../state.svelte";

    
</script>

<div class="grid">

    <div class="grid-item" style="padding-right: 0">
        <Map />
    </div>

    <div class="grid-nest" style="grid-template-columns: 2fr 3fr; padding-left: 0">

        <div class="grid-item" style="padding-left: 0">
        <NavigationPlan style="width: 100%; height: 100%;" />
        </div>

        <div class="grid-item">
        <div class="container">
            IMU data
        </div>
        </div>

    </div>

    <div class="grid-item" style="padding-right: 0">
        <DoubleVideo camera1={depthCamera} camera2={frontCamera} />
    </div>

    <div class="grid-item">
        <Costmap style="width: 100%; height: 100%;" />
    </div>

</div>