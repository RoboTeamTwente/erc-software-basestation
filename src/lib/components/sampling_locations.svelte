<script lang="ts">
// -----TYPES -----
    type Sample = {
        location_name: string,
        coordinates: string,
        image_path_before: string,
        image_path_after: string,
        measurement: string,
        weight: number | null,

        location_name_check: boolean,
        coordinates_check: boolean,
        image_path_before_check: boolean,
        image_path_after_check: boolean,
        measurement_check: boolean,
        weight_check: boolean,
        all_check: boolean,
    }


// ----- STATES -----
    let samples = $state<Sample[]>([]);


// ===============================
// SAMPLES MANAGEMENT
// ===============================
    function addNewSample(){
        const newSample: Sample = {
            location_name: "",
            coordinates: "",
            image_path_before: "",
            image_path_after: "",
            measurement: "",
            weight: null,

            location_name_check: false,
            coordinates_check: false,
            image_path_before_check: false,     // auto-generated
            image_path_after_check: false,
            measurement_check: false,
            weight_check: false,
            all_check: false,
        };

        samples = [...samples, newSample]; // important for reactivity
    }
    function deleteSample(index: number) {
        samples = samples.filter((_, i) => i !== index);
    }
    $effect(() => {
        samples.forEach(sample => {
            sample.all_check = !!(
                sample.location_name_check &&
                sample.coordinates_check &&
                sample.image_path_before_check &&
                sample.image_path_after_check &&
                sample.measurement_check &&
                sample.weight_check
            );
        });

        // Trigger Svelte reactivity
        samples = samples;
    });

</script>


<div class="container">
    <div class="grid-nest" style="grid-template-rows: 1fr 14fr 2fr">

        <div class="grid-item">
            <h1 class="heading"> Sampling  Locations</h1>
        </div> 

        <div class="grid-item">
            <div class="task-list">
                {#each samples as sample, i}
                    <div class="task-card no-hover">
                        <!-- Location Name (typed by user) -->  
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-bottom: 0.8rem">
                            <label class="custom-checkbox" style="font-size: 1.5rem">
                                <input type="checkbox" bind:checked={samples[i].all_check} />
                                <span></span>
                            </label>
                            <input 
                                class="editable-text"
                                type="text"
                                placeholder="Location Name"
                                bind:value={samples[i].location_name}
                                oninput={() => samples[i].location_name_check = samples[i].location_name !== ""}
                            />
                            <button class="delete-button" title="Delete Sample" onclick={() => deleteSample(i)}>
                                <img class="right-icon" src="/delete.svg" alt="Delete" />
                            </button>
                        </div> 

                        <!-- Coordinates (from backend later) -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={samples[i].coordinates_check} />
                                <span></span>
                            </label>
                            <span><strong>Coordinates: </strong></span>
                            <span>{sample.coordinates}</span>
                        </div>

                        <!-- Measurement -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={samples[i].measurement_check} />
                                <span></span>
                            </label>
                            <span><strong>Measurement: </strong></span>
                            <span>{sample.measurement}</span>
                        </div>

                        <!-- Weight -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={samples[i].weight_check} />
                                <span></span>
                            </label>
                            <span><strong>Weight: </strong></span>
                            <span>{sample.weight}</span>
                        </div>

                        <!-- Image path before sampling -->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={samples[i].image_path_before_check} />
                                <span></span>
                            </label>
                            <span><strong>Image before sampling: </strong></span>
                            <span>{sample.image_path_before}</span>
                        </div>

                        <!-- Image path after sampling-->
                        <div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
                            <label class="custom-checkbox">
                                <input type="checkbox" bind:checked={samples[i].image_path_after_check} />
                                <span></span>
                            </label>
                            <span><strong>Image after sampling: </strong></span>
                            <span>{sample.image_path_after}</span>
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
</div>