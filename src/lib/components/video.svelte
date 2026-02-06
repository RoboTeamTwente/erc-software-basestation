<script lang="ts">
    import '../../global.css';

    let port = $props();

    let videoUrl = "http://localhost:5000";
</script>

<div class="frame">
    <h1 class="heading">Live Camera Stream</h1>
    <img class="video-img" src={videoUrl} alt="Live video stream" />
</div>
