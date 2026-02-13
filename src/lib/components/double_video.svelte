<script lang="ts">
// ----- STYLES -----
    import '../../global.css';

    let { port1, port2 } = $props();


// ----- STATES -----
    let videoUrl1 = "http://localhost:" + port1;
    let videoUrl2 = "http://localhost:" + port2;

    let toggle = $state(true)


// ----- UTILITIES -----
    async function toggleVideo(){
        toggle = !toggle;
    }

</script>

<div class="frame">
    {#if toggle}
        <h1 class="heading">POLE VIDEO</h1>
        <img
        src={videoUrl1}
        class="video-img"
        alt="Video feed from port {port1}"
        />
    {/if}
    {#if !toggle}
        <h1 class="heading"> FRONT VIDEO </h1>
        <img
        src={videoUrl2}
        class="video-img"
        alt="Video feed from port {port2}"
        />
    {/if}
    <button class="frame" style="height: 30%; width: 25%; align-self: flex-end; margin: 10px; position: absolute; bottom: 0; cursor: pointer" onclick={() => toggleVideo()}>
        {#if toggle}
            <h1 class="heading"> POLE VIDEO </h1>
            <img
            src={videoUrl2}
            class="video-img"
            alt="Video feed from port {port2}"
            />
        {/if}
        {#if !toggle}
            <h1 class="heading">FRONT VIDEO</h1>
            <img
            src={videoUrl1}
            class="video-img"
            alt="Video feed from port {port1}"
            />
        {/if}
    </button>
</div>