<script lang="ts">
    import type { Writable } from 'svelte/store';
    import type { InterestLocation } from "$lib/stores/map";
    import '../../global.css';
    import '$lib/css/map.css';

    let { locations, hoveredId }: {
        locations: Writable<InterestLocation[]>;
        hoveredId: Writable<string | null>;
    } = $props();

    let editingId = $state<string | null>(null);
    let editName  = $state("");

    function startEdit(loc: InterestLocation) {
        editingId = loc.id;
        editName  = loc.name;
    }

    function commitEdit(id: string) {
        locations.update(locs =>
            locs.map(l => l.id === id ? { ...l, name: editName.trim() || l.name } : l)
        );
        editingId = null;
    }

    function remove(id: string) {
        locations.update(locs => locs.filter(l => l.id !== id));
    }
</script>

<div class="attached-container">
    <div class="grid-nest" style="grid-template-rows: auto 1fr">

        <div class="grid-item">
            <h1 class="heading" style="padding-bottom: 0">Locations of Interest</h1>
        </div>

        <div class="grid-item" style="flex-direction: column; overflow: hidden;">
            <div class="task-list">

                {#if $locations.length === 0}
                    <p class="muted" style="padding: 0.75rem 1rem;">
                        Click the map to add a location.
                    </p>
                {/if}

                {#each $locations as loc (loc.id)}
                    <div
                        class="loc-row {$hoveredId === loc.id ? 'hovered' : ''}"
                        onmouseenter={() => hoveredId.set(loc.id)}
                        onmouseleave={() => hoveredId.set(null)}
                    >
                        <div class="loc-text">
                            {#if editingId === loc.id}
                                <input
                                    class="name-input"
                                    bind:value={editName}
                                    onblur={() => commitEdit(loc.id)}
                                    onkeydown={e => e.key === 'Enter' && commitEdit(loc.id)}
                                    autofocus
                                />
                            {:else}
                                <span class="loc-name">{loc.name}</span>
                                <span class="loc-coord">{loc.x.toFixed(1)}, {loc.y.toFixed(1)} m</span>
                            {/if}
                        </div>
                        <button class="pin-action" onclick={() => startEdit(loc)} title="Rename">✏️</button>
                        <button class="pin-action" onclick={() => remove(loc.id)} title="Remove">✕</button>
                    </div>
                {/each}

            </div>
        </div>

    </div>
</div>

