<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from '@tauri-apps/api/event';
    import { onMount } from "svelte";
    import { BasestationDetectedObject, detectedObjectTypeToJSON } from "../proto/components/basestation/detected_object";
    import { detectedObjectsState, handleDetectedObject } from "../state/detectedObjects.svelte";

    function formatType(type: number | undefined) {
        const raw = detectedObjectTypeToJSON(type ?? 0);
        return raw.replace("OBJECT_", "").replaceAll("_", " ");
    }

    async function startObjectDummy(){
        try {
            await invoke("start_detection_sim");
        } catch (e) {
            console.error("startObjectDummy failed:", e);
        }
    }

    async function stopGenDummy(){
        try {
            await invoke("stop_dummy_streams");
        } catch (e) {
            console.error("stopGenDummy failed:", e);
        }
    }

    onMount(() => {
        const unlisten = listen("detected-objects-update", (event) => {
            const obj = BasestationDetectedObject.fromJSON(event.payload);
            handleDetectedObject(obj);
        });
        return () => { unlisten.then((f) => f()); };
    });
</script>


<div class="container">
    <div class="grid-nest" style="grid-template-columns: 1fr; grid-template-rows: 1fr 10fr 2fr;">
        
        <div class="grid-item">
            <h1 class="heading">Maintenance Panel Actions</h1>
        </div>

        <div class="grid-item">
            <div class="task-list">

                {#each detectedObjectsState.objects as obj (obj.data.id)}
                    <div
                        class="task-card"
                        class:hovered={detectedObjectsState.hoveredId === obj.data.id}
                        style="cursor:pointer;"
                        onmouseenter={() => detectedObjectsState.hoveredId = obj.data.id ?? null}
                        onmouseleave={() => detectedObjectsState.hoveredId = null}
                        onclick={() => invoke("select_object", { objectId: obj.data.id })}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => e.key === 'Enter' && invoke("select_object", { objectId: obj.data.id })}
                    >
                        <div class="task-info">
                            <span>
                                <strong>ID:</strong> {obj.data.id}
                            </span>

                            <span>
                                <strong>Type:</strong> {formatType(obj.data.type)}
                            </span>

                            <span>
                                <strong>Confidence:</strong> {obj.data.confidence}
                            </span>

                            <span>
                                <strong>Complete:</strong>
                                {obj.actions.complete ? "Yes" : "No"}
                            </span>
                        </div>

                    </div>
                {/each}

            </div>
        </div>

        <div class="grid-item" style="flex-direction: column;">
            <button class="button" style="align-self: flex-end;" onclick={() => startObjectDummy()}>
                Request Actions Identification
            </button>
            <button class="button" style="align-self: flex-end;" onclick={() => stopGenDummy()}>
                Stop Actions Identification
            </button>
        </div>

    </div>
</div>