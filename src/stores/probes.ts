import { writable } from "svelte/store";
import type { Probe } from "../types"; 

export const probes = writable<Probe[]>([]);