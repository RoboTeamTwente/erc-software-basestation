<script lang="ts">
// ----- TAURI / EXTERNAL -----
    import { invoke } from "@tauri-apps/api/core";
    import { convertFileSrc } from '@tauri-apps/api/core';
    import { appDataDir } from '@tauri-apps/api/path';

// ----- SVELTE -----
    import { onMount } from 'svelte';
    import { get } from "svelte/store";

// ----- STYLES -----
    import '../../global.css';
    import { displayedMap, pinnedCoords } from "../../stores/map";
    import type { PinnedCoord } from "../../stores/map";


// ── Types ────────────────────────────────────────────────────────────────────

    interface MapMeta {
        img_width:        number;
        img_height:       number;
        world_x_min:      number;
        world_y_min:      number;
        metres_per_pixel: number;
        format:           string;
    }


// ── State ────────────────────────────────────────────────────────────────────

    let mapFiles    = $state<string[]>([]);
    let selectedMap = $state<string | null>(null);
    let openedMap   = $state<string | null>(null);

    let mapPath     = $state<string>("");
    let mapMeta     = $state<MapMeta | null>(null);
    let isRendering = $state(false);
    let renderError = $state<string | null>(null);

    let imgEl           = $state<HTMLImageElement | null>(null);
    let mousePixel      = $state<{ x: number; y: number } | null>(null);
    let mouseWorld      = $state<{ x: number; y: number } | null>(null);

    // Whether the image should be rotated 90° to put its longest side horizontal
    let rotated = $state(false);

    const NEEDS_RENDER = ["obj", "las", "laz", "e57"];


// ── Helpers ──────────────────────────────────────────────────────────────────

    function fileExt(name: string): string {
        return name.split('.').pop()?.toLowerCase() ?? "";
    }

    function needs3DRender(name: string): boolean {
        return NEEDS_RENDER.includes(fileExt(name));
    }

    /** Pixel coords scaled to the PNG's actual resolution. */
    function eventToImgPixel(e: MouseEvent): { px: number; py: number } | null {
        if (!imgEl || !mapMeta) return null;
        const rect = imgEl.getBoundingClientRect();

        let px: number, py: number;

        if (rotated) {
            // Image is rotated 90° CW via CSS transform.
            // The displayed rect is the post-transform bounding box.
            // Map the mouse back into the original (unrotated) pixel space.
            const relX = e.clientX - rect.left;
            const relY = e.clientY - rect.top;
            // In the rotated frame:  displayed-x maps to original-y (bottom→top),
            //                        displayed-y maps to original-x (left→right).
            const scaleX = mapMeta.img_width  / rect.height; // note: swapped
            const scaleY = mapMeta.img_height / rect.width;
            px = relY * scaleX;
            py = (rect.width - relX) * scaleY;
        } else {
            const scaleX = mapMeta.img_width  / rect.width;
            const scaleY = mapMeta.img_height / rect.height;
            px = (e.clientX - rect.left) * scaleX;
            py = (e.clientY - rect.top)  * scaleY;
        }

        return { px, py };
    }


// ── Map management ───────────────────────────────────────────────────────────

    async function loadMap() {
        const stored = get(displayedMap);
        if (stored) {
            openedMap = stored;
            await openMap(stored);
        } else {
            await listMaps();
        }
    }

    async function listMaps() {
        const result = await invoke<string[]>("list_task_files", { directory: "maps" });
        mapFiles = result;
        if (result.length === 1) {
            selectedMap = result[0];
            await confirmMapSelection();
        }
    }

    async function reload() {
        openedMap    = null;
        selectedMap  = null;
        mapPath      = "";
        mapMeta      = null;
        renderError  = null;
        rotated      = false;
        pinnedCoords.set([]);
        displayedMap.set(null);
        await listMaps();
    }

    async function confirmMapSelection() {
        if (!selectedMap) return;
        openedMap = selectedMap;
        displayedMap.set(openedMap);
        await openMap(openedMap);
    }

    async function openMap(filename: string) {
        renderError = null;
        mapMeta     = null;
        mapPath     = "";
        rotated     = false;

        const base       = await appDataDir();
        const normalized = base.endsWith('/') ? base : base + '/';

        if (needs3DRender(filename)) {
            isRendering = true;
            try {
                const meta = await invoke<MapMeta>("render_map", { filename });
                mapMeta = meta;
                // Rotate if the image is taller than it is wide
                rotated = meta.img_height > meta.img_width;

                const stem    = filename.replace(/\.[^.]+$/, "");
                const pngName = `${stem}_preview.png`;
                mapPath = convertFileSrc(normalized + 'maps/' + pngName);
            } catch (err) {
                renderError = String(err);
            } finally {
                isRendering = false;
            }
        } else {
            mapPath = convertFileSrc(normalized + 'maps/' + filename);
            // For plain images, detect orientation once the image loads
            mapMeta = null;
        }
    }

    /** Called after a plain image loads so we can check its natural dimensions. */
    function onImgLoad() {
        if (!mapMeta && imgEl) {
            rotated = imgEl.naturalHeight > imgEl.naturalWidth;
        }
    }


// ── Coordinate picking ───────────────────────────────────────────────────────

    async function onMouseMove(e: MouseEvent) {
        if (!mapMeta) return;
        const pix = eventToImgPixel(e);
        if (!pix) return;
        mousePixel = { x: Math.round(pix.px), y: Math.round(pix.py) };

        const [wx, wy] = await invoke<[number, number]>("pixel_to_world", {
            px: pix.px,
            py: pix.py,
            meta: mapMeta,
        });
        mouseWorld = { x: wx, y: wy };
    }

    function onMouseLeave() {
        mousePixel = null;
        mouseWorld = null;
    }

    async function onMapClick(e: MouseEvent) {
        if (!mapMeta || !mouseWorld) return;
        pinnedCoords.update(pins => [
            ...pins,
            { id: crypto.randomUUID(), x: mouseWorld!.x, y: mouseWorld!.y }
        ]);
    }

    function removePin(id: string) {
        pinnedCoords.update(pins => pins.filter(p => p.id !== id));
    }

    function copyPin(coord: PinnedCoord) {
        navigator.clipboard.writeText(`${coord.x.toFixed(3)}, ${coord.y.toFixed(3)}`);
    }


// ── Lifecycle ────────────────────────────────────────────────────────────────

    onMount(() => { loadMap(); });
</script>



<div class="frame" aria-hidden="true">
    <div class="header" style="z-index: 100;">
        <button class="reload-button" onclick={reload} title="Reload maps">⟳</button>
    </div>

    {#if openedMap === null}
        <div class="file-modal" style="width: 100%; height: 100%">
            {#if mapFiles.length === 0}
                <p class="muted" style="padding-left: 1rem;">No maps found, please add a map file.</p>
            {:else}
                <div class="file-header">Select a map:</div>
                <div class="file-list">
                    {#each mapFiles as file}
                        <div class="file-row">
                            <button
                                class="file-item {selectedMap === file ? 'selected' : ''}"
                                onclick={() => selectedMap = file}
                            >
                                <span class="map-icon">{needs3DRender(file) ? '🧊' : '🗺️'}</span>
                                <span class="name">{file}</span>
                                {#if needs3DRender(file)}<span class="badge">3D</span>{/if}
                            </button>
                        </div>
                    {/each}
                </div>
                <div class="file-footer">
                    <button class="button secondary" onclick={confirmMapSelection}>
                        Confirm selection
                    </button>
                </div>
            {/if}
        </div>

    {:else if isRendering}
        <div class="center-message">
            <span class="spinner">⏳</span>
            Rendering 3D map to top-down view…
        </div>

    {:else if renderError}
        <div class="center-message error">
            <p>⚠️ Failed to render map:</p>
            <pre>{renderError}</pre>
            <button class="button secondary" onclick={reload}>← Back</button>
        </div>

    {:else}
        <div class="map-container">
            <img
                bind:this={imgEl}
                class="map-img {mapMeta ? 'crosshair' : ''} {rotated ? 'rotated' : ''}"
                src={mapPath}
                alt="Map"
                onload={onImgLoad}
                onmousemove={onMouseMove}
                onmouseleave={onMouseLeave}
                onclick={onMapClick}
            />

            {#if mapMeta && mouseWorld}
                <div class="coord-overlay">
                    <span>px ({mousePixel?.x}, {mousePixel?.y})</span>
                    <span>world ({mouseWorld.x.toFixed(2)} m, {mouseWorld.y.toFixed(2)} m)</span>
                    <span class="hint">Click to pin</span>
                </div>
            {:else if !mapMeta && mousePixel}
                <div class="coord-overlay">px ({mousePixel.x}, {mousePixel.y})</div>
            {/if}

            {#if $pinnedCoords.length > 0}
                <div class="pin-list">
                    <div class="pin-header">Pinned coordinates</div>
                    {#each $pinnedCoords as coord, i}
                        <div class="pin-row">
                            <span class="pin-index">#{i + 1}</span>
                            <span class="pin-coord">{coord.x.toFixed(2)} m, {coord.y.toFixed(2)} m</span>
                            <button class="pin-action" onclick={() => copyPin(coord)} title="Copy">📋</button>
                            <button class="pin-action" onclick={() => removePin(coord.id)} title="Remove">✕</button>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    {/if}
</div>