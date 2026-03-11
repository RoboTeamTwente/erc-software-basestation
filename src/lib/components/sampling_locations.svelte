<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { invoke } from "@tauri-apps/api/core";

// ----- SVELTE -----
    import { get } from "svelte/store";

// -----IMPORTS -----
    import { samples } from "../../stores/samples";
    import type { Sample } from "../../types";
    import Video from '$lib/components/video.svelte';
    import { armCamera, depthCamera, frontCamera } from "../../state.svelte";


// ----- STATES -----
    let popup = $state(false);
    let selectedSample = $state<Sample>();
    let selectedIndex = $state<number | null>(null);
    let modalType = $state<
        | "coordinates"
        | "measurement"
        | "weight"
        | "image_before"
        | "image_after"
        | null
    >(null);
    const cameras = [armCamera, frontCamera, depthCamera];
    const simpleCameras = [armCamera, frontCamera];

// ===============================
// SAMPLES MANAGEMENT
// ===============================
    function addNewSample() {
        const newSample: Sample = {
            label: `${Date.now()}`,
            location_name: "",
            coordinates: "",
            image_path_before: "",
            image_path_after: "",
            measurement: null,
            weight: null,

            location_name_check: false,
            coordinates_check: false,
            image_path_before_check: false,
            image_path_after_check: false,
            measurement_check: false,
            weight_check: false,
            all_check: false,
        };

        samples.update((arr) => [...arr, newSample]);
    }

    async function deleteSample(index: number) {
        const currentSamples = get(samples);
        const sample = currentSamples[index];
        await invoke("delete_one_file", { directory: "images", fileName: sample.image_path_before });
        await invoke("delete_one_file", { directory: "images", fileName: sample.image_path_after });

        samples.update((arr) => arr.filter((_, i) => i !== index));
    }



// ===============================
// MODAL LOGIC
// ===============================
    function closeModal() {
        popup = false;
        modalType = null;
        selectedIndex = null;
    }

    function openModal(index: number, type: "coordinates" | "measurement" | "weight" | "image_before" | "image_after") {
        selectedSample = $samples[index];
        selectedIndex = index;
        modalType = type;
        popup = true;
    }

// ===============================
// POPULATE FIELDS
// ===============================
    async function handleCoordinates() {
        await invoke("request_coordinates").then((coords) => {
            const typedCoords = coords as [number, number];
            if (selectedIndex === null || !selectedSample) return;

            const coordinatesStr = `Lat: ${typedCoords[0]}, Lon: ${typedCoords[1]}`;

            samples.update(arr => {
                const updated = [...arr];
                updated[selectedIndex as number] = {
                    ...updated[selectedIndex as number],
                    coordinates: coordinatesStr,
                    coordinates_check: true
                };
                return updated;
            });

            selectedSample = {
                ...selectedSample,
                coordinates: coordinatesStr,
                coordinates_check: true
            };
        });
        closeModal();
    }

    async function handleMeasurement(value: number) {
        if (selectedIndex === null || !selectedSample) return;

        samples.update(arr => {
            const updated = [...arr];
            updated[selectedIndex as number] = {
                ...updated[selectedIndex as number],
                measurement: value,
                measurement_check: true
            };
            return updated;
        });

        selectedSample = {
            ...selectedSample,
            measurement: value,
            measurement_check: true
        };
    }

    async function handleWeight() {
        await invoke("request_weight").then((weight) => {
            if (selectedIndex === null || !selectedSample) return;

            const weightNum = weight as number;

            samples.update(arr => {
                const updated = [...arr];
                updated[selectedIndex as number] = {
                    ...updated[selectedIndex as number],
                    weight: weightNum,
                    weight_check: true
                };
                return updated;
            });

            selectedSample = {
                ...selectedSample,
                weight: weightNum,
                weight_check: true
            };
        });
        closeModal();
    }

    async function handleImage(port: string, phase: string) {
        if (selectedIndex === null || !selectedSample) return;

        const newImagePath = `${selectedSample.label}_${phase}`;

        const pathKey = `image_path_${phase}` as
            | "image_path_before"
            | "image_path_after";

        const checkKey = `image_path_${phase}_check` as
            | "image_path_before_check"
            | "image_path_after_check";

        samples.update(arr => {
            const updated = [...arr];
            updated[selectedIndex as number] = {
                ...updated[selectedIndex as number],
                [pathKey]: newImagePath,
                [checkKey]: true
            };
            return updated;
        });

        selectedSample = {
            ...selectedSample,
            [pathKey]: newImagePath,
            [checkKey]: true
        };

        await invoke("save_snapshot", { port, fileName: newImagePath });
        closeModal();
    }

</script>


<div class="container">
    <div class="grid-nest" style="grid-template-rows: auto 1fr">

        <div class="grid-item">
            <h1 class="heading"> Sampling  Locations</h1>
        </div> 

        <div class="grid-item">
            <div class="task-list">
                {#each $samples as sample, i}
                    <div class="task-card no-hover">
                        <!-- Location Name (typed by user) -->  
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-bottom: 0.8rem">
                            <label class="custom-checkbox" style="font-size: 1.5rem">
                                <input type="checkbox" 
                                        checked={
                                            sample.location_name_check &&
                                            sample.coordinates_check &&
                                            sample.image_path_before_check &&
                                            sample.image_path_after_check &&
                                            sample.measurement_check &&
                                            sample.weight_check
                                        } 
                                        disabled
                                        />
                                <span></span>
                            </label>
                            <input 
                                class="editable-text"
                                type="text"
                                placeholder="Location Name"
                                bind:value={$samples[i].location_name}
                                oninput={() => $samples[i].location_name_check = $samples[i].location_name !== ""}
                            />
                            <button class="delete-button" title="Delete Sample" onclick={() => deleteSample(i)}>
                                <img class="right-icon" src="/delete.svg" alt="Delete" />
                            </button>
                        </div> 

                        <!-- Coordinates (from backend later) -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={$samples[i].coordinates_check} />
                                <span></span>
                            </label>
                            <span><strong>Coordinates: </strong></span>
                            <span>{sample.coordinates}</span>
                            <button class="delete-button plus-button" title="Fill in coordinates" onclick={() => openModal(i, "coordinates")}>
                                +
                            </button>
                        </div>

                        <!-- Measurement -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={$samples[i].measurement_check} />
                                <span></span>
                            </label>
                            <span><strong>Size: </strong></span>
                            <span>{sample.measurement}</span>
                            <button class="delete-button plus-button" title="Fill in size" onclick={() => openModal(i, "measurement")}>
                                +
                            </button>
                        </div>

                        <!-- Weight -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={$samples[i].weight_check} />
                                <span></span>
                            </label>
                            <span><strong>Weight: </strong></span>
                            <span>{sample.weight} {#if sample.weight_check} grams {/if}</span>
                            <button class="delete-button plus-button" title="Fill in weight" onclick={() => openModal(i, "weight")}>
                                +
                            </button>
                        </div>

                        <!-- Image path before sampling -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={$samples[i].image_path_before_check} />
                                <span></span>
                            </label>
                            <span><strong>Image before sampling: </strong></span>
                            <span>{sample.image_path_before}</span>
                            <button class="delete-button plus-button" title="Fill in image before sample" onclick={() => openModal(i, "image_before")}>
                                +
                            </button>
                        </div>

                        <!-- Image path after sampling-->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={$samples[i].image_path_after_check} />
                                <span></span>
                            </label>
                            <span><strong>Image after sampling: </strong></span>
                            <span>{sample.image_path_after}</span>
                            <button class="delete-button plus-button" title="Fill in image after sample" onclick={() => openModal(i, "image_after")}>
                                +
                            </button>
                        </div>

                    </div>
                {/each}
            </div>
        </div>

        <div class="grid-item">
            <button class="button" onclick={addNewSample}>
                + Add Sampling Location
            </button>
        </div>
    </div>

    {#if popup}
        <div class="modal-overlay">
            <div class="modal">
                <button class="close-button" onclick={closeModal}>&times;</button>
                <h1 class="heading">{selectedSample?.location_name}</h1>

                {#if modalType === "coordinates"}
                    <h3>Fill in Coordinates</h3>
                    <button class="button" onclick={handleCoordinates}>
                        Get Coordinates from Rover
                    </button>
                {/if}

                {#if modalType === "measurement"}
                    <h3>Fill in Measurement</h3>
                        <div class="video-row">
                            {#each simpleCameras as cam}
                                <div class="clickable-video">
                                    <Video camera={cam} pixelMode={true} measure={true} on:measurement={(e) => handleMeasurement(e.detail)}/>
                                </div>
                            {/each}
                        </div>
                    <button class="button" style="align-self: flex-end;" onclick={closeModal}>
                        Done
                    </button>
                {/if}

                {#if modalType === "weight"}
                    <h3>Fill in Weight</h3>
                    <button class="button" onclick={handleWeight}>
                        Get Weight from Rover
                    </button>
                {/if}

                {#if modalType === "image_before" || modalType === "image_after"}
                    <h3>Take Image {modalType === "image_before" ? "Before Sampling" : "After Sampling"}</h3>
                    <div class="video-row">
                        {#each cameras as cam}
                            <div 
                                class="clickable-video" 
                                role="button"
                                tabindex="0"
                                onclick={() => handleImage(cam.port, modalType === "image_before" ? "before" : "after")}
                                onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") handleImage(cam.port, modalType === "image_before" ? "before" : "after") }}
                            >
                                <Video camera={cam} pixelMode={false} measure={false}/>
                                <div class="video-overlay">
                                    <span>
                                        <img src="/camera.svg" alt="Camera Icon"/>
                                    </span>
                                    <span class="overlay-text">Take a picture</span>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}

            </div>
        </div>
    {/if}

</div>