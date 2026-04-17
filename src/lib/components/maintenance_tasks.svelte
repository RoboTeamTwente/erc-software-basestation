<script lang="ts">
    import { listen } from '@tauri-apps/api/event';
    import { onMount } from "svelte";
    import {
        BasestationDetectedObject,
        detectedObjectTypeToJSON
    } from "../proto/components/basestation/detected_object";

    type TrackedState = {
        complete: boolean;
    };

    type TrackedObject = {
        data: BasestationDetectedObject;
        actions: TrackedState;
    };

    // live objects only (can disappear)
    let objectsMap = new Map<number, BasestationDetectedObject>();
    let lastSeen = new Map<number, number>();

    // persistent state (never deleted unless you explicitly want to reset it)
    let trackedMap = new Map<number, TrackedState>();

    let objects: TrackedObject[] = [];

    function syncObjects() {
        objects = Array.from(objectsMap.entries()).map(([id, data]) => {
            return {
                data,
                actions: trackedMap.get(id) ?? { complete: false }
            };
        });
    }

    function pruneOld(currentFrame: number) {
        for (const [id, lastFrame] of lastSeen.entries()) {
            if (currentFrame - lastFrame > 5) {
                lastSeen.delete(id);
                objectsMap.delete(id);
                // NOTE: trackedMap is NOT deleted
            }
        }
    }

    function formatType(type: number | undefined) {
        const raw = detectedObjectTypeToJSON(type ?? 0);
        return raw.replace("OBJECT_", "").replaceAll("_", " ");
    }

    onMount(() => {
        const unlisten = listen("detected-objects-update", (event) => {
            const obj = BasestationDetectedObject.fromJSON(event.payload);

            if (obj.id === undefined || obj.frame_id === undefined) return;

            const id = obj.id;
            const frame = obj.frame_id;

            // update live object
            objectsMap.set(id, obj);
            lastSeen.set(id, frame);

            // ensure persistent tracking exists
            if (!trackedMap.has(id)) {
                trackedMap.set(id, {
                    complete: false
                });
            }

            pruneOld(frame);
            syncObjects();
        });

        return () => {
            unlisten.then((f) => f());
        };
    });
</script>


<div class="container">
    <div class="grid-nest" style="grid-template-columns: 1fr; grid-template-rows: 1fr 8fr 1fr;">
        
        <div class="grid-item">
            <h1 class="heading">Maintenance Panel Actions</h1>
        </div>

        <div class="grid-item">
            <div class="task-list">

                {#each objects as obj (obj.data.id)}
                    <div class="task-card">

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
            <button class="button" style="align-self: flex-end;">
                Request Actions Identification
            </button>
        </div>

    </div>
</div>