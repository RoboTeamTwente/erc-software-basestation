<script lang="ts">
// ----- TAURI / EXTERNAL -----
    import { invoke } from "@tauri-apps/api/core";
    import { confirm } from '@tauri-apps/plugin-dialog';

// ----- SVELTE -----
    import { onMount } from 'svelte';

// ----- STYLES -----
    import '../../global.css';


// ----- TYPES -----
    type Task = {
        task_name: string;
        task_number: string;
        completion_time: string;
        finished_at: string;
        attached_content: string;
        file_name: string;
    }


// ----- STATE -----
    let taskFiles = $state<string[]>([]);
    let tasks = $state<Task[]>([]);
    let selectedTask = $state<Task | null>(null);


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
    <div class="grid-nest" style="grid-template-rows: 1fr 5fr">

        <div class="grid-item">
            <h1 class="heading">Task Completion Overview

            </h1>
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
                        <div class="task-card" onclick={() => selectTask(task)} role="button" tabindex="0">
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
            </div>
        </div>
    {/if}
</div>