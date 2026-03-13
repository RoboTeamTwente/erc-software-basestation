<script lang="ts">
// ----- STYLES -----
    let { camera1, camera2 } = $props();


// ----- STATES -----
    let activeCam = $state(camera1);
    let secondaryCam = $state(camera2);


    // React when parent props change
    $effect(() => { activeCam = camera1; secondaryCam = camera2; });

    $effect(() => {secondaryCam = camera2;});


// ------ CAMERA LOGIC -----
    async function toggleVideo(){
        let temp = activeCam;
        activeCam = secondaryCam
        secondaryCam = temp;
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