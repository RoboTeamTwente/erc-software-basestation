<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import { confirm } from '@tauri-apps/plugin-dialog';

// ----- SVELTE -----
    import { onMount, onDestroy } from "svelte";


// ----- STYLES -----
    import '../global.css';


    let { children } = $props();

// ----- NAVIGATION CONFIG -----
    const links = [
        { name: "Science", path: "/science" },
        { name: "Navigation", path: "/navigation" },
        { name: "Maintenance", path: "/maintenance" },
        { name: "Probing", path: "/probing" }
    ];
    

// ----- UI STATE -----
    let dropdownOpen = $state(false);
    let dropdownOpen2 = $state(false);
    let currentPage = $state("Task");


// ----- ROVER MODES -----
    let manualMode = $state(true);
    let controlMode = $state("Manual")


// ----- INPUT STATE -----
    let pressed = new Set();


// ----- DROPDOWN LOGIC -----
    function toggleDropdown() {
        dropdownOpen = !dropdownOpen;
    }
    function toggleDropdown2() {
        dropdownOpen2 = !dropdownOpen2;
    }

    function handleClickOutside(event: MouseEvent) {
        if (!dropdownEl.contains(event.target as Node)) {
            dropdownOpen = false;
        }
    }
    function handleClickOutside2(event: MouseEvent) {
        if (!dropdownEl2.contains(event.target as Node)) {
            dropdownOpen2 = false;
        }
    }

    let dropdownEl: HTMLDivElement;
    let dropdownEl2: HTMLDivElement;

    $effect(() => {
        if (dropdownOpen) {
            document.addEventListener('click', handleClickOutside);
        } else {
            document.removeEventListener('click', handleClickOutside);
        }
    });
    $effect(() => {
        if (dropdownOpen2) {
            document.addEventListener('click', handleClickOutside2);
        } else {
            document.removeEventListener('click', handleClickOutside2);
        }
    });


// ----- NAVIGATION AND ROVER MODES -----
    function navigateTo(path: string) {
        goto(path);
        currentPage = links.find(link => link.path === path)?.name || "Task";
        dropdownOpen = false;
    }
    function toggleControlMode(){
        dropdownOpen2 = false;
        if (manualMode){
            controlMode = "Manual";
        } else {
            controlMode = "Automatic"
        }
    }


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
                        attached_content: "",
                    })
                );

                await invoke("save_task_file", {
                    directory: "tasks",
                    fileName: fileName,
                    data: data,
                });

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
	onMount(() => {
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
    <div class="dropdown" bind:this={dropdownEl} class:show={dropdownOpen}>
        <button class="dropdown-button" onclick={toggleDropdown}>
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

    <!-- Control Mode dropdown menu -->
    <div class="dropdown" bind:this={dropdownEl2} class:show={dropdownOpen2}>
        <button class="dropdown-button" onclick={toggleDropdown2}>
            {controlMode} ▼ 
        </button>
        <div class="dropdown-content">
            <a href="#" onclick={() => { manualMode = true ; toggleControlMode()}}>
                Manual
            </a>
            <a href="#" onclick={() => { manualMode = false ; toggleControlMode()}}>
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

    <!-- Right-aligned icons and controls -->
    <div class="right-icons">
        <!-- Settings icon link -->
        <div class="icon-link">
            <a href="/settings" onclick ={() => navigateTo('/settings')}>
                <img src="/settings.svg" alt="Settings" class="icon"/>
            </a>
        </div>

        <!-- Home icon link -->
        <div class="icon-link">
            <a href="/" onclick ={() => navigateTo('/')}>
                <img src="/home.svg" alt="Home" class="icon"/>
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


<style>
    /* Main navigation bar container */
    .navbar {
        display: flex;
        justify-content: flex-start;
        align-items: center;
        background: #D9D9D9;
        padding: 0.75rem 1.5rem;
        color: white;
        gap: 1rem;
        column-gap: 2%;
    }

    /* Dropdown wrapper for positioning */
    .dropdown {
        position: relative;
        display: inline-block;
    }

    /* Dropdown button styling */
    .dropdown-button {
        background-color: #5A1C74;
        color: white;
        padding: 0.5rem 1rem;
        font-size: 1rem;
        border: none;
        border-radius: 12px;
        cursor: pointer;
    }

    .dropdown-button:hover{
        background-color: #401453;
    }

    /* Dropdown content container (hidden by default) */
    .dropdown-content {
        display: none;
        position: absolute;
        background-color: #f9f9f9;
        min-width: 160px;
        box-shadow: 0px 8px 16px 0px rgba(0,0,0,0.2);
        z-index: 2;
    }

    /* Dropdown menu item styling */
    .dropdown-content a {
        color: black;
        padding: 12px 16px;
        text-decoration: none;
        display: block;
    }

    /* Dropdown menu item hover effect */
    .dropdown-content a:hover {
        background-color: #f1f1f1;
        z-index: 2;
    }

    /* Show dropdown content when dropdown has 'show' class */
    .dropdown.show .dropdown-content {
        display: block;
    }

    /* Right aligned icons */
    .right-icons {
        margin-left: auto;
        display: flex;
        gap: 2rem;
        align-items: center;
    }

    /* Icon link container */
    .icon-link {
        margin-left: 5;
    }

    /* Icon image sizing */
    .icon {
        height: 36px;
    }

</style>

{@render children()}