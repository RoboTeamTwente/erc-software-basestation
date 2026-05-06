import type { BasestationDetectedObject } from "../proto/components/basestation/detected_object";

export type TrackedState = { complete: boolean };
export type TrackedObject = {
    data: BasestationDetectedObject;
    actions: TrackedState;
};

export const detectedObjectsState = $state({
    objects: [] as TrackedObject[],
    hoveredId: null as number | null,
});

const objectsMap = new Map<number, BasestationDetectedObject>();
const lastSeenAt = new Map<number, number>();
const trackedMap = new Map<number, TrackedState>();

let lastBatchAt = 0;
let avgIntervalMs = 500; // initial guess

export function handleDetectedObjects(batch: BasestationDetectedObject[]) {
    if (!batch.length) return;

    const now = Date.now();
    
    // Update rolling average of how often batches arrive
    if (lastBatchAt > 0) {
        const gap = now - lastBatchAt;
        avgIntervalMs = avgIntervalMs * 0.8 + gap * 0.2; // exponential moving average
    }
    lastBatchAt = now;

    // Prune anything not seen in 5 update cycles
    const staleMs = Math.max(2000, avgIntervalMs * 5);

    for (const obj of batch) {
        if (obj.id === undefined) continue;
        objectsMap.set(obj.id, obj);
        lastSeenAt.set(obj.id, now);
        if (!trackedMap.has(obj.id)) {
            trackedMap.set(obj.id, { complete: false });
        }
    }

    for (const [oid, seenAt] of lastSeenAt.entries()) {
        if (now - seenAt > staleMs) {
            lastSeenAt.delete(oid);
            objectsMap.delete(oid);
        }
    }

    detectedObjectsState.objects = Array.from(objectsMap.entries()).map(([oid, data]) => ({
        data,
        actions: trackedMap.get(oid) ?? { complete: false },
    }));
}