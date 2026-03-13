import { writable } from "svelte/store";
import type { Waypoint } from "../types";

export const displayedMap = writable<string | null>(null);

export const startPoint = writable<Waypoint>({
    id: "start",
    lat: 0,
    lng: 0
});
export const endPoint = writable<Waypoint>({
    id: "end",
    lat: 0,
    lng: 0
});

export const waypoints = writable<Waypoint[]>([]);