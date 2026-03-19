<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ----- STYLE IMPORTS -----
    import '../../imu.css';

// ── Types ──────────────────────────────────────────────────────────────────
    interface ImuPayload {
        accel_x: number; accel_y: number; accel_z: number;
        gyro_x:  number; gyro_y:  number; gyro_z:  number;
        mag_x:   number; mag_y:   number; mag_z:   number;
        is_calibrated: boolean;
        state: number;
        error_code: number;
    }

    const SENSOR_STATE: Record<number, string> = {
        0: 'Idle', 1: 'Operating', 2: 'Calibrating', 3: 'Error',
    };
    const IMU_ERROR: Record<number, string> = {
        0: 'No error',       1: 'Comm failure',  2: 'Cal required',
        3: 'Cal failed',     4: 'Invalid data',  5: 'Sensor fault',
        6: 'Gyro error',     7: 'Mag error',     8: 'Accel error',
    };

// ── Reactive state ─────────────────────────────────────────────────────────
    let imu: ImuPayload = {
        accel_x: 0, accel_y: 0, accel_z: 0,
        gyro_x:  0, gyro_y:  0, gyro_z:  0,
        mag_x:   0, mag_y:   0, mag_z:   0,
        is_calibrated: false, state: 0, error_code: 0,
    };

    const HISTORY = 60;
    type Series = { color: string; data: number[] };

    let accelSeries: Series[] = [
        { color: '#c0392b', data: Array(HISTORY).fill(0) },
        { color: '#5A1C74', data: Array(HISTORY).fill(0) },
        { color: '#1a7a4a', data: Array(HISTORY).fill(0) },
    ];
    let gyroSeries: Series[] = [
        { color: '#c0392b', data: Array(HISTORY).fill(0) },
        { color: '#5A1C74', data: Array(HISTORY).fill(0) },
        { color: '#1a7a4a', data: Array(HISTORY).fill(0) },
    ];

    let pitch = 0, roll = 0, yaw = 0;
    let lastTs = 0;
    let unlisten: UnlistenFn | null = null;
    let packetCount = 0;
    let hz = 0;
    let hzInterval: ReturnType<typeof setInterval>;
    let compassCanvas: HTMLCanvasElement;

    // ── rAF batching ───────────────────────────────────────────────────────
    let pending: ImuPayload | null = null;
    let rafId: number | null = null;

// ── Helpers ────────────────────────────────────────────────────────────────
    function push(series: Series[], vals: [number, number, number]) {
        series.forEach((s, i) => { s.data = [...s.data.slice(1), vals[i]]; });
    }

    const fmt = (n: number) => n.toFixed(2);

    function sparkPath(data: number[], w: number, h: number): string {
        let min = data[0], max = data[0];
        for (let i = 1; i < data.length; i++) {
            if (data[i] < min) min = data[i];
            if (data[i] > max) max = data[i];
        }
        const range = max - min || 1;
        let d = 'M';
        for (let i = 0; i < data.length; i++) {
            const x = (i / (data.length - 1)) * w;
            const y = h - ((data[i] - min) / range) * (h - 4) - 2;
            d += (i > 0 ? 'L' : '') + x.toFixed(1) + ',' + y.toFixed(1);
        }
        return d;
    }

    function isDark(): boolean {
        return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    function textColor()  { return isDark() ? '#c2c0b6' : '#3d3d3a'; }
    function mutedColor() { return isDark() ? '#5f5e5a' : '#b4b2a9'; }
    function ringColor()  { return isDark() ? 'rgba(255,255,255,0.15)' : 'rgba(0,0,0,0.15)'; }

    function drawCompass(mx: number, my: number) {
        if (!compassCanvas) return;
        const ctx = compassCanvas.getContext('2d');
        if (!ctx) return;
        const W = compassCanvas.width, H = compassCanvas.height;
        const cx = W / 2, cy = H / 2, r = W / 2 - 8;
        ctx.clearRect(0, 0, W, H);

        // Outer ring
        ctx.beginPath();
        ctx.arc(cx, cy, r, 0, Math.PI * 2);
        ctx.strokeStyle = ringColor();
        ctx.lineWidth = 1;
        ctx.stroke();

        // Tick marks
        for (let i = 0; i < 36; i++) {
            const a = (i / 36) * Math.PI * 2 - Math.PI / 2;
            const r1 = i % 9 === 0 ? r - 10 : (i % 3 === 0 ? r - 7 : r - 4);
            ctx.beginPath();
            ctx.moveTo(cx + Math.cos(a) * r1, cy + Math.sin(a) * r1);
            ctx.lineTo(cx + Math.cos(a) * (r - 1), cy + Math.sin(a) * (r - 1));
            ctx.strokeStyle = i % 9 === 0 ? textColor() : mutedColor();
            ctx.lineWidth = i % 9 === 0 ? 1.5 : 0.8;
            ctx.stroke();
        }

        // Cardinal labels
        const labelR = r - 18;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        (['N', 'E', 'S', 'W'] as const).forEach((lbl, idx) => {
            const a = (idx * 90 - 90) * Math.PI / 180;
            ctx.font = '500 11px sans-serif';
            ctx.fillStyle = lbl === 'N' ? '#E24B4A' : textColor();
            ctx.fillText(lbl, cx + Math.cos(a) * labelR, cy + Math.sin(a) * labelR);
        });

        // Needle
        const headingRad = Math.atan2(my, mx);
        const nLen = r - 26;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(cx + Math.cos(headingRad) * nLen, cy + Math.sin(headingRad) * nLen);
        ctx.strokeStyle = '#E24B4A';
        ctx.lineWidth = 2.5;
        ctx.lineCap = 'round';
        ctx.stroke();

        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(cx - Math.cos(headingRad) * (nLen * 0.55), cy - Math.sin(headingRad) * (nLen * 0.55));
        ctx.strokeStyle = mutedColor();
        ctx.lineWidth = 2;
        ctx.stroke();

        // Center dot
        ctx.beginPath();
        ctx.arc(cx, cy, 4, 0, Math.PI * 2);
        ctx.fillStyle = textColor();
        ctx.fill();
    }

    function scheduleRender() {
        if (rafId !== null) return;
        rafId = requestAnimationFrame(() => {
            rafId = null;
            if (!pending) return;

            imu = pending;
            pending = null;

            push(accelSeries, [imu.accel_x, imu.accel_y, imu.accel_z]);
            push(gyroSeries,  [imu.gyro_x,  imu.gyro_y,  imu.gyro_z]);
            accelSeries = [...accelSeries];
            gyroSeries  = [...gyroSeries];

            const now = performance.now();
            const dt = lastTs ? (now - lastTs) / 1000 : 0.02;
            lastTs = now;
            pitch = (pitch + imu.gyro_x * dt * 57.2958) % 360;
            roll  = (roll  + imu.gyro_y * dt * 57.2958) % 360;
            yaw   = (yaw   + imu.gyro_z * dt * 57.2958) % 360;

            drawCompass(imu.mag_x, imu.mag_y);
        });
    }

    $: cubeStyle = `transform: rotateX(${pitch.toFixed(1)}deg) rotateY(${yaw.toFixed(1)}deg) rotateZ(${roll.toFixed(1)}deg)`;

// ── Lifecycle ──────────────────────────────────────────────────────────────
    onMount(async () => {
        unlisten = await listen<ImuPayload>('imu-update', ({ payload }) => {
            pending = payload;
            packetCount++;
            scheduleRender();
        });

        hzInterval = setInterval(() => { hz = packetCount; packetCount = 0; }, 1000);
    });

    onDestroy(() => {
        unlisten?.();
        clearInterval(hzInterval);
        if (rafId !== null) cancelAnimationFrame(rafId);
    });
</script>


<div class="container">

    <!-- ── Header ── -->
    <div class="header" style="padding: 8px 12px 0;">
        <h1 class="heading" style="padding: 0; font-size: 1rem;">IMU</h1>
        <div class="status-row">
            <span class="pill" class:pill-ok={imu.is_calibrated} class:pill-warn={!imu.is_calibrated}>
                {imu.is_calibrated ? '✓ Cal' : '! Uncal'}
            </span>
            <span class="pill pill-neutral">{SENSOR_STATE[imu.state] ?? imu.state}</span>
            {#if imu.error_code !== 0}
                <span class="pill pill-err">{IMU_ERROR[imu.error_code] ?? imu.error_code}</span>
            {/if}
            <span class="pill pill-neutral hz">{hz} Hz</span>
        </div>
    </div>

    <!-- ── Body ── -->
    <div class="body">

        <!-- Accel -->
        <div class="sensor-row">
            <div class="sensor-meta">
                <span class="sensor-label">Accel</span>
                <span class="sensor-unit">m/s²</span>
            </div>
            <div class="sensor-vals">
                {#each ['X','Y','Z'] as axis, i}
                <div class="val-chip" style="--ac:{accelSeries[i].color}">
                    <span class="axis-tag">{axis}</span>
                    <span class="axis-val">{fmt([imu.accel_x, imu.accel_y, imu.accel_z][i])}</span>
                </div>
                {/each}
            </div>
            <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
                {#each accelSeries as s}
                <path d={sparkPath(s.data, 120, 28)} fill="none" stroke={s.color} stroke-width="1.2" opacity="0.8"/>
                {/each}
            </svg>
        </div>

        <!-- Gyro -->
        <div class="sensor-row">
            <div class="sensor-meta">
                <span class="sensor-label">Gyro</span>
                <span class="sensor-unit">°/s</span>
            </div>
            <div class="sensor-vals">
                {#each ['X','Y','Z'] as axis, i}
                <div class="val-chip" style="--ac:{gyroSeries[i].color}">
                    <span class="axis-tag">{axis}</span>
                    <span class="axis-val">{fmt([imu.gyro_x, imu.gyro_y, imu.gyro_z][i])}</span>
                </div>
                {/each}
            </div>
            <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
                {#each gyroSeries as s}
                <path d={sparkPath(s.data, 120, 28)} fill="none" stroke={s.color} stroke-width="1.2" opacity="0.8"/>
                {/each}
            </svg>
        </div>

        <!-- ── Bottom row: cube + compass ── -->
        <div class="bottom-row">

            <!-- Cube -->
            <div class="orient">
                <div class="scene">
                    <div class="cube" style={cubeStyle}>
                        <div class="face face-front">F</div>
                        <div class="face face-back">B</div>
                        <div class="face face-left">L</div>
                        <div class="face face-right">R</div>
                        <div class="face face-top">T</div>
                        <div class="face face-bottom">↓</div>
                    </div>
                </div>
                <div class="euler">
                    <div class="euler-row"><span class="euler-label">P</span><span class="euler-val">{pitch.toFixed(1)}°</span></div>
                    <div class="euler-row"><span class="euler-label">R</span><span class="euler-val">{roll.toFixed(1)}°</span></div>
                    <div class="euler-row"><span class="euler-label">Y</span><span class="euler-val">{yaw.toFixed(1)}°</span></div>
                </div>
            </div>

            <!-- Compass -->
            <div class="compass-wrap">
                <span class="compass-label">MAG</span>
                <canvas bind:this={compassCanvas} width="130" height="130"></canvas>
                <div class="mag-vals">
                    <div class="mag-chip"><span class="axis-tag" style="color:#c0392b">X</span><span class="axis-val">{fmt(imu.mag_x)}</span></div>
                    <div class="mag-chip"><span class="axis-tag" style="color:#5A1C74">Y</span><span class="axis-val">{fmt(imu.mag_y)}</span></div>
                    <div class="mag-chip"><span class="axis-tag" style="color:#1a7a4a">Z</span><span class="axis-val">{fmt(imu.mag_z)}</span></div>
                </div>
            </div>

        </div>

    </div>
</div>