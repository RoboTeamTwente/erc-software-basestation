<script lang="ts">
    
let modalOpen = $state(false);
</script>


<div class="container" style="flex-direction: row; align-items: center; justify-content: space-between;">
    <h1 class="heading">Short Instructions for controls</h1>
    <button class="button" style="margin-right: 2rem;" onclick={() => modalOpen = true}>
        Open Instructions
    </button>
</div>

{#if modalOpen}
    <div class="modal-overlay">
        <div class="modal">
            <button class="close-button" onclick={() => modalOpen = false}>&times;</button>
            <h1 class="heading">Instructions</h1>
            <p>Use the controls to operate the robot. Here are some basic instructions:</p>

            <div class="grid-nest" style="grid-template-columns: 1fr 2fr; gap: 1rem;">
                <div class="grid-item" style="flex-direction: column;">

                    <h2 class="subheading">General controls</h2>
                    <ul>                       
                        <li><strong>- Button</strong> Toggle current state (automatic/manual)</li>
                        <li><strong>+ Button</strong> Toggle between drive and pickup mode</li>
                    </ul>

                    <h2 class="subheading">Driving Mode</h2>
                    <ul>
                        <li><strong>L-Stick</strong> Driving forwards/backwards and turning left/right</li>
                        <li><strong>ZL</strong> Engage/disengage hand brake</li>
                        <li><strong>ZR</strong> Engage temporary brake </li>
                    </ul>

                    <h2 class="subheading">Pickup Mode</h2>
                    <ul>
                        <li><strong>L-Stick</strong> Wrist flick and roll</li>
                        <li><strong>R-Stick</strong> Arm XY-plane</li>
                        <li><strong>DPad</strong> Arm Z-axis (up/down)</li>
                        <li><strong>R</strong> Open gripper</li>
                        <li><strong>L</strong> Close gripper</li>
                    </ul>
                </div>
                <div class="grid-item" style="display: flex; align-items: center; justify-content: center;">
                    <img src="/controllerSchema.svg" alt="Controller Schema" style="height: 80%; width: 80%;"/>
                </div>
            </div>
        </div>
    </div>
{/if}