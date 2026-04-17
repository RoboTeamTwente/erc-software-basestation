import type { BasestationDetectedObject } from "../proto/components/basestation/detected_object";

export type TrackedState = { complete: boolean };
export type TrackedObject = {
    data: BasestationDetectedObject;
    actions: TrackedState;
};

// Reactive shared state
export const detectedObjectsState = $state({
    objects: [] as TrackedObject[],
});

const objectsMap = new Map<number, BasestationDetectedObject>();
const lastSeen   = new Map<number, number>();
const trackedMap = new Map<number, TrackedState>();

export function handleDetectedObject(obj: BasestationDetectedObject) {
    if (obj.id === undefined || obj.frame_id === undefined) return;

    const id    = obj.id;
    const frame = obj.frame_id;

    objectsMap.set(id, obj);
    lastSeen.set(id, frame);

    if (!trackedMap.has(id)) {
        trackedMap.set(id, { complete: false });
    }

    // prune objects not seen in the last 5 frames
    for (const [oid, lastFrame] of lastSeen.entries()) {
        if (frame - lastFrame > 5) {
            lastSeen.delete(oid);
            objectsMap.delete(oid);
        }
    }

    detectedObjectsState.objects = Array.from(objectsMap.entries()).map(([oid, data]) => ({
        data,
        actions: trackedMap.get(oid) ?? { complete: false },
    }));
}