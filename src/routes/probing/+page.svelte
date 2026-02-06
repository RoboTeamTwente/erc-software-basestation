<script lang="ts">
  import "../../global.css"
  import Map from '$lib/components/map.svelte';
  import Video from '$lib/components/video.svelte';
</script>

<main class="grid">

  <div class="grid-item" style="padding-right: 0">
    <Map style="width: 100%; height: 100%;"/>
  </div>

  <div class="grid-nest" style="grid-template-columns: 1fr 2fr">
    <div class="grid-item">
      <div class="container">
        Locations of interest
      </div>
    </div>
    <div class="grid-item">
      <div class="container">
        Probes
      </div>
    </div>
  </div>

  <div class="grid-item" style="padding-right: 0">
    <Video port={5000}/>
  </div>
</main>