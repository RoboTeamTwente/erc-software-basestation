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
    async function startDummyStream() {
        await invoke("start_dummy_imu_stream");
    }
    async function stopDummyStream() {
        await invoke("stop_dummy_imu_stream");
    }
    async function pingGPS() {
        try {
            await invoke("send_ping_cmd", { packetType: 'gps' });
        } catch (e) {
            console.error("pingGPS failed:", e);
        }
    }
    async function pingPh() {
        try {
            await invoke("send_ping_cmd", { packetType: 'ph' });
        } catch (e) {
            console.error("pingPh failed:", e);
        }
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
</script>

<div>
    <button class="button" style="margin: 10px;" onclick={ping}>
        Ping Rust
    </button>

    <button class="button" style="margin: 10px;" onclick={clearCache}>
        Clear cache
    </button>

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

    <button class="button" style="margin: 10px;" onclick={() => pingUdp()}>
        Ping UDP
    </button>

    <button class="button" style="margin: 10px;" onclick={() => pingGPS()}>
        Ping GPS
    </button>

    <button class="button" style="margin: 10px;" onclick={() => pingPh()}>
        Ping PH
    </button>

    <button class="button" style="margin: 10px;" onclick={() => startDummyStream()}>
        Start dummy IMU stream
    </button>

    <button class="button" style="margin: 10px;" onclick={() => stopDummyStream()}>
        Stop dummy IMU stream
    </button>

    <button class="button" style="margin: 10px;" onclick={() => startGenDummy()}>
        Start dummy general stream
    </button>

    <button class="button" style="margin: 10px;" onclick={() => stopGenDummy()}>
        Stop dummy general stream
    </button>

    <button class="button" style="margin: 10px" onclick={() => whereIsTheModel()}>
        Where is the model
    </button>


    <button class="button" onclick={getIP}>
        Check my IP
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

    
    <div>
        {#if taskFiles.length > 0}
        <div style="margin: 10px;">
            <h3>Task Files:</h3>
            <ul>
                {#each taskFiles as file}
                <li>
                    <button
                    class="link-button"
                    onclick={() => openTaskFile(file)}
                    >
                    {file}
                    </button>
                </li>
                {/each}
            </ul>
        </div>
        {/if}
    </div>

    {#if selectedFile}
        <div style="margin: 10px;">
        <h3>Contents of {selectedFile}</h3>
        <pre>{fileContents}</pre>
        </div>
    {/if}

</div>