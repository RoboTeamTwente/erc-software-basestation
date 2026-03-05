import { writable } from "svelte/store";

export const displayedMap = writable<string | null>(null);
