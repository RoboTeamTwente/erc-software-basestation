<script lang="ts">
// ----- STYLES -----
    import '../../global.css';

// ----- STYLES -----
    let { camera } = $props();


</script>

<div class="frame">
    <h1 class="heading"> {camera.name} </h1>
    <img class="video-img" src={camera.port} alt="Live video stream at {camera.name}" />
</div>
