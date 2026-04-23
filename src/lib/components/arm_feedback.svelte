<script lang="ts">
  import { armData } from '$lib/state/arm';

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
    { key: 'base_actual_position',             label: 'Base',    min: -90,  max: 90  },
    { key: 'stepper_top_actual_position',      label: 'Upper',   min: -60,  max: 60  },
    { key: 'stepper_bottom_actual_position',   label: 'Lower',   min: -60,  max: 60  },
    { key: 'gripper_rotation_actual_position', label: 'Rotate',  min: -180, max: 180 },
    { key: 'gripper_pitch_actual_position',    label: 'Pitch',   min: -90,  max: 90  },
    { key: 'jaw_actual_position',              label: 'Jaw',     min: 0,    max: 45  },
  ] as const;
</script>

<div class="arm-gauges">
  <span class="heading">Joint Angles</span>
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

<style>
  .arm-gauges {
    background: var(--color-light-gray);
    border: 1px solid var(--color-border-gray);
    border-radius: 8px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    /* Fill cell, never impose size */
    width: 100%;
    height: 100%;
    min-height: 0;
    box-sizing: border-box;
    overflow: hidden;
  }

  .gauges-title {
    font-size: 13px;
    font-weight: 600;
    color: #1a1a1a;
    padding-bottom: 7px;
    border-bottom: 1px solid var(--color-border-gray);
    flex-shrink: 0;
  }

  .gauges-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    /* rows grow equally to fill remaining height */
    grid-template-rows: repeat(2, 1fr);
    gap: 6px;
    flex: 1 1 0;
    min-height: 0;
  }

  .gauge-cell {
    background: var(--color-offwhite);
    border: 1px solid var(--color-border-gray);
    border-radius: 6px;
    padding: 4px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-height: 0;
    overflow: hidden;
  }

  /* SVG scales to whatever the cell gives it */
  .gsv {
    width: 100%;
    /* aspect-ratio keeps it square without imposing a pixel height */
    aspect-ratio: 1;
    max-width: 56px;   /* cap so it doesn't look absurd in huge cells */
    overflow: visible;
  }

  .g-track { fill:none; stroke:#e0e0e0; stroke-width:3; stroke-linecap:round; }
  .g-fill  { fill:none; stroke:var(--color-rtpurple); stroke-width:3; stroke-linecap:round; }

  .g-val {
    font-size: 7.5px;
    fill: #333;
    text-anchor: middle;
    dominant-baseline: middle;
    font-family: sans-serif;
    font-weight: 600;
  }
  .g-label { font-size: 9px; color: #888; font-family: sans-serif; }
</style>