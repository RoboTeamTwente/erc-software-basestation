<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { armData, armFeedback } from '$lib/stores/arm';
  import { ArmBoardMovementFeedback_ArmError, ArmBoardMovementFeedback } from '$lib/proto/components/arm_board/movement_software_feedback';
  import '$lib/css/arm.css';

  let unlisten: (() => void) | undefined;

  onMount(async () => {
    unlisten = await listen<{ arm_error: number }>('arm-feedback-update', (event) => {
      armFeedback.set(ArmBoardMovementFeedback.fromJSON(event.payload));
    });
  });

  onDestroy(() => unlisten?.());

  // ── Error display helpers ─────────────────────────────────────────────────
  type Severity = 'ok' | 'warn' | 'error';

  const ERROR_META: Record<ArmBoardMovementFeedback_ArmError, { label: string; severity: Severity }> = {
    [ArmBoardMovementFeedback_ArmError.ALL_OK]:           { label: 'All OK',           severity: 'ok'    },
    [ArmBoardMovementFeedback_ArmError.OBSTRUCTION]:      { label: 'Obstruction',       severity: 'warn'  },
    [ArmBoardMovementFeedback_ArmError.CALIBRATION]:      { label: 'Calibrating…',      severity: 'warn'  },
    [ArmBoardMovementFeedback_ArmError.POINT_NOT_IN_RANGE]:{ label: 'Out of Range',     severity: 'error' },
    [ArmBoardMovementFeedback_ArmError.MOTOR_MALFUNCTION]:{ label: 'Motor Malfunction', severity: 'error' },
    [ArmBoardMovementFeedback_ArmError.UNRECOGNIZED]:     { label: 'Unknown Error',     severity: 'error' },
  };

  $: meta = ERROR_META[$armFeedback.arm_error ?? ArmBoardMovementFeedback_ArmError.ALL_OK]
            ?? ERROR_META[ArmBoardMovementFeedback_ArmError.UNRECOGNIZED];

  // ── Gauge arc helpers (40×40 viewBox, centred at 20,21) ──────────────────
  const R=15, GCX=20, GCY=21, SD=-225, SW=270;
  function gpt(d: number) {
    const rad = d * Math.PI / 180;
    return { x: GCX + R * Math.cos(rad), y: GCY + R * Math.sin(rad) };
  }
  function gtrack() {
    const s=gpt(SD), e=gpt(SD+SW);
    return `M ${s.x} ${s.y} A ${R} ${R} 0 1 1 ${e.x} ${e.y}`;
  }
  function gfill(val: number, min: number, max: number) {
    const pct = Math.min(1, Math.max(0, (val - min) / (max - min)));
    if (pct <= 0) return '';
    const sweep = SW * pct;
    const s = gpt(SD), e = gpt(SD + sweep);
    return `M ${s.x} ${s.y} A ${R} ${R} 0 ${sweep > 180 ? 1 : 0} 1 ${e.x} ${e.y}`;
  }

  const joints = [
    { key: 'base_actual_position',             label: 'Base',   min: -90,  max: 90  },
    { key: 'stepper_top_actual_position',      label: 'Upper',  min: -60,  max: 60  },
    { key: 'stepper_bottom_actual_position',   label: 'Lower',  min: -60,  max: 60  },
    { key: 'gripper_rotation_actual_position', label: 'Rotate', min: -180, max: 180 },
    { key: 'gripper_pitch_actual_position',    label: 'Pitch',  min: -90,  max: 90  },
    { key: 'jaw_actual_position',              label: 'Jaw',    min: 0,    max: 45  },
  ] as const;
</script>

<div class="arm-gauges">
  <span class="heading">Joint Angles</span>

  <!-- ── Movement feedback status banner ── -->
  <div class="arm-status" class:ok={meta.severity === 'ok'} class:warn={meta.severity === 'warn'} class:error={meta.severity === 'error'}>
    <span class="status-pip"></span>
    <span class="status-label">{meta.label}</span>
  </div>

  <div class="gauges-grid">
    {#each joints as j}
      {@const val = ($armData[j.key] ?? 0)}
      <div class="gauge-cell">
        <svg viewBox="0 0 40 40" class="gsv">
          <path d={gtrack()} class="g-track"/>
          {#if gfill(val, j.min, j.max)}
            <path d={gfill(val, j.min, j.max)} class="g-fill"/>
          {/if}
          <text x="20" y="23" class="g-val">{val.toFixed(0)}°</text>
        </svg>
        <span class="g-label">{j.label}</span>
      </div>
    {/each}
  </div>
</div>
