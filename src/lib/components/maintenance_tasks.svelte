<script lang="ts">
    import { listen } from '@tauri-apps/api/event';
</script>


<div class="container">
    <div class="grid-nest" style="grid-template-columns: 1fr; grid-template-rows: 1fr 8fr 1fr;">
        <div class="grid-item">
            <h1 class="heading"> Maintenance Panel Actions </h1>
        </div>

        <div class="grid-item">
            <div class="task-list">
                <div class="task-card">
                    Actions to do like this
                </div>
            </div>
        </div>

        <div class="grid-item" style="flex-direction: column;">
            <button class="button" style="align-self: flex-end;"> Request Actions Identification </button>
        </div>
    </div> 
</div>