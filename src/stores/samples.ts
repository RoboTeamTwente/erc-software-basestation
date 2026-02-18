import { writable } from "svelte/store";
import type { Sample } from "../types"; // define Sample type in a separate file if needed

export const samples = writable<Sample[]>([]);
