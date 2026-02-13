<script lang="ts">
// ----- STYLES -----
    import '../../global.css';

    export let port: string; // <-- prop from parent

// ----- STATES -----
    let videoUrl = "http://localhost:" + port;
</script>

<div class="frame">
    <h1 class="heading">Live Camera Stream</h1>
    <img class="video-img" src={videoUrl} alt="Live video stream at {videoUrl}" />
</div>
