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

const STALE_MS = 2000;

export function handleDetectedObjects(batch: BasestationDetectedObject[]) {
    if (!batch.length) return;

    const now = Date.now();

    for (const obj of batch) {
        if (obj.id === undefined) continue;
        objectsMap.set(obj.id, obj);
        lastSeenAt.set(obj.id, now);
        if (!trackedMap.has(obj.id)) {
            trackedMap.set(obj.id, { complete: false });
        }
    }

    for (const [oid, seenAt] of lastSeenAt.entries()) {
        if (now - seenAt > STALE_MS) {
            lastSeenAt.delete(oid);
            objectsMap.delete(oid);
        }
    }

    detectedObjectsState.objects = Array.from(objectsMap.entries()).map(([oid, data]) => ({
        data,
        actions: trackedMap.get(oid) ?? { complete: false },
    }));
}