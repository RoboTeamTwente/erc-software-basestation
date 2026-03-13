<script lang="ts">
    let { checked = $bindable(), label, value, title, onclick } = $props();
</script>

<div class="task-info" style="align-items: flex-start; justify-content:flex-start; padding-left: 4rem">
    <label class="custom-checkbox">
        <input type="checkbox" bind:checked />
        <span></span>
    </label>
    <span><strong>{label}: </strong></span>
    <span>{value ?? ""}</span>
    <button class="delete-button plus-button" {title} {onclick}>+</button>
</div>