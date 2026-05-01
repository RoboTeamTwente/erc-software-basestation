import { writable } from "svelte/store";

export const displayedMap = writable<string | null>(null);

export interface PinnedCoord {
    id: string;
    x: number;
    y: number;
}

export interface Waypoint {
    id: string;
    x: number;
    y: number;
    label?: string;
}

export const pinnedCoords  = writable<PinnedCoord[]>([]);
export const startPoint    = writable<Waypoint | null>(null);
export const endPoint      = writable<Waypoint | null>(null);
export const waypoints     = writable<Waypoint[]>([]);

// Which waypoint/start/end id is currently hovered in the nav plan list
export const hoveredNavId  = writable<string | null>(null);