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
            measurement: "",
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

    function openCoordinatesModal(index: number) {
        selectedSample = $samples[index];
        selectedIndex = index;
        modalType = "coordinates";
        popup = true;
    }

    function openMeasurementModal(index: number) {
        selectedSample = $samples[index];
        selectedIndex = index;
        modalType = "measurement";
        popup = true;
    }

    function openWeightModal(index: number) {
        selectedSample = $samples[index];
        selectedIndex = index;
        modalType = "weight";
        popup = true;
    }

    function openImageBeforeModal(index: number) {
        selectedSample = $samples[index];
        selectedIndex = index;
        modalType = "image_before";
        popup = true;
    }

    function openImageAfterModal(index: number) {
        selectedSample = $samples[index];
        selectedIndex = index;
        modalType = "image_after";
        popup = true;
    }

// ===============================
// POPULATE FIELDS
// ===============================
    async function handleCoordinates() {
        // TODO: implement coordinates logic
    }

    async function handleMeasurement() {
        // TODO: implement measurement logic
    }

    async function handleWeight() {
        // TODO: implement weight logic
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
                            <button class="delete-button plus-button" title="Fill in coordinates" onclick={() => openCoordinatesModal(i)}>
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
                            <button class="delete-button plus-button" title="Fill in size" onclick={() => openMeasurementModal(i)}>
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
                            <span>{sample.weight}</span>
                            <button class="delete-button plus-button" title="Fill in weight" onclick={() => openWeightModal(i)}>
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
                            <button class="delete-button plus-button" title="Fill in image before sample" onclick={() => openImageBeforeModal(i)}>
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
                            <button class="delete-button plus-button" title="Fill in image after sample" onclick={() => openImageAfterModal(i)}>
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
                <button onclick={closeModal}>&times;</button>
                <h1 class="heading">{selectedSample?.location_name}</h1>

                {#if modalType === "coordinates"}
                    <h3>Fill in Coordinates</h3>
                    <p>Coordinates input placeholder</p>
                {/if}

                {#if modalType === "measurement"}
                    <h3>Fill in Measurement</h3>
                    <p>Measurement input placeholder</p>
                {/if}

                {#if modalType === "weight"}
                    <h3>Fill in Weight</h3>
                    <p>Weight input placeholder</p>
                {/if}

                {#if modalType === "image_before" || modalType == "image_after"}
                    <h3>Take Image Before Sampling</h3>
                    <div class="video-row">
                        <div 
                            class="clickable-video" 
                            role="button"
                            tabindex="0"
                            onclick={() => {
                                if (modalType === "image_before") handleImage(armCamera.port, "before");
                                else if (modalType === "image_after") handleImage(armCamera.port, "after");
                            }}
                            onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") 
                                if (modalType === "image_before") handleImage(armCamera.port, "before"); 
                                else if (modalType === "image_after") handleImage(armCamera.port, "after");
                            }}
                        >
                            <Video camera={armCamera}/>

                            <!-- Overlay shown on hover -->
                            <div class="video-overlay">
                                <span class="camera-icon">📷</span>
                                <span class="overlay-text">Take a picture</span>
                            </div>
                        </div>
                        <div 
                            class="clickable-video" 
                            role="button"
                            tabindex="0"
                            onclick={() => {
                                if (modalType === "image_before") handleImage(frontCamera.port, "before");
                                else if (modalType === "image_after") handleImage(frontCamera.port, "after");
                            }}
                            onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") 
                                if (modalType === "image_before") handleImage(frontCamera.port, "before"); 
                                else if (modalType === "image_after") handleImage(frontCamera.port, "after");
                            }}
                        >
                            <Video camera={frontCamera}/>

                            <!-- Overlay shown on hover -->
                            <div class="video-overlay">
                                <span class="camera-icon">📷</span>
                                <span class="overlay-text">Take a picture</span>
                            </div>
                        </div>
                        <div 
                            class="clickable-video" 
                            role="button"
                            tabindex="0"
                            onclick={() => {
                                if (modalType === "image_before") handleImage(depthCamera.port, "before");
                                else if (modalType === "image_after") handleImage(depthCamera.port, "after");
                            }}
                            onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") 
                                if (modalType === "image_before") handleImage(depthCamera.port, "before"); 
                                else if (modalType === "image_after") handleImage(depthCamera.port, "after");
                            }}
                        >
                            <Video camera={depthCamera}/>

                            <!-- Overlay shown on hover -->
                            <div class="video-overlay">
                                <span class="camera-icon">📷</span>
                                <span class="overlay-text">Take a picture</span>
                            </div>
                        </div>
                    </div>
                {/if}

            </div>
        </div>
    {/if}

</div>