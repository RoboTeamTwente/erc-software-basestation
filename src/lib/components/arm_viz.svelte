<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { armData } from '$lib/stores/arm';
  import type { ArmBoardActualPositions } from '$lib/proto/components/arm_board/movement_software_feedback';

  import'$lib/css/arm.css';

  let unlisten: UnlistenFn | undefined;

  // ── Projection (precompute trig constants) ────────────────────────────────
  const ISO_AZ = 45 * Math.PI / 180;
  const ISO_EL = 28 * Math.PI / 180;
  const SCALE  = 1.15;
  const SVG_OX = 100;
  const SVG_OY = 125;

  // Precomputed so project() does only multiplications
  const cosAZ = Math.cos(ISO_AZ), sinAZ = Math.sin(ISO_AZ);
  const cosEL = Math.cos(ISO_EL), sinEL = Math.sin(ISO_EL);

  function project(x: number, y: number, z: number) {
    const rx = x * cosAZ + z * sinAZ;
    const rz = -x * sinAZ + z * cosAZ;
    return {
      sx: SVG_OX + rx * SCALE,
      sy: SVG_OY - (y * cosEL - rz * sinEL) * SCALE,
    };
  }
  function projectFlat(x: number, z: number) { return project(x, 0, z); }
  function p2s(p: V3) { return project(p[0], p[1], p[2]); }
  function pstr(p: V3) { const {sx,sy} = p2s(p); return `${sx},${sy}`; }
  function pstrFlat(p: V3) { const {sx,sy} = projectFlat(p[0],p[2]); return `${sx},${sy}`; }

  // ── Math ──────────────────────────────────────────────────────────────────
  type V3 = [number,number,number];
  const d2r = (v: number | undefined) => (v ?? 0) * Math.PI / 180;
  const add  = (a: V3, b: V3): V3 => [a[0]+b[0], a[1]+b[1], a[2]+b[2]];
  const sc   = (v: V3, s: number): V3 => [v[0]*s, v[1]*s, v[2]*s];
  const dot  = (a: V3, b: V3) => a[0]*b[0]+a[1]*b[1]+a[2]*b[2];
  const cross= (a: V3, b: V3): V3 => [
    a[1]*b[2]-a[2]*b[1],
    a[2]*b[0]-a[0]*b[2],
    a[0]*b[1]-a[1]*b[0],
  ];
  function norm(v: V3): V3 {
    const m = Math.sqrt(v[0]**2+v[1]**2+v[2]**2) || 1;
    return [v[0]/m, v[1]/m, v[2]/m];
  }
  function rotY(v: V3, a: number): V3 {
    const c=Math.cos(a), s=Math.sin(a);
    return [v[0]*c+v[2]*s, v[1], -v[0]*s+v[2]*c];
  }
  function rotZ(v: V3, a: number): V3 {
    const c=Math.cos(a), s=Math.sin(a);
    return [v[0]*c-v[1]*s, v[0]*s+v[1]*c, v[2]];
  }
  function rotAxis(v: V3, axis: V3, a: number): V3 {
    const [ux,uy,uz]=axis, c=Math.cos(a), s=Math.sin(a), t=1-c;
    return [
      (t*ux*ux+c)*v[0]    + (t*ux*uy-s*uz)*v[1] + (t*ux*uz+s*uy)*v[2],
      (t*ux*uy+s*uz)*v[0] + (t*uy*uy+c)*v[1]    + (t*uy*uz-s*ux)*v[2],
      (t*ux*uz-s*uy)*v[0] + (t*uy*uz+s*ux)*v[1] + (t*uz*uz+c)*v[2],
    ];
  }

  // ── Shading (precomputed light colours as RGB tuples) ─────────────────────
  const LIGHT = norm([0.4, 1.0, 0.6]);
  // seg1: lit=#7a3a94  dark=#2a0840
  const S1L: V3 = [0x7a, 0x3a, 0x94], S1D: V3 = [0x2a, 0x08, 0x40];
  // seg2: lit=#6a2880  dark=#200630
  const S2L: V3 = [0x6a, 0x28, 0x80], S2D: V3 = [0x20, 0x06, 0x30];

  function shadeRGB(dir: V3, lit: V3, dark: V3): string {
    const t = (dot(norm(dir), LIGHT) + 1) / 2;
    const r = Math.round(dark[0] + (lit[0]-dark[0])*t);
    const g = Math.round(dark[1] + (lit[1]-dark[1])*t);
    const b = Math.round(dark[2] + (lit[2]-dark[2])*t);
    return `rgb(${r},${g},${b})`;
  }

  // ── Arm constants ─────────────────────────────────────────────────────────
  const L1=60, L2=48, LG=8, FL=13;
  // Finger half-thickness (visual only, tiny)
  const FH = 1.8;

  // World up — used as stable fallback for lateral axis
  const WORLD_UP: V3 = [0, 1, 0];

  // ── Static geometry (precomputed once) ───────────────────────────────────
  // Ground ring (projected ellipse)
  const groundArc: string = (() => {
    const r=18, pts: string[]=[];
    for (let i=0; i<=32; i++) {
      const a = -Math.PI + (2*Math.PI*i/32);
      const {sx,sy} = projectFlat(r*Math.cos(a), r*Math.sin(a));
      pts.push(`${i===0?'M':'L'} ${sx} ${sy}`);
    }
    return pts.join(' ');
  })();

  // Static world-axis lines (don't depend on arm pose)
  const AX_X = (() => { const {sx,sy}=project(22,0,0);  return {sx,sy}; })();
  const AX_Z = (() => { const {sx,sy}=project(0,0,22);  return {sx,sy}; })();
  const AX_Y = (() => { const {sx,sy}=project(0,22,0);  return {sx,sy}; })();

  // ── Reactive state (only what the template needs) ─────────────────────────
  let jawOpen  = false;
  let seg1Color = '#5A1C74';
  let seg2Color = '#401453';

  // Projected screen points
  let p0 = project(0,0,0),  p1 = project(0,60,0),
      p2 = project(0,108,0), p3 = project(0,126,0);

  // Shadow points (y=0 projections)
  let s0 = projectFlat(0,0), s1 = projectFlat(0,0),
      s2 = projectFlat(0,0), s3 = projectFlat(0,0);

  // Drop-line pairs: just 2 numbers each, no object array
  let d1x1=0,d1y1=0,d1x2=0,d1y2=0;
  let d2x1=0,d2y1=0,d2x2=0,d2y2=0;
  let d3x1=0,d3y1=0,d3x2=0,d3y2=0;

  let needleX=0, needleY=0;

  // Finger polygon point strings
  let f1pts='', f2pts='';
  let f1shadow='', f2shadow='';

  // Shadow arm lines (no blur — replaced with static offset + opacity)
  let shLine1='', shLine2='', shLine3='';

  $: {
    const d = $armData;
    const BASE  = d2r(d.base_actual_position);
    const TOP   = d2r(d.stepper_top_actual_position);
    const BOT   = d2r(d.stepper_bottom_actual_position);
    const ROT   = d2r(d.gripper_rotation_actual_position);
    const PITCH = d2r(d.gripper_pitch_actual_position);
    const JAW   = d.jaw_actual_position ?? 0;
    jawOpen = d.jaw_open ?? false;

    // ── Forward kinematics ──────────────────────────────────────────────────
    const seg1Dir: V3 = rotY(rotZ([0,1,0], TOP), BASE);
    const _p0: V3 = [0,0,0];
    const _p1 = add(_p0, sc(seg1Dir, L1));

    const seg2Dir: V3 = rotY(rotZ([0,1,0], TOP+BOT), BASE);
    const _p2 = add(_p1, sc(seg2Dir, L2));

    const baseGripDir: V3 = rotY(rotZ([0,1,0], TOP+BOT+PITCH), BASE);
    const gripDir = rotAxis(baseGripDir, norm(seg2Dir), ROT);
    const _p3 = add(_p2, sc(gripDir, LG));

    // ── Gripper lateral axis ────────────────────────────────────────────────
    // Cross gripDir with world-up to get a stable horizontal spread axis.
    // Fall back to world-X if gripDir is nearly vertical.
    let lateralRaw = cross(gripDir, WORLD_UP);
    if (lateralRaw[0]**2+lateralRaw[1]**2+lateralRaw[2]**2 < 0.001) {
      lateralRaw = cross(gripDir, [1,0,0]);
    }
    const lateral = norm(lateralRaw);
    const upDir   = norm(cross(lateral, gripDir));

    // ── Finger geometry ─────────────────────────────────────────────────────
    // jawSpread: 0° → fingers touching, 45° → spread by FL*sin(45°) ≈ 9 units
    const spread = Math.max(0.5, JAW * 0.22);  // half-gap between fingers

    // Each finger: a quad of 4 world-space corners
    // Finger 1: offset +spread along lateral
    // Finger 2: offset -spread along lateral
    function makeFingerQuad(side: 1|-1): [V3,V3,V3,V3] {
      const fingerBase = add(_p3, sc(lateral, side * spread));
      const fingerTip  = add(fingerBase, sc(gripDir, FL));
      const hw = FH / 2;
      return [
        add(add(fingerBase, sc(upDir,  hw)), sc(lateral, side * FH * 0.3)),
        add(add(fingerBase, sc(upDir, -hw)), sc(lateral, side * FH * 0.3)),
        add(add(fingerTip,  sc(upDir, -hw)), sc(lateral, side * FH * 0.3)),
        add(add(fingerTip,  sc(upDir,  hw)), sc(lateral, side * FH * 0.3)),
      ];
    }

    const q1 = makeFingerQuad(1);
    const q2 = makeFingerQuad(-1);

    f1pts    = q1.map(pstr).join(' ');
    f2pts    = q2.map(pstr).join(' ');
    f1shadow = q1.map(pstrFlat).join(' ');
    f2shadow = q2.map(pstrFlat).join(' ');

    // ── Project joints ──────────────────────────────────────────────────────
    p0 = p2s(_p0); p1 = p2s(_p1); p2 = p2s(_p2); p3 = p2s(_p3);

    // ── Shadow positions (flat projection) ──────────────────────────────────
    s0 = projectFlat(_p0[0],_p0[2]);
    s1 = projectFlat(_p1[0],_p1[2]);
    s2 = projectFlat(_p2[0],_p2[2]);
    s3 = projectFlat(_p3[0],_p3[2]);

    // ── Drop lines ──────────────────────────────────────────────────────────
    d1x1=p1.sx; d1y1=p1.sy; d1x2=s1.sx; d1y2=s1.sy;
    d2x1=p2.sx; d2y1=p2.sy; d2x2=s2.sx; d2y2=s2.sy;
    d3x1=p3.sx; d3y1=p3.sy; d3x2=s3.sx; d3y2=s3.sy;

    // ── Base needle ─────────────────────────────────────────────────────────
    const ndir = rotY([1,0,0], BASE);
    const np = projectFlat(ndir[0]*18, ndir[2]*18);
    needleX = np.sx; needleY = np.sy;

    // ── Segment shading ─────────────────────────────────────────────────────
    seg1Color = shadeRGB(seg1Dir, S1L, S1D);
    seg2Color = shadeRGB(seg2Dir, S2L, S2D);
  }
  
  onMount(async () => {
    unlisten = await listen<ArmBoardActualPositions>('arm-pos-update', e => {
      armData.set(e.payload);
    });
  });
  onDestroy(() => unlisten?.());
</script>

<div class="arm-viz">
  <div class="viz-header">
    <span class="heading">Arm Position</span>
    <span class="jaw-pill" class:open={jawOpen}>
      <span class="pip"></span>
      {jawOpen ? 'Jaw open' : 'Jaw closed'}
    </span>
  </div>

  <div class="diagram-card">
    <svg viewBox="0 0 200 175" xmlns="http://www.w3.org/2000/svg"
         class="arm-svg" preserveAspectRatio="xMidYMid meet"
         shape-rendering="optimizeSpeed">

      <!-- Ground ring -->
      <path d={groundArc} class="ground-arc"/>
      <!-- Base rotation needle -->
      <line x1={p0.sx} y1={p0.sy} x2={needleX} y2={needleY} class="base-needle"/>

      <!--
        Shadow: drawn as semi-transparent copies with no filter.
        Offset slightly down (+3px) and darkened. Much cheaper than feGaussianBlur.
      -->
      <g class="shadow-group">
        <line x1={s0.sx} y1={s0.sy} x2={s1.sx} y2={s1.sy} stroke="#5A1C74" stroke-width="6" stroke-linecap="round"/>
        <line x1={s1.sx} y1={s1.sy} x2={s2.sx} y2={s2.sy} stroke="#401453" stroke-width="5" stroke-linecap="round"/>
        <line x1={s2.sx} y1={s2.sy} x2={s3.sx} y2={s3.sy} stroke="#333"    stroke-width="2" stroke-linecap="round"/>
        <polygon points={f1shadow} fill="#333"/>
        <polygon points={f2shadow} fill="#333"/>
      </g>

      <!-- Drop lines (joint height indicators) -->
      <line x1={d1x1} y1={d1y1} x2={d1x2} y2={d1y2} class="drop-line"/>
      <line x1={d2x1} y1={d2y1} x2={d2x2} y2={d2y2} class="drop-line"/>
      <line x1={d3x1} y1={d3y1} x2={d3x2} y2={d3y2} class="drop-line"/>

      <!-- Base mount -->
      <ellipse cx={p0.sx} cy={p0.sy} rx="10" ry="4" class="base-ellipse"/>

      <!-- Upper arm (shaded) -->
      <line x1={p0.sx} y1={p0.sy} x2={p1.sx} y2={p1.sy}
            stroke={seg1Color} stroke-width="7" stroke-linecap="round"/>
      <!-- Lower arm (shaded) -->
      <line x1={p1.sx} y1={p1.sy} x2={p2.sx} y2={p2.sy}
            stroke={seg2Color} stroke-width="5.5" stroke-linecap="round"/>
      <!-- Gripper shaft -->
      <line x1={p2.sx} y1={p2.sy} x2={p3.sx} y2={p3.sy} class="gshaft"/>

      <!-- Gripper fingers -->
      <polygon points={f1pts} class="finger-quad"/>
      <polygon points={f2pts} class="finger-quad"/>

      <!-- Joints -->
      <circle cx={p0.sx} cy={p0.sy} r="5.5" class="joint jbase"/>
      <circle cx={p1.sx} cy={p1.sy} r="4.5" class="joint"/>
      <circle cx={p2.sx} cy={p2.sy} r="3.5" class="joint"/>
      <circle cx={p3.sx} cy={p3.sy} r="2.5" class="joint jtip"/>

      <!-- World axes (static screen positions) -->
      <line x1={p0.sx} y1={p0.sy} x2={AX_X.sx} y2={AX_X.sy} class="axis ax-x"/>
      <line x1={p0.sx} y1={p0.sy} x2={AX_Z.sx} y2={AX_Z.sy} class="axis ax-z"/>
      <line x1={p0.sx} y1={p0.sy} x2={AX_Y.sx} y2={AX_Y.sy} class="axis ax-y"/>
    </svg>
  </div>
</div>

<style>
</style>