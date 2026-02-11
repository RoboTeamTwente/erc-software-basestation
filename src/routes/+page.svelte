<script lang="ts">
  import Double_Video from '$lib/components/double_video.svelte';
  import Map from '$lib/components/map.svelte';
  import NavigationPlan from '$lib/components/navigation_plan.svelte';
  import TaskCompletion from '$lib/components/task_completion.svelte';

</script>

<div class="grid">

  <div class="grid-item" style="padding-right: 0">
    <Double_Video port1={"5000"} port2={"5001"} />
  </div>


  <div class="grid-nest" style="grid-template-columns: 1fr 2fr; grid-template-rows: 2fr 3fr;">

    <div class="grid-item" style="padding-left: 0">
      <div class="attached-container">
        <div class="button" style="margin: 10px">
          Go to Pick-up Mode
        </div>
      </div>
    </div>

    <div class="grid-item">
      <div class="container">
        <h1 class="heading">Short Instructions for controls</h1>
      </div>
    </div>

    <div class="grid-item">
      <div class="container">
        <h1 class="heading">Model</h1>
      </div>
    </div>

    <div class="grid-item">
      <TaskCompletion />
    </div>

  </div>


  <div class="grid-item" style="padding-right: 0">
    <Map />
  </div>


  <div class="grid-nest" style="grid-template-columns: 2fr 3fr;">

    <div class="grid-item" style="padding-left: 0">
      <NavigationPlan style="width: 100%; height: 100%;" />
    </div>

    <div class="grid-item">
      <div class="container">
        <h1 class="heading">IMU and Sensor Data</h1> 
      </div>
    </div>

  </div>


</div>

