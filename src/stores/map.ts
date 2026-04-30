// src/stores/map.ts
import { writable } from "svelte/store";

export const displayedMap = writable<string | null>(null);

// A pinned coordinate picked from the map
export interface PinnedCoord {
    id: string;
    x: number;   // world metres
    y: number;   // world metres
}

// A waypoint in the navigation plan
export interface Waypoint {
    id: string;
    x: number;
    y: number;
    label?: string;
}

// Pins dropped on the map (shared with nav plan)
export const pinnedCoords = writable<PinnedCoord[]>([]);

// Navigation plan
export const startPoint  = writable<Waypoint | null>(null);
export const endPoint    = writable<Waypoint | null>(null);
export const waypoints   = writable<Waypoint[]>([]);