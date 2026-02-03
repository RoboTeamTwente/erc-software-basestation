<script lang="ts">
    export let style: string = "";
</script>

<div class="attached-container">
    <div class="grid-nest" style="grid-template-rows: 1fr 8fr 2fr {style}">
        <div class="grid-item">
            Navigation plan
        </div>

        <div class="grid-item">
            <div class="grid-nest" style="grid-template-rows: repeat(6, 1fr){style}">
                <div class="start">
                    Starting Point
                </div>
                <div class="waypoint">
                    Waypoint 1
                </div>
                <div class="waypoint">
                    Waypoint 2
                </div>
                <div class="waypoint">
                    Waypoint 3
                </div>
                <div class="waypoint">
                    Waypoint 4
                </div>
                <div class="start">
                    End Point
                </div>
            </div>
        </div>

        <div class="grid-item">
            <button class="button">
                + Add Map File
            </button>
            <button class="button">
                Plan Route
            </button>
        </div>
    </div>
</div>


<style>
    .start {
        background-color: white;
        border-radius: 16px;
        display: flex;
        padding: 0.7rem;
        margin: 0.25rem;
        font-size: 1rem;
        font-weight: bold;
        height: fit-content;
        overflow: hidden;
    }

    .waypoint {
        background-color: white;
        border-radius: 16px;
        display: flex;
        padding: 0.5rem;
        margin: 0.25rem;
        font-size: 1rem;
        width: 80%;
        height: fit-content;
        overflow: hidden;
    }
</style>
