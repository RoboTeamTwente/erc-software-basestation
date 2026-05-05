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

export interface GpsPosition {
    x: number;  // longitude field reused as map X metres
    y: number;  // latitude field reused as map Y metres
    heading: number;
}

export const gpsPosition = writable<GpsPosition | null>(null);

export const pinnedCoords  = writable<PinnedCoord[]>([]);
export const startPoint    = writable<Waypoint | null>(null);
export const endPoint      = writable<Waypoint | null>(null);
export const waypoints     = writable<Waypoint[]>([]);

// Which waypoint/start/end id is currently hovered
export const hoveredNavId  = writable<string | null>(null);
export const hoveredScienceId = writable<string | null>(null);
export const hoveredProbingId = writable<string | null>(null);

export interface InterestLocation {
    id: string;
    x: number;
    y: number;
    name: string;
}
export const scienceLocations = writable<InterestLocation[]>([]);
export const probingLocations = writable<InterestLocation[]>([]);