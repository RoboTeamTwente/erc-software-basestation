<script lang="ts">
  import '../../global.css';
  import { invoke } from "@tauri-apps/api/core";
  export let style: string = "";

  let imgUrl: string | null = null;
  async function startPolling() {
      while (true) {
          const bytes: number[] = await invoke("fetch_snapshot", { port: 4500 });
          const blob = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });

          if (imgUrl) URL.revokeObjectURL(imgUrl);
          imgUrl = URL.createObjectURL(blob);

          await new Promise(r => setTimeout(r, 33));
      }
  }
  startPolling();
</script>

<div class="frame">
    <h1 class="heading">VIDEO</h1>
    <img
    src={imgUrl}
    class="video-img"
    alt="Video feed"
    />
</div>
