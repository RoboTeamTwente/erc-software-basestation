<script lang="ts">
// ----- EXTERNAL / TAURI -----
    import { confirm } from '@tauri-apps/plugin-dialog';
    import { invoke } from "@tauri-apps/api/core";


// ----- STATE -----
    let taskFiles: string[] = [];
    let selectedFile: string | null = null;
    let fileContents = ""; 


// ----- UTILITIES -----
    async function ping() {
        await invoke("ping");
    }
    async function clearCache(){
        await invoke("clear_cache");
    }
    async function pingUdp() {
        await invoke("send_ping_cmd");
    }

    async function startGenDummy(){
        try {
            await invoke("start_dummy_streams");
        } catch (e) {
            console.error("startGenDummy failed:", e);
        }
    }
    async function stopGenDummy(){
        try {
            await invoke("stop_dummy_streams");
        } catch (e) {
            console.error("stopGenDummy failed:", e);
        }
    }
    async function startObjectDummy(){
        try {
            await invoke("start_detection_sim");
        } catch (e) {
            console.error("startObjectDummy failed:", e);
        }
    }

    async function whereIsTheModel() {
        const debug = await invoke('debug_resource_dir')
        console.log(debug)
    }


// ----- FILE MANAGEMENT -----
    async function listFiles(directory: string) {
        const result = await invoke<string[]>("list_task_files", {directory});
        if (result.length > 0) {
            taskFiles = result;
        } else {
            taskFiles = ["Nothing found"];
        }
    }

    async function clearAllFiles(directory: string) {
        const confirmed = await confirm(
        "Are you sure you want to delete all images?",
        { title: "Delete", kind: "warning" }
        );

        if (confirmed) {
            await invoke("delete_all_task_files", {directory});
                taskFiles = [];
                selectedFile = null;
                fileContents = "";
            }
    }

    async function openTaskFile(file: string) {
        try {
            const data = await invoke<number[]>("read_task_file", {
                fileName: file
            });

            const uint8Array = new Uint8Array(data);
            const decoder = new TextDecoder("utf-8");

            fileContents = decoder.decode(uint8Array);
            selectedFile = file;
        } catch (e) {
            console.error("Failed to read file:", e);
        }
    }


// ----- SNAPSHOT -----
    async function saveSnapshot() {
        await invoke("save_snapshot", {port:"5000", fileName: "test"});
    }


// ----- CHECK MY IP -----
    let ip = '';
    let loading = false;
    let error = '';

    async function getIP() {
        loading = true;
        error = '';
        ip = '';

        try {
        const res = await fetch('https://api.ipify.org?format=json');
        const data = await res.json();
        ip = data.ip;
        } catch (e) {
        error = 'Failed to fetch IP';
        } finally {
        loading = false;
        }
    }

// ----- CHANGE DESTINATION IP -----
    let roverAddress = '';
    let roverAddressStatus: 'idle' | 'saved' | 'error' = 'idle';
    let roverAddressError = '';

    async function loadRoverAddress() {
        try {
            roverAddress = await invoke<string>('get_rover_address');
        } catch (e) {
            console.error('Failed to load rover address:', e);
        }
    }

    async function changeRoverIP() {
        try {
            await invoke('set_rover_address', { address: roverAddress });
            roverAddressStatus = 'saved';
            setTimeout(() => roverAddressStatus = 'idle', 2000);
        } catch (e) {
            roverAddressError = e as string;
            roverAddressStatus = 'error';
        }
    }

    loadRoverAddress();
</script>

<div class="grid">
    <div class="grid-item">
        <div class="container">

            <div class="grid-nest" style=" grid-template-columns: 1fr 1fr">


                <div class="grid-item" style="flex-direction: column;">
                    <h1 class="heading"> <span> File management </span> </h1>

                    <button class="button" style="margin: 10px;" onclick={() => listFiles("images")}>
                        List image files
                    </button>

                    <button class="button" style="margin: 10px;" onclick={() => saveSnapshot()}>
                        Save an image file
                    </button>

                    <button class="button" style="margin: 10px;" onclick={() => clearAllFiles("tasks")}>
                        Delete all task files
                    </button>

                    <button class="button" style="margin: 10px;" onclick={() => clearAllFiles("images")}>
                        Delete all image files
                    </button>

                    <button class="button" style="margin: 10px;" onclick={() => clearAllFiles("maps")}>
                        Delete all map files
                    </button>
                </div>


                <div class="grid-item" style="flex-direction: column;">
                    <div class="task-list">
                        {#if taskFiles.length > 0}
                            <h1 class="heading">Image Files</h1>

                                {#each taskFiles as file}
                                    <div class="task-card">
                                        {file}
                                    </div>
                                {/each}

                        {/if}
                    </div>

                    {#if selectedFile}
                        <div style="margin: 10px;">
                        <h3>Contents of {selectedFile}</h3>
                        <pre>{fileContents}</pre>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>



    <div class="grid-item" style="flex-direction: column;">
        <div class="container">

            <h1 class="heading"> <span> Checks </span> </h1>

            <button class="button" style="margin: 10px;" onclick={ping}>
                Ping Rust
            </button>

            <button class="button" style="margin: 10px;" onclick={() => pingUdp()}>
                Ping UDP
            </button>

        </div>
    </div>


    <div class="grid-item" style="flex-direction: column;">
        <div class="container">
            <h1 class="heading"> <span> My Info </span> </h1>

            <button class="button" style="margin: 10px" onclick={() => whereIsTheModel()}>
                Where is the model
            </button>

            <button class="button" onclick={getIP}>
                What is my IP
            </button>

            {#if loading}
            <p>Loading...</p>
            {/if}

            {#if ip}
            <p>Your IP: {ip}</p>
            {/if}

            {#if error}
            <p style="color:red">{error}</p>
            {/if}

        </div>
    </div>


    <div class="grid-item" style="flex-direction: column;">
        <div class="container">
            <h1 class="heading"> <span> Dummy Data</span> </h1>

            <button class="button" style="margin: 10px;" onclick={() => startGenDummy()}>
                Start dummy general stream
            </button>

            <button class="button" style="margin: 10px;" onclick={() => stopGenDummy()}>
                Stop dummy general stream
            </button>

            <button class="button" style="margin: 10px;" onclick={() => startObjectDummy()}>
                Start dummy object stream
            </button>

        </div>
    </div>


    <div class="grid-item" style="flex-direction: column;">
        <div class="container">
            <h1 class="heading"> <span> Rover Connection </span> </h1>

            <div style="display: flex; align-items: center; gap: 10px; margin: 10px;">
                <input
                    type="text"
                    placeholder="192.168.1.10:9000"
                    bind:value={roverAddress}
                    style="flex: 1; padding: 6px 10px; font-size: 14px;"
                />
                <button class="button" onclick={changeRoverIP}>
                    Save
                </button>
            </div>

            {#if roverAddressStatus === 'saved'}
                <p style="color: green; margin: 0 10px;">✓ Address saved</p>
            {/if}

            {#if roverAddressStatus === 'error'}
                <p style="color: red; margin: 0 10px;">✗ {roverAddressError}</p>
            {/if}
        </div>
    </div>

    <div class="grid-item" style="flex-direction: column;">
        <div class="container">
            <h1 class="heading"> <span> Cache </span> </h1>

            <button class="button" style="margin: 10px;" onclick={clearCache}>
                Clear cache
            </button>
        </div>
    </div>

</div>