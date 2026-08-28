<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import { confirm } from '@tauri-apps/plugin-dialog';

// ----- SVELTE -----
    import { onMount, onDestroy } from "svelte";
    import { get } from "svelte/store";
    import { initCameraHealthListener } from '../state.svelte.js';

// ----- STYLES -----
    import '$lib/css/global.css';
    import '$lib/css/navbar.css';
    import '$lib/css/components.css';
    import { samples } from "$lib/stores/samples";


    let { children } = $props();

// ----- NAVIGATION CONFIG ----- 
    const links = [
        { name: "Science", path: "/science" },
        { name: "Navigation", path: "/navigation" },
        { name: "Maintenance", path: "/maintenance" },
        { name: "Probing", path: "/probing" }
    ];
    
    let dropdownElTask: HTMLDivElement;

// ----- UI STATE -----
    let dropdownOpenTask = $state(false);
    let currentPage = $state("Task");


// ----- ROVER MODES -----
    let manualMode = $state(true);
    let pickupMode = $state(false);
    let braked = $state(false);


// ----- DROPDOWN LOGIC -----
    function toggleDropdownTask() {
        dropdownOpenTask = !dropdownOpenTask;
    }

    function handleClickOutsideTask(event: MouseEvent) {
        if (!dropdownElTask.contains(event.target as Node)) {
            dropdownOpenTask = false;
        }
    }


    $effect(() => {
        if (dropdownOpenTask) {
            document.addEventListener('click', handleClickOutsideTask);
        } else {
            document.removeEventListener('click', handleClickOutsideTask);
        }
    });


// ----- NAVIGATION AND ROVER MODES -----
    async function navigateTo(path: string) {
        await goto(path);
        currentPage = links.find(link => link.path === path)?.name || "Task";
        dropdownOpenTask = false;
    }
    async function getModes() {
        pickupMode = await invoke("get_state", { stateType: "Pickup" });
        manualMode = await invoke("get_state", { stateType: "Manual" });
        braked = await invoke("get_state", { stateType: "Braked" });
    }

    async function togglePickup() {
        pickupMode = !pickupMode;
        await invoke("set_state", {stateType: "Pickup", value: pickupMode}); 
    }
    async function toggleManual() {
        manualMode = !manualMode;
        await invoke("set_state", { stateType: "Manual", value: manualMode });
    }

    $effect(() => {
        const interval = setInterval(getModes, 250);
        return () => clearInterval(interval);
    });


// ===============================
// TASK FILE MANAGEMENT
// ===============================
    let taskFiles: string[] = [];

    async function listTaskFiles() {
        const result = await invoke<string[]>("list_task_files", {directory: "tasks"});
        if (result.length > 0) {
        taskFiles = result;
        } else {
        taskFiles = ["Nothing found"];
        }
    }

    function getNextTaskPrefix(taskName: string, taskFiles: string[]): string {
        const normalizedName = taskName.replace(" ", "_").toLowerCase();

        const matching = taskFiles
            .filter(file => file.endsWith(`_${normalizedName}.json`))
            .map(file => {
                const match = file.match(/^(\d{4})_/);
                return match ? parseInt(match[1], 10) : null;
            })
            .filter((n): n is number => n !== null);

        const nextNumber = matching.length > 0
            ? Math.max(...matching) + 1
            : 0;

        return nextNumber.toString().padStart(4, "0");
    }


// ===============================
// TIMER LOGIC
// ===============================
    let startTime = 0;
    let elapsed = $state(0);
    let running = $state(false);
    let rafId: number;
    let runningTask = $state("None");

    async function start() {
        startTime = performance.now() - elapsed;
        
        if (elapsed === 0 && get(samples).length > 0) {
            const keepSamples = await confirm(
                "You have unsaved samples. Would you like to keep them for the new task?",
                { title: "Unsaved Samples", kind: "warning" }
            );
            if (!keepSamples) {
                samples.set([]);
            }
        }
        running = true;
        loop();
    }

    function pause() {
        running = false;
        cancelAnimationFrame(rafId);
    }

    function loop() {
        if (!running) return;
        elapsed = performance.now() - startTime;
        rafId = requestAnimationFrame(loop);
    }

    async function reset() {
        if (elapsed!=0) {
            pause();

            const confirmed = await confirm(
            "Are you sure you want to end the current task?",
            { title: "End Task", kind: "warning" }
            );
            if (confirmed) {
                cancelAnimationFrame(rafId);

                await listTaskFiles();

                const prefix = getNextTaskPrefix(runningTask, taskFiles);
                const normalizedName = runningTask.replace(" ", "_").toLowerCase();
                const fileName = `${prefix}_${normalizedName}.json`;

                const encoder = new TextEncoder();
                const data = encoder.encode(
                    JSON.stringify({
                        task_name: runningTask,
                        task_number: prefix,
                        completion_time: elapsed >= 60000 ? `${Math.floor(elapsed / 60000)}m ${Math.floor((elapsed % 60000) / 1000)}s` : `${Math.floor(elapsed / 1000)}s`,
                        finished_at: new Date().toISOString(),
                        file_name: fileName,
                        attached_content: get(samples),
                    })
                );

                await invoke("save_task_file", {
                    directory: "tasks",
                    fileName: fileName,
                    data: data,
                });

                samples.set([]);
                elapsed = 0;
                runningTask = "None";
            } else {
                start(); // Resume if not confirmed
            }
        } else {
            return;
        }
    }

    function setTask() {
        if (elapsed > 0) return; // Don't change task if already running
        runningTask = links.find(link => link.path === window.location.pathname)?.name || "None";
    }



// ===============================
// LIFECYCLE
// ===============================
    
    
	onMount(async () => {
        initCameraHealthListener();
	});

    onDestroy(() => {
        cancelAnimationFrame(rafId);
    });

</script>

<!-- Navigation bar with dropdowns and control buttons -->
<nav class="navbar">
    <!-- Task dropdown menu -->
    <div class="dropdown" bind:this={dropdownElTask} class:show={dropdownOpenTask}>
        <button class="dropdown-button" onclick={toggleDropdownTask}>
            {currentPage}  ▼ 
        </button>
        <div class="dropdown-content">
            {#each links as link}
                <a href={link.path} onclick ={() => navigateTo(link.path)}>
                    {link.name}
                </a>
            {/each}
        </div>
    </div>

    <!-- Manual / Auto switch -->
    <label class="toggle-switch" title={manualMode ? "Manual" : "Automatic"}>
        <span class="toggle-label">{manualMode ? "Manual" : "Auto"}</span>
        <div class="toggle-track" class:active={!manualMode} onclick={toggleManual} role="switch" 
            aria-checked={!manualMode} tabindex="0" 
            onkeydown={(e) => e.key === 'Enter' && toggleManual()}>
            <div class="toggle-thumb"></div>
        </div>
    </label>

    <!-- Drive / Arm switch -->
    <label class="toggle-switch" title={pickupMode ? "Arm mode" : "Drive mode"}>
        <span class="toggle-label">{pickupMode ? "Arm" : "Drive"}</span>
        <div class="toggle-track" class:active={pickupMode} onclick={togglePickup} role="switch"
            aria-checked={pickupMode} tabindex="0"
            onkeydown={(e) => e.key === 'Enter' && togglePickup()}>
            <div class="toggle-thumb"></div>
        </div>
    </label>

    <!-- Start Task button -->
    <div>
        <button class="button" onclick={() => {running ? pause() : start(); setTask()}}>
            {running ? `❚❚ Pause ${runningTask}` : (elapsed > 0 ? `▶︎ Resume ${runningTask}` : '▶︎ Start Task')}
            <span style="margin-left: 0.5rem;">
                {elapsed >= 60000 ? `${Math.floor(elapsed / 60000)}m ${Math.floor((elapsed % 60000) / 1000)}s` : `${Math.floor(elapsed / 1000)}s`}
            </span>
        </button>
    </div>

    <!-- Centered icon -->
    <button class="center-icons" type="button" style="background: none; border: none; cursor: pointer;" onclick={togglePickup} aria-label="Toggle pickup mode">
        {#if pickupMode}
            <img src="/arm.svg" alt="Arm mode icon" class="mode-icon"/>
        {:else}
            <img src="/driving.svg" alt="Driving mode icon" class="mode-icon"/>
        {/if}
    </button>


    {#if braked}
        <div class="center-icons" style="margin-left: 4rem;"> 
            <img src="/parking.svg" alt="Parking icon" class="mode-icon"/>
        </div>
    {/if}

    <!-- Right-aligned icons and controls -->
    <div class="right-icons">
        <!-- Go on stage -->
        <button class="button" onclick={() => invoke("go_on_stage")}>
            Go on Stage
        </button>

        <!-- Stop going on stage -->
        <button class="button" onclick={() => invoke("stop_going_on_stage")}>
            Stop Going on Stage
        </button>
        <!-- Settings icon link -->
        <div class="icon-link">
            <a href="/settings" onclick ={() => navigateTo('/settings')}>
                <img src="/settings.svg" alt="Settings" class="nav-icon"/>
            </a>
        </div>

        <!-- Home icon link -->
        <div class="icon-link">
            <a href="/" onclick ={() => navigateTo('/')}>
                <img src="/home.svg" alt="Home" class="nav-icon"/>
            </a>
        </div>

        <!-- Stop button -->
        <div>
            <button class="button" onclick={() => reset()} style="background-color: #2C2C2C">
                END TASK
            </button>
        </div>
    </div>
</nav>


{@render children()}