<script lang="ts">
  import "../../global.css"
  import Video from '$lib/components/video.svelte';
  import Map from '$lib/components/map.svelte';
</script>

<main class="grid" style="grid-template-rows: 1fr; grid-template-columns: 5fr 3fr;">

  <div class="grid-nest" style="grid template-rows: 1fr 1fr; grid-template-columns: 4fr 1fr;">
    <div class="grid-item" style="padding-right: 0">
      <Map style="width: 100%; height: 100%;"/>
    </div>
    <div class="grid-item" style="padding-left: 0;">
      <div class="container" style="border-top-left-radius: 0; border-bottom-left-radius: 0;">
        Locations of interest
      </div>
    </div>
    <div class="grid-item" style="padding-right: 0">
      <Video style="width: 100%; height: 100%;"/>
    </div>
    <div class="grid-item" style="padding-left: 0">
      <div class="container" style="border-top-left-radius: 0; border-bottom-left-radius: 0;">
        Side menu
      </div>
    </div>
  </div>

  <div class="grid-item">
    <div class="container">
      Sampling Locations
    </div>
  </div>
</main>