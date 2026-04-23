<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import '../../global.css';
    import { detectedObjectsState, handleDetectedObjects } from "$lib/state/detectedObjects.svelte";
    import { detectedObjectTypeToJSON } from "$lib/proto/components/basestation/detected_object";

    type Props = {
        camera: any;
        mode?: 'pick' | 'measure' | 'detect' | null;
        onmeasurement?: (result: number) => void;
    }
    let { camera, mode, onmeasurement }: Props = $props();

    let imgElement: HTMLImageElement;
    let canvasElement: HTMLCanvasElement;

    let lastClick: { x: number; y: number; cam: string } | null = null;
    let points: { x: number; y: number }[] = [];

    // ----- DRAW: unified draw call -----
    function draw() {
        if (!canvasElement) return;
        const ctx = canvasElement.getContext("2d");
        if (!ctx) return;

        ctx.clearRect(0, 0, canvasElement.width, canvasElement.height);

        if (mode === 'detect') {
            drawBoundingBoxes(ctx);
        } else {
            drawMeasurePoints(ctx);
        }
    }

    // ----- BOUNDING BOXES -----
    function drawBoundingBoxes(ctx: CanvasRenderingContext2D) {
        const rect      = imgElement.getBoundingClientRect();
        const imgRatio  = imgElement.naturalWidth / imgElement.naturalHeight;
        const rectRatio = rect.width / rect.height;

        let displayWidth: number, displayHeight: number;
        let offsetX = 0, offsetY = 0;

        if (rectRatio > imgRatio) {
            displayHeight = rect.height;
            displayWidth  = rect.height * imgRatio;
            offsetX       = (rect.width - displayWidth) / 2;
        } else {
            displayWidth  = rect.width;
            displayHeight = rect.width / imgRatio;
            offsetY       = (rect.height - displayHeight) / 2;
        }

        // Single pass — hover state handled inline
        for (const obj of detectedObjectsState.objects) {
            const { bbox, type, id, confidence } = obj.data;
            if (!bbox) continue;

            const isHovered = detectedObjectsState.hoveredId === id;
            const isComplete = obj.actions.complete;

            const x = offsetX + ((bbox.x ?? 0) / 1000) * displayWidth;
            const y = offsetY + ((bbox.y ?? 0) / 1000) * displayHeight;
            const w =           ((bbox.width  ?? 0) / 1000) * displayWidth;
            const h =           ((bbox.height ?? 0) / 1000) * displayHeight;

            const color = isHovered ? "#ffff00" : isComplete ? "#00ff88" : "#ff4444";

            ctx.strokeStyle = color;
            ctx.lineWidth   = isHovered ? 3 : 2;
            ctx.strokeRect(x, y, w, h);

            const label = `${detectedObjectTypeToJSON(type ?? 0).replace("OBJECT_", "")}  ${((confidence ?? 0) * 100).toFixed(0)}%`;
            ctx.font = "bold 12px monospace";
            const textW = ctx.measureText(label).width;
            ctx.fillStyle = isHovered ? "#ffff00cc" : isComplete ? "#00ff88cc" : "#ff4444cc";
            ctx.fillRect(x, y - 18, textW + 8, 18);
            ctx.fillStyle = "#ffffff";
            ctx.fillText(label, x + 4, y - 4);
        }
    }

    // ----- MEASURE POINTS (unchanged logic, extracted) -----
    function drawMeasurePoints(ctx: CanvasRenderingContext2D) {
        for (const p of points) {
            ctx.beginPath();
            ctx.arc(p.x, p.y, 5, 0, Math.PI * 2);
            ctx.fillStyle = "red";
            ctx.fill();
        }
        if (points.length === 2) {
            ctx.beginPath();
            ctx.moveTo(points[0].x, points[0].y);
            ctx.lineTo(points[1].x, points[1].y);
            ctx.lineWidth   = 2;
            ctx.strokeStyle = "red";
            ctx.stroke();
        }
    }

    // ----- REACTIVE: redraw whenever detected objects change in detect mode -----
    $effect(() => {
        if (mode === 'detect') {
            // touch the reactive list so $effect re-runs on updates
            detectedObjectsState.objects;
            draw();
        }
    });

    // ----- CANVAS RESIZE -----
    async function resizeCanvas() {
        if (!imgElement || !canvasElement) return;
        const rect         = imgElement.getBoundingClientRect();
        canvasElement.width  = rect.width;
        canvasElement.height = rect.height;
        draw();
    }

    // ----- CLICK HANDLERS (pick / measure and detect) -----
    async function handleClick(event: MouseEvent) {
        const rect      = imgElement.getBoundingClientRect();
        const clickX    = event.clientX - rect.left;
        const clickY    = event.clientY - rect.top;
        const imgRatio  = imgElement.naturalWidth / imgElement.naturalHeight;
        const rectRatio = rect.width / rect.height;

        let displayWidth: number, displayHeight: number;
        let offsetX = 0, offsetY = 0;

        if (rectRatio > imgRatio) {
            displayHeight = rect.height;
            displayWidth  = rect.height * imgRatio;
            offsetX       = (rect.width - displayWidth) / 2;
        } else {
            displayWidth  = rect.width;
            displayHeight = rect.width / imgRatio;
            offsetY       = (rect.height - displayHeight) / 2;
        }

        const x      = Math.max(0, Math.min(clickX - offsetX, displayWidth));
        const y      = Math.max(0, Math.min(clickY - offsetY, displayHeight));
        const nx     = x / displayWidth;
        const ny     = y / displayHeight;
        const canvasX = offsetX + x;
        const canvasY = offsetY + y;

        if (mode === 'pick') {
            await invoke("send_pixel", { camera: camera.name, x: nx, y: ny });
        } else if (mode === 'measure') {
            if (points.length === 2) points = [];
            points.push({ x: canvasX, y: canvasY });
            draw();

            if (lastClick !== null && lastClick.cam === camera.name) {
                const result = await invoke<number>("request_measurement", {
                    camera1: camera.name, x1: nx, y1: ny,
                    camera2: lastClick.cam, x2: lastClick.x, y2: lastClick.y,
                });
                onmeasurement?.(result);
                lastClick = null;
            } else {
                lastClick = { x: nx, y: ny, cam: camera.name };
            }
        }
    }

    async function handleDetectClick(event: MouseEvent) {
        const rect      = imgElement.getBoundingClientRect();
        const imgRatio  = imgElement.naturalWidth / imgElement.naturalHeight;
        const rectRatio = rect.width / rect.height;

        let displayWidth: number, displayHeight: number;
        let offsetX = 0, offsetY = 0;

        if (rectRatio > imgRatio) {
            displayHeight = rect.height;
            displayWidth  = rect.height * imgRatio;
            offsetX       = (rect.width - displayWidth) / 2;
        } else {
            displayWidth  = rect.width;
            displayHeight = rect.width / imgRatio;
            offsetY       = (rect.height - displayHeight) / 2;
        }

        const clickX = event.clientX - rect.left - offsetX;
        const clickY = event.clientY - rect.top  - offsetY;

        // convert click to 0-1000 space
        const nx = (clickX / displayWidth)  * 1000;
        const ny = (clickY / displayHeight) * 1000;

        for (const obj of detectedObjectsState.objects) {
            const { bbox, id } = obj.data;
            if (!bbox || id === undefined) continue;

            const x = bbox.x ?? 0;
            const y = bbox.y ?? 0;
            const w = bbox.width  ?? 0;
            const h = bbox.height ?? 0;

            if (nx >= x && nx <= x + w && ny >= y && ny <= y + h) {
                await invoke("select_object", { objectId: id });
                break; // only select the first hit
            }
        }
        
    }

    function handleDetectMouseMove(event: MouseEvent) {
        const rect      = imgElement.getBoundingClientRect();
        const imgRatio  = imgElement.naturalWidth / imgElement.naturalHeight;
        const rectRatio = rect.width / rect.height;

        let displayWidth: number, displayHeight: number;
        let offsetX = 0, offsetY = 0;

        if (rectRatio > imgRatio) {
            displayHeight = rect.height;
            displayWidth  = rect.height * imgRatio;
            offsetX       = (rect.width - displayWidth) / 2;
        } else {
            displayWidth  = rect.width;
            displayHeight = rect.width / imgRatio;
            offsetY       = (rect.height - displayHeight) / 2;
        }

        const clickX = event.clientX - rect.left - offsetX;
        const clickY = event.clientY - rect.top  - offsetY;
        const nx = (clickX / displayWidth)  * 1000;
        const ny = (clickY / displayHeight) * 1000;

        let hit: number | null = null;
        for (const obj of detectedObjectsState.objects) {
            const { bbox, id } = obj.data;
            if (!bbox || id === undefined) continue;
            const x = bbox.x ?? 0, y = bbox.y ?? 0;
            const w = bbox.width ?? 0, h = bbox.height ?? 0;
            if (nx >= x && nx <= x + w && ny >= y && ny <= y + h) {
                hit = id;
                break;
            }
        }

        detectedObjectsState.hoveredId = hit;
        draw(); // redraw to update highlight immediately
    }

    function handleDetectMouseLeave() {
        detectedObjectsState.hoveredId = null;
        draw();
    }

    onMount(() => {
        resizeCanvas();
        window.addEventListener("resize", resizeCanvas);
        return () => window.removeEventListener("resize", resizeCanvas);
    });
</script>

<div class="frame">
    <h1 class="heading">{camera.name}</h1>
    <img
        class="video-img"
        bind:this={imgElement}
        src={camera.port}
        alt="Live video stream at {camera.name}"
        onload={resizeCanvas}
    />
    {#if mode === 'pick' || mode === 'measure'}
        <canvas
            bind:this={canvasElement}
            onclick={handleClick}
            style="position:absolute; top:0; left:0; width:100%; height:100%; cursor:crosshair; z-index:101;"
        ></canvas>
    {:else if mode === 'detect'}
        <canvas
            bind:this={canvasElement}
            onclick={handleDetectClick}
            onmousemove={handleDetectMouseMove}
            onmouseleave={handleDetectMouseLeave}
            style="position:absolute; top:0; left:0; width:100%; height:100%; cursor:pointer; z-index:101;"
        ></canvas>
    {/if}
    {#if camera.stale}
        <div class="stale-overlay">
            <span class="stale-text">⚠ SIGNAL LOST</span>
        </div>
    {/if}
</div>