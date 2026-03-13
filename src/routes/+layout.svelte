<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import { confirm } from '@tauri-apps/plugin-dialog';
    import { listen } from '@tauri-apps/api/event';

// ----- SVELTE -----
    import { onMount, onDestroy } from "svelte";
    import { get } from "svelte/store";
    import { initCameraHealthListener } from '../state.svelte.js';

// ----- STYLES -----
    import '../global.css';
    import '../navbar.css';
    import '../components.css';
    import { samples } from "../stores/samples";


    let { children } = $props();

// ----- NAVIGATION CONFIG ----- 
    const links = [
        { name: "Science", path: "/science" },
        { name: "Navigation", path: "/navigation" },
        { name: "Maintenance", path: "/maintenance" },
        { name: "Probing", path: "/probing" }
    ];
    

// ----- UI STATE -----
    let dropdownOpenTask = $state(false);
    let dropdownOpenDrive = $state(false);
    let dropdownOpenArm = $state(false);
    let currentPage = $state("Task");


// ----- ROVER MODES -----
    let manualDriveMode = $state(true);
    let manualArmMode = $state(true);
    let driveControlMode = $state("Manual drive");
    let armControlMode = $state("Manual arm");
    let pickupMode = $state(false);


// ----- INPUT STATE -----
    let pressed = new Set();


// ----- DROPDOWN LOGIC -----
    function toggleDropdownTask() {
        dropdownOpenTask = !dropdownOpenTask;
    }
    function toggleDropdownDrive() {
        dropdownOpenDrive = !dropdownOpenDrive;
    }
    function toggleDropdownArm() {
        dropdownOpenArm = !dropdownOpenArm;
    }

    function handleClickOutsideTask(event: MouseEvent) {
        if (!dropdownElTask.contains(event.target as Node)) {
            dropdownOpenTask = false;
        }
    }
    function handleClickOutsideDrive(event: MouseEvent) {
        if (!dropdownElDrive.contains(event.target as Node)) {
            dropdownOpenDrive = false;
        }
    }
    function handleClickOutsideArm(event: MouseEvent) {
        if (!dropdownElArm.contains(event.target as Node)) {
            dropdownOpenArm = false;
        }
    }

    let dropdownElTask: HTMLDivElement;
    let dropdownElDrive: HTMLDivElement;
    let dropdownElArm: HTMLDivElement;

    $effect(() => {
        if (dropdownOpenTask) {
            document.addEventListener('click', handleClickOutsideTask);
        } else {
            document.removeEventListener('click', handleClickOutsideTask);
        }
    });
    $effect(() => {
        if (dropdownOpenDrive) {
            document.addEventListener('click', handleClickOutsideDrive);
        } else {
            document.removeEventListener('click', handleClickOutsideDrive);
        }
    });
    $effect(() => {
        if (dropdownOpenArm) {
            document.addEventListener('click', handleClickOutsideArm);
        } else {
            document.removeEventListener('click', handleClickOutsideArm);
        }
    });


// ----- NAVIGATION AND ROVER MODES -----
    function navigateTo(path: string) {
        goto(path);
        currentPage = links.find(link => link.path === path)?.name || "Task";
        dropdownOpenTask = false;
    }
    async function toggleDriveControlMode(){
        dropdownOpenDrive = false;
        if (manualDriveMode){
            driveControlMode = "Manual drive";
        } else {
            driveControlMode = "Automatic drive";
        }
        await invoke("set_state", {stateType: "DriveManual", value: manualDriveMode});
    }
    async function toggleArmControlMode(){
        dropdownOpenArm = false;
        if (manualArmMode){
            armControlMode = "Manual arm";
        } else {
            armControlMode = "Automatic arm";
        }
        await invoke("set_state", {stateType: "ArmManual", value: manualArmMode});
    }
    async function getPickupMode() {
        pickupMode = await invoke("get_state", {stateType: "Pickup"})
    }

    $effect(() => {
        const interval = setInterval(getPickupMode, 250);
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

    function start() {
        startTime = performance.now() - elapsed;
        running = true;
        samples.set([]);
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
// KEYBOARD INPUT
// ===============================
	function handleKeyDown(e: KeyboardEvent) {
		if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.code)) {
			e.preventDefault(); // stop scrolling
			pressed.add(e.code);
			sendCommand();
		}
    }

	function handleKeyUp(e: KeyboardEvent) {
		pressed.delete(e.code);
		sendCommand();
	}

	async function sendCommand() {
		// Example movement logic
		const command = {
			up: pressed.has("ArrowUp"),
			down: pressed.has("ArrowDown"),
			left: pressed.has("ArrowLeft"),
			right: pressed.has("ArrowRight")
		};
        await invoke("pressed_key", {command});
	}


// ===============================
// LIFECYCLE
// ===============================
    initCameraHealthListener();
    
	onMount(async () => {
		window.addEventListener("keydown", handleKeyDown);
		window.addEventListener("keyup", handleKeyUp);
	});

    onDestroy(() => {
        cancelAnimationFrame(rafId);
		window.removeEventListener("keydown", handleKeyDown);
		window.removeEventListener("keyup", handleKeyUp);
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

    <!-- Drive Control Mode dropdown menu -->
    <div class="dropdown" bind:this={dropdownElDrive} class:show={dropdownOpenDrive}>
        <button class="dropdown-button" onclick={toggleDropdownDrive}>
            {driveControlMode} ▼ 
        </button>
        <div class="dropdown-content">
            <a href="#" onclick={() => { manualDriveMode = true ; toggleDriveControlMode()}}>
                Manual
            </a>
            <a href="#" onclick={() => { manualDriveMode = false ; toggleDriveControlMode()}}>
                Automatic
            </a>
        </div>
    </div>

    <!-- Arm Control Mode dropdown menu -->
    <div class="dropdown" bind:this={dropdownElArm} class:show={dropdownOpenArm}>
        <button class="dropdown-button" onclick={toggleDropdownArm}>
            {armControlMode} ▼ 
        </button>
        <div class="dropdown-content">
            <a href="#" onclick={() => { manualArmMode = true ; toggleArmControlMode()}}>
                Manual
            </a>
            <a href="#" onclick={() => { manualArmMode = false ; toggleArmControlMode()}}>
                Automatic
            </a>
        </div>
    </div>

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
    <div class="center-icons">
        {#if pickupMode}
            <img src="/arm.svg" alt="Arm mode icon" class="mode-icon"/>
        {:else}
            <img src="/driving.svg" alt="Driving mode icon" class="mode-icon"/>
        {/if}
    </div>

    <!-- Right-aligned icons and controls -->
    <div class="right-icons">
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