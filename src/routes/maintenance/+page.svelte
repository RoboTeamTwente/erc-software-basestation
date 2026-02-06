<script lang="ts">
  import "../../global.css"
  import Video from '$lib/components/video.svelte';
</script>

<main class="grid" style="grid-template-rows: 1fr; grid-template-columns: 5fr 3fr;">

  <div class="grid-nest" style="grid template-rows: 1fr 1fr; grid-template-columns: 4fr 1fr;">
    <div class="grid-item" style="padding-right: 0">
      <Video port={5000}/>
    </div>
    <div class="grid-item" style="padding-left: 0">
      <div class="container" style="border-top-left-radius: 0; border-bottom-left-radius: 0;">
        Arm feedback
      </div>
    </div>
    <div class="grid-item" style="padding-right: 0">
      <Video port={5000}/>
    </div>
    <div class="grid-item">
      <div class="container">
        Arm position???
      </div>
    </div>
  </div>

  <div class="grid-item">
    <div class="container">
      TODO list
    </div>
  </div>
</main>