import { writable } from "svelte/store";
import type { Sample } from "../types"; 

export const samples = writable<Sample[]>([]);
