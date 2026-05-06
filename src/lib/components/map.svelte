<script lang="ts">
    let { mode = 'navigation' } = $props<{ mode?: 'navigation' | 'science' | 'probing' }>();

    import { invoke } from "@tauri-apps/api/core";
    import { convertFileSrc } from '@tauri-apps/api/core';
    import { appDataDir } from '@tauri-apps/api/path';
    import { listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte';
    import { get } from "svelte/store";
    import '../../global.css';
    import { displayedMap, pinnedCoords, waypoints, startPoint, endPoint, hoveredNavId } from "../stores/map";
    import type { PinnedCoord } from "../stores/map";
    import { gpsPosition, scienceLocations, probingLocations } from '../stores/map';
    import { hoveredScienceId, hoveredProbingId } from '../stores/map';
    import type { GpsPosition } from '../stores/map';

    interface MapMeta {
        img_width:        number;
        img_height:       number;
        world_x_min:      number;
        world_y_min:      number;
        metres_per_pixel: number;
        format:           string;
        rotated:          boolean;
    }

    let mapFiles     = $state<string[]>([]);
    let selectedMap  = $state<string | null>(null);
    let openedMap    = $state<string | null>(null);
    let mapPath      = $state<string>("");
    let mapMeta      = $state<MapMeta | null>(null);
    let isRendering  = $state(false);
    let renderError  = $state<string | null>(null);
    let imgEl        = $state<HTMLImageElement | null>(null);
    let mousePixel   = $state<{ x: number; y: number } | null>(null);
    let mouseWorld   = $state<{ x: number; y: number } | null>(null);
    let rotated      = $state(false);
    let hoveredPinId = $state<string | null>(null);

    const NEEDS_RENDER = ["obj", "las", "laz", "e57"];

    function fileExt(name: string) { return name.split('.').pop()?.toLowerCase() ?? ""; }
    function needs3DRender(name: string) { return NEEDS_RENDER.includes(fileExt(name)); }

    // ── Letterbox geometry ────────────────────────────────────────────────────
    function getRenderedRect(rect: DOMRect): { rW: number; rH: number; oX: number; oY: number } {
        if (!mapMeta) return { rW: rect.width, rH: rect.height, oX: 0, oY: 0 };

        const imgAspect = mapMeta.img_width / mapMeta.img_height;
        const elAspect  = rect.width / rect.height;

        let rW: number, rH: number, oX: number, oY: number;
        if (imgAspect > elAspect) {
            rW = rect.width;
            rH = rect.width / imgAspect;
            oX = 0;
            oY = (rect.height - rH) / 2;
        } else {
            rH = rect.height;
            rW = rect.height * imgAspect;
            oX = (rect.width - rW) / 2;
            oY = 0;
        }
        return { rW, rH, oX, oY };
    }

    // ── Mouse → PNG pixel ─────────────────────────────────────────────────────
    function eventToImgPixel(e: MouseEvent): { px: number; py: number } | null {
        if (!imgEl || !mapMeta) return null;
        const rect = imgEl.getBoundingClientRect();
        const { rW, rH, oX, oY } = getRenderedRect(rect);

        const relX = e.clientX - rect.left - oX;
        const relY = e.clientY - rect.top  - oY;

        if (relX < 0 || relY < 0 || relX > rW || relY > rH) return null;

        return {
            px: relX / rW * mapMeta.img_width,
            py: (1 - relY / rH) * mapMeta.img_height,
        };
    }

    // ── World coord → CSS position over the <img> element ────────────────────
    function worldToCSSPos(wx: number, wy: number): { left: string; top: string } | null {
        if (!imgEl || !mapMeta) return null;
        const rect = imgEl.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) return null;
        const { rW, rH, oX, oY } = getRenderedRect(rect);

        const relX = (wx / mapMeta.metres_per_pixel) / mapMeta.img_width  * rW;
        const relY = (1 - (wy / mapMeta.metres_per_pixel) / mapMeta.img_height) * rH;

        return {
            left: `${((oX + relX) / rect.width  * 100).toFixed(3)}%`,
            top:  `${((oY + relY) / rect.height * 100).toFixed(3)}%`,
        };
    }

    // ── Map management ────────────────────────────────────────────────────────
    async function loadMap() {
        const stored = get(displayedMap);
        if (stored) { openedMap = stored; await openMap(stored); }
        else { await listMaps(); }
    }

    async function listMaps() {
        mapFiles = await invoke<string[]>("list_task_files", { directory: "maps" });
        if (mapFiles.length === 1) { selectedMap = mapFiles[0]; await confirmMapSelection(); }
    }

    async function reload() {
        openedMap = null; selectedMap = null; mapPath = "";
        mapMeta = null; renderError = null; rotated = false;
        pinnedCoords.set([]); displayedMap.set(null);
        await listMaps();
    }

    async function confirmMapSelection() {
        if (!selectedMap) return;
        openedMap = selectedMap;
        displayedMap.set(openedMap);
        await openMap(openedMap);
    }

    async function openMap(filename: string) {
        renderError = null; mapMeta = null; mapPath = ""; rotated = false;
        const base = await appDataDir();
        const normalized = base.endsWith('/') ? base : base + '/';

        if (needs3DRender(filename)) {
            isRendering = true;
            try {
                const meta = await invoke<MapMeta>("render_map", { filename });
                mapMeta    = meta;
                rotated    = meta.rotated;
                const stem = filename.replace(/\.[^.]+$/, "");
                mapPath    = convertFileSrc(normalized + 'maps/' + stem + '_preview.png');
            } catch (err) { renderError = String(err); }
            finally { isRendering = false; }
        } else {
            mapPath = convertFileSrc(normalized + 'maps/' + filename);
        }
    }

    function onImgLoad() {
        if (!mapMeta && imgEl) rotated = imgEl.naturalHeight > imgEl.naturalWidth;
    }

    // ── Interaction ───────────────────────────────────────────────────────────
    async function onMouseMove(e: MouseEvent) {
        if (!mapMeta) return;
        const pix = eventToImgPixel(e);
        if (!pix) { mousePixel = null; mouseWorld = null; return; }

        mousePixel = { x: Math.round(pix.px), y: Math.round(pix.py) };
        const [wx, wy] = await invoke<[number, number]>("pixel_to_world", {
            px: pix.px, py: pix.py, meta: mapMeta,
        });
        mouseWorld = { x: wx, y: wy };
    }

    function onMouseLeave() { mousePixel = null; mouseWorld = null; }

    async function onMapClick(e: MouseEvent) {
        if (!mapMeta || !mouseWorld) return;

        if (mode === 'navigation') {
            pinnedCoords.update(pins => [
                ...pins,
                { id: crypto.randomUUID(), x: mouseWorld!.x, y: mouseWorld!.y }
            ]);
        } else if (mode === 'science') {
            scienceLocations.update(locs => [
                ...locs,
                { id: crypto.randomUUID(), x: mouseWorld!.x, y: mouseWorld!.y, name: `Location ${locs.length + 1}` }
            ]);
        } else if (mode === 'probing') {
            probingLocations.update(locs => [
                ...locs,
                { id: crypto.randomUUID(), x: mouseWorld!.x, y: mouseWorld!.y, name: `Location ${locs.length + 1}` }
            ]);
        }
    }

    function removePin(id: string) { pinnedCoords.update(p => p.filter(x => x.id !== id)); }
    function copyPin(coord: PinnedCoord) {
        navigator.clipboard.writeText(`${coord.x.toFixed(3)}, ${coord.y.toFixed(3)}`);
    }

    onMount(async () => {
        await loadMap();
        await listen<{ latitude: number; longitude: number; heading: number }>('gps-update', e => {
            gpsPosition.set({
                x: e.payload.longitude,
                y: e.payload.latitude,
                heading: e.payload.heading,
            });
        });
    });
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
                    <button class="button secondary" onclick={confirmMapSelection}>Confirm selection</button>
                </div>
            {/if}
        </div>

    {:else if isRendering}
        <div class="center-message"><span class="spinner">⏳</span> Rendering 3D map to top-down view…</div>

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
                class="map-img {mapMeta ? 'crosshair' : ''}"
                src={mapPath}
                alt="Map"
                onload={onImgLoad}
                onmousemove={onMouseMove}
                onmouseleave={onMouseLeave}
                onclick={onMapClick}
            />

            {#if mapMeta}
                {#if mode === 'navigation'}
                    {#each $pinnedCoords as pin, i}
                        {@const pos = worldToCSSPos(pin.x, pin.y)}
                        {#if pos}
                            <div
                                class="map-pin pin-unassigned {hoveredPinId === pin.id ? 'highlighted' : ''}"
                                style="left:{pos.left}; top:{pos.top}; pointer-events: auto;"
                                onmouseover={() => hoveredPinId = pin.id}
                                onmouseout={() => hoveredPinId = null}
                            >
                                <div class="map-pin-dot" style="pointer-events: none;"></div>
                                <div class="map-pin-label" style="pointer-events: none;">#{i + 1}</div>
                            </div>
                        {/if}
                    {/each}

                    {#each [
                        ...($startPoint ? [{ id: $startPoint.id, x: $startPoint.x, y: $startPoint.y, label: '▶', kind: 'start' }] : []),
                        ...$waypoints.map((wp, i) => ({ id: wp.id, x: wp.x, y: wp.y, label: `${i + 1}`, kind: 'waypoint' })),
                        ...($endPoint   ? [{ id: $endPoint.id,   x: $endPoint.x,   y: $endPoint.y,   label: '⏹', kind: 'end' }] : []),
                    ] as item}
                        {@const pos = worldToCSSPos(item.x, item.y)}
                        {#if pos}
                            <div
                                class="map-pin pin-{item.kind} {$hoveredNavId === item.id ? 'highlighted' : ''}"
                                style="left:{pos.left}; top:{pos.top}; pointer-events: auto;"
                                onmouseover={() => hoveredNavId.set(item.id)}
                                onmouseout={() => hoveredNavId.set(null)}
                            >
                                <div class="map-pin-dot" style="pointer-events: none;"></div>
                                <div class="map-pin-label" style="pointer-events: none;">{item.label}</div>
                            </div>
                        {/if}
                    {/each}
                

                {:else if mode === 'science'}
                    {#each $scienceLocations as loc, i (loc.id)}
                        {@const pos = worldToCSSPos(loc.x, loc.y)}
                        {#if pos}
                            <div
                                class="map-pin pin-unassigned {$hoveredScienceId === loc.id ? 'highlighted' : ''}"
                                style="left:{pos.left}; top:{pos.top}; pointer-events: auto;"
                                onmouseover={() => hoveredScienceId.set(loc.id)}
                                onmouseout={() => hoveredScienceId.set(null)}
                            >
                                <div class="map-pin-dot" style="pointer-events: none;"></div>
                                <div class="map-pin-label" style="pointer-events: none;">{i + 1}</div>
                            </div>
                        {/if}
                    {/each}

                {:else if mode === 'probing'}
                    {#each $probingLocations as loc, i (loc.id)}
                        {@const pos = worldToCSSPos(loc.x, loc.y)}
                        {#if pos}
                            <div
                                class="map-pin pin-unassigned {$hoveredProbingId === loc.id ? 'highlighted' : ''}"
                                style="left:{pos.left}; top:{pos.top}; pointer-events: auto;"
                                onmouseover={() => hoveredProbingId.set(loc.id)}
                                onmouseout={() => hoveredProbingId.set(null)}
                            >
                                <div class="map-pin-dot" style="pointer-events: none;"></div>
                                <div class="map-pin-label" style="pointer-events: none;">{i + 1}</div>
                            </div>
                        {/if}
                    {/each}
                {/if}

            {/if}

            {#if mode === 'navigation' && $pinnedCoords.length > 0}
                <div class="pin-list">
                    <div class="pin-header">Pinned coordinates</div>
                    {#each $pinnedCoords as coord, i}
                        <div
                            class="pin-row"
                            onmouseenter={() => hoveredPinId = coord.id}
                            onmouseleave={() => hoveredPinId = null}
                        >
                            <span class="pin-index">#{i + 1}</span>
                            <span class="pin-coord">{coord.x.toFixed(2)} m, {coord.y.toFixed(2)} m</span>
                            <button class="pin-action" onclick={() => copyPin(coord)} title="Copy">📋</button>
                            <button class="pin-action" onclick={() => removePin(coord.id)} title="Remove">✕</button>
                        </div>
                    {/each}
                </div>
            {/if}

            {#if mapMeta && mouseWorld && mousePixel}
                <div class="coord-overlay">
                    <span>px ({mousePixel.x}, {mousePixel.y})</span>
                    <span>world ({mouseWorld.x.toFixed(2)} m, {mouseWorld.y.toFixed(2)} m)</span>
                    <span class="hint">Click to pin</span>
                </div>
            {/if}

            {#if $gpsPosition && mapMeta}
                {@const gpsPos = worldToCSSPos($gpsPosition.x, $gpsPosition.y)}
                {#if gpsPos}
                    <div
                        class="gps-marker"
                        style="left:{gpsPos.left}; top:{gpsPos.top}; --heading:{$gpsPosition.heading}deg;"
                    >
                        <div class="gps-arrow">▲</div>
                    </div>
                {/if}
            {/if}
        </div>
    {/if}
</div>