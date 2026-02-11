<script lang="ts">
    import { onMount } from 'svelte';
    import '../../global.css';
    import { invoke } from "@tauri-apps/api/core";

    type Task = {
        task_name: string;
        task_number: string;
        completion_time: string;
        finished_at: string;
        attached_content: string;
    }

    let taskFiles = $state<string[]>([]);
    let tasks = $state<Task[]>([]);
    let selectedTask = $state<Task | null>(null);

	onMount(() => {
		listFiles();
	});

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

    function selectTask(task: Task) {
        selectedTask = task;
    }

    function closeModal() {
        selectedTask = null;
    }

    function formatDate(dateStr: string) {
        const date = new Date(dateStr);
        return date.toLocaleString();
    }
</script>


<div class="container">
    <h1 class="heading">Task Completion Overview</h1>

    {#if tasks.length === 0}
        <p>No tasks found.</p>
    {:else}
        <div class="task-list">
            {#each tasks as task}
                <button class="task-card" onclick={() => selectTask(task)}>
                    <div class="task-info">
                        <span><strong>{task.task_name}</strong></span>
                        <span>#{task.task_number}</span>
                        <span>Time: {task.completion_time}</span>
                        <span>Finished: {formatDate(task.finished_at)}</span>
                    </div>
                </button>
            {/each}
        </div>
    {/if}

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


<style>
    .task-list {
        max-height: 300px;
        overflow-y: auto;        width: fit-content;
        border-radius: 8px;
        padding: 0.5rem;
    }

    .task-card {
        background-color: #fff;
        padding: 0.5rem 1rem;
        margin: 0.5rem;
        font-size: 1rem;
        border: none;
        margin: 0.4rem;
        width: 95%;
        border-radius: 12px;
        cursor: pointer;
        height: fit-content;
    }

    .task-card:hover {
        background-color: #e6f0ff;
    }

    .task-info {
        display: flex;
        gap: 1rem;
        flex-wrap: nowrap;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    /* Modal Styles */
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0,0,0,0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
    }

    .modal {
        background-color: #fff;
        padding: 2rem;
        border-radius: 10px;
        max-width: 400px;
        width: 90%;
        box-shadow: 0 5px 15px rgba(0,0,0,0.3);
        position: relative;
    }

    .modal h2 {
        margin-top: 0;
        margin-bottom: 1rem;
    }

    .modal button {
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        background: none;
        border: none;
        font-size: 1.2rem;
        cursor: pointer;
    }
</style>