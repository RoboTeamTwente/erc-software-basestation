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
    let magSeries: Series[] = [
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

    function scheduleRender() {
        if (rafId !== null) return;
        rafId = requestAnimationFrame(() => {
            rafId = null;
            if (!pending) return;

            imu = pending;
            pending = null;

            push(accelSeries, [imu.accel_x, imu.accel_y, imu.accel_z]);
            push(gyroSeries,  [imu.gyro_x,  imu.gyro_y,  imu.gyro_z]);
            push(magSeries,   [imu.mag_x,   imu.mag_y,   imu.mag_z]);
            accelSeries = [...accelSeries];
            gyroSeries  = [...gyroSeries];
            magSeries   = [...magSeries];

            const now = performance.now();
            const dt = lastTs ? (now - lastTs) / 1000 : 0.02;
            lastTs = now;
            pitch = (pitch + imu.gyro_x * dt * 57.2958) % 360;
            roll  = (roll  + imu.gyro_y * dt * 57.2958) % 360;
            yaw   = (yaw   + imu.gyro_z * dt * 57.2958) % 360;
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

        <!-- Left: sensor panels stacked -->
        <div class="sensors">

        {#each [
            { label: 'Accel', unit: 'm/s²', vals: [imu.accel_x, imu.accel_y, imu.accel_z], series: accelSeries },
            { label: 'Gyro',  unit: '°/s',  vals: [imu.gyro_x,  imu.gyro_y,  imu.gyro_z],  series: gyroSeries  },
            { label: 'Mag',   unit: 'µT',   vals: [imu.mag_x,   imu.mag_y,   imu.mag_z],   series: magSeries   },
        ] as sensor}
            <div class="sensor-row">
            <div class="sensor-meta">
                <span class="sensor-label">{sensor.label}</span>
                <span class="sensor-unit">{sensor.unit}</span>
            </div>
            <div class="sensor-vals">
                {#each ['X','Y','Z'] as axis, i}
                <div class="val-chip" style="--ac:{sensor.series[i].color}">
                    <span class="axis-tag">{axis}</span>
                    <span class="axis-val">{fmt(sensor.vals[i])}</span>
                </div>
                {/each}
            </div>
            <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
                {#each sensor.series as s}
                <path d={sparkPath(s.data, 120, 28)} fill="none" stroke={s.color} stroke-width="1.2" opacity="0.8"/>
                {/each}
            </svg>
            </div>
        {/each}

        </div>

        <!-- Right: orientation cube -->
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

    </div>
</div>