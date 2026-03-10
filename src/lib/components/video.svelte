<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { invoke } from "@tauri-apps/api/core";


// ----- SVELTE -----
    import { onMount } from "svelte";


// ----- STYLES -----
    import '../../global.css';
    let { camera, pixelMode, measure } = $props();

    let imgElement: HTMLImageElement;
    let canvasElement: HTMLCanvasElement;

    let lastClick: {x:number,y:number, cam: string}|null = null;


// ----- SELECT PIXEL -----
    function resizeCanvas() {
        if (!imgElement || !canvasElement) return;

        const rect = imgElement.getBoundingClientRect();
        canvasElement.width = rect.width;
        canvasElement.height = rect.height;
    }



    async function handleClick(event: MouseEvent) {

        const rect = imgElement.getBoundingClientRect();

        const clickX = event.clientX - rect.left;
        const clickY = event.clientY - rect.top;

        const imgRatio = imgElement.naturalWidth / imgElement.naturalHeight;
        const rectRatio = rect.width / rect.height;

        let displayWidth;
        let displayHeight;
        let offsetX = 0;
        let offsetY = 0;
        let coordX;
        let coordY;

        if (rectRatio > imgRatio) {
            // vertical bars (image height fits)
            displayHeight = rect.height;
            displayWidth = rect.height * imgRatio;
            offsetX = (rect.width - displayWidth) / 2;
        } else {
            // horizontal bars (image width fits)
            displayWidth = rect.width;
            displayHeight = rect.width / imgRatio;
            offsetY = (rect.height - displayHeight) / 2;
        }

        const x = clickX - offsetX;
        const y = clickY - offsetY;

        // change clicks outside the actual image
        if (x < 0){
            coordX = 0;
        } else if (x > displayWidth){
            coordX = displayWidth;
        } else {
            coordX = x;
        }

        if (y < 0){
            coordY = 0;
        } else if (y > displayHeight){
            coordY = displayHeight;
        } else {
            coordY = y;
        }

        const nx = coordX / displayWidth;
        const ny = coordY / displayHeight;

        console.log(`Normalized image coords: (${nx}, ${ny})`);

        if (measure) {
            await invoke("request_measurement", {
                camera1: camera.name,
                x1: nx,
                y1: ny,
                camera2: lastClick ? lastClick.cam : null,
                x2: lastClick ? lastClick.x : null,
                y2: lastClick ? lastClick.y : null,
            });

            lastClick = {x: nx, y: ny, cam: camera.name};
        } else {
            await invoke("send_pixel", {
                camera: camera.name,
                x: nx,
                y: ny,
            });
        }
    }


// ===============================
// LIFECYCLE
// ===============================
    onMount(()=>{
        resizeCanvas();
        window.addEventListener("resize",resizeCanvas);
    });
</script>

<div class="frame">
    <h1 class="heading"> {camera.name} </h1>
    <img class="video-img" bind:this={imgElement} src={camera.port} alt="Live video stream at {camera.name}" onload={resizeCanvas}/>
    {#if pixelMode}
        <canvas bind:this={canvasElement} onclick={handleClick} style="position:absolute; top:0; left:0; width:100%; height:100%; cursor:crosshair; border: 2px solid #; z-index: 101;"></canvas>
    {/if}
</div>
