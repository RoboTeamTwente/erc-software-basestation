<script lang="ts">
// ----- TAURI / EXTERNAL -----
    import { invoke } from "@tauri-apps/api/core";
    import { confirm } from '@tauri-apps/plugin-dialog';
    import { convertFileSrc } from '@tauri-apps/api/core';
    import { appDataDir } from '@tauri-apps/api/path';

// ----- SVELTE -----
    import { onMount } from 'svelte';

// ----- STYLES -----
    import '../../global.css';
    import type { Sample } from "../../types";


// ----- TYPES -----
    type Task = {
        task_name: string;
        task_number: string;
        completion_time: string;
        finished_at: string;
        attached_content: Sample[];
        file_name: string;
    }


// ----- STATE -----
    let taskFiles = $state<string[]>([]);
    let tasks = $state<Task[]>([]);
    let selectedTask = $state<Task | null>(null);
    let showImage = $state(false);
    let imageBeforeUrl = $state("");
    let imageAfterUrl = $state("");


// ----- UTILITIES -----
    function formatDate(dateStr: string) {
        const date = new Date(dateStr);
        return date.toLocaleString();
    }


// ===============================
// TASK FILE MANAGEMENT
// ===============================
    async function listFiles() {
        const result = await invoke<string[]>("list_task_files", { directory: "tasks" });
        if (result.length > 0) {
            taskFiles = result;
            await loadTasks();
        } else {
            taskFiles = [];
        }
    }

    async function loadTasks() {
        tasks = [];
        for (const file of taskFiles) {
            try {
                const data = await invoke<number[]>("read_task_file", { fileName: file });
                const uint8Array = new Uint8Array(data);
                const decoder = new TextDecoder("utf-8");
                const content = decoder.decode(uint8Array);

                tasks.push(JSON.parse(content));
            } catch (e) {
                console.error(`Failed to read ${file}:`, e);
            }
        }
    }

    async function deleteTask(task:Task) {
        const confirmed = await confirm(
        "Are you sure you want to delete this task?",
        { title: "Delete", kind: "warning" }
        );

        if (confirmed) {
            await invoke("delete_one_file", {
                directory: "tasks",
                fileName: task.file_name,
            })
            loadTasks();
        }
    }

    async function displayImage(before:string, after: string) {
        const directory = await appDataDir();
        const imageBeforePath = `${directory}/images/${before}.jpg`;
        const imageAfterPath = `${directory}/images/${after}.jpg`;
        imageBeforeUrl = convertFileSrc(imageBeforePath);
        imageAfterUrl = convertFileSrc(imageAfterPath);
        showImage = true;
    }
    function closeImage(){
        imageBeforeUrl = "";
        imageAfterUrl = "";
        showImage = false;
    }
// ===============================
// POPUP
// ===============================
    function selectTask(task: Task) {
        selectedTask = task;
    }

    function closeModal() {
        selectedTask = null;
    }



// ===============================
// LIFECYCLE
// ===============================
	onMount(() => {
		listFiles();
	});
</script>


<div class="container">
    <div class="grid-nest" style="grid-template-rows: auto 1fr">

        <div class="grid-item">
            <h1 class="heading">Task Completion Overview</h1>
            
            <button class="reload-button" onclick={listFiles} title="Reload tasks">
                ⟳
            </button>
        </div>

        <div class="grid-item">
            {#if tasks.length === 0}
                <p>No tasks found.</p>
            {:else}
                <div class="task-list">
                    {#each tasks as task}
                        <div 
                            class="task-card" 
                            role="button" 
                            tabindex="0" 
                            onclick={() => selectTask(task)} 
                            onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") selectTask(task); }}
                        >
                            <div class="task-info">
                                <span><strong>{task.task_name}</strong></span>
                                <span>#{task.task_number}</span>
                                <span>Time: {task.completion_time}</span>
                                <span>Finished: {formatDate(task.finished_at)}</span>
                                
                                <button
                                    class="delete-button"
                                    onclick={(event) => {
                                        event.stopPropagation();
                                        deleteTask(task);
                                        }}
                                    title="Delete task"
                                >
                                    <img class="right-icon" src="/delete.svg" alt="Delete" />
                                </button>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    </div>

    {#if selectedTask}
        <div class="modal-overlay">
            <div class="modal">
                <button onclick={closeModal}>&times;</button>
                <h2>{selectedTask.task_name} (#{selectedTask.task_number})</h2>
                <p><strong>Completion Time:</strong> {selectedTask.completion_time}</p>
                <p><strong>Finished At:</strong> {formatDate(selectedTask.finished_at)}</p>

                <div class="task-list" style="background-color: var(--color-offwhite);">
                    {#each selectedTask.attached_content as sample}
                        <hr />
                        <p><strong>Location:</strong> {sample.location_name}</p>
                        <p><strong>Coordinates:</strong> {sample.coordinates}</p>
                        <p><strong>Measurement:</strong> {sample.measurement}</p>
                        <p><strong>Weight:</strong> {sample.weight}</p>
                        <p>
                            <strong>Picture before sampling:</strong>
                            <span
                                class="image-link"
                                role="button"
                                tabindex="0"
                                onclick={() => displayImage(sample.image_path_before, sample.image_path_after)}
                                onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") displayImage(sample.image_path_before, sample.image_path_after); }}
                            >
                                {sample.image_path_before}
                            </span>
                        
                        </p>
                        <p><strong>Picture after sampling:</strong> 
                            <span
                                class="image-link"
                                role="button"
                                tabindex="0"
                                onclick={() => displayImage(sample.image_path_before, sample.image_path_after)}
                                onkeypress={(e) => { if (e.key === "Enter" || e.key === " ") displayImage(sample.image_path_before, sample.image_path_after); }}
                            >
                            {sample.image_path_after}
                            </span>
                        </p>
                    {/each}
                </div>
            </div>
        </div>
    {/if}

    {#if showImage}
        <div class="modal-overlay">
            <div class="modal image-modal">
                <button onclick={closeImage}>&times;</button>

                <div class="image-grid">
                    <div class="image-column">
                        <h3>Image before sampling</h3>
                        <img src={imageBeforeUrl} alt="Sample before" />
                    </div>

                    <div class="image-column">
                        <h3>Image after sampling</h3>
                        <img src={imageAfterUrl} alt="Sample after" />
                    </div>
                </div>
            </div>
        </div>
    {/if}
</div>