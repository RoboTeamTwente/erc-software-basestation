import { listen } from '@tauri-apps/api/event';

// ── Single source of truth for ports ──────────────────────────
export const PORTS = {
  depth: 5000,
  front: 5001,
  arm:   5002,
};

// ── Camera states ──────────────────────────────────────────────
export const depthCamera = $state({
  name: "Depth Camera",
  port: `http://localhost:${PORTS.depth}`,
  stale: false,
});

export const frontCamera = $state({
  name: "Front Camera",
  port: `http://localhost:${PORTS.front}`,
  stale: false,
});

export const armCamera = $state({
  name: "Arm Camera",
  port: `http://localhost:${PORTS.arm}`,
  stale: false,
});

// ── Health listener ────────────────────────────────────────────
export async function initCameraHealthListener() {
  // Small delay to ensure Tauri bridge is ready before listening
  await new Promise(resolve => setTimeout(resolve, 500));

  await listen('camera-feed-status', (event) => {
    const port = Number(event.payload.port);
    const stale = event.payload.stale;

    if (port === PORTS.depth) depthCamera.stale = stale;
    else if (port === PORTS.front) frontCamera.stale = stale;
    else if (port === PORTS.arm) armCamera.stale = stale;
  });

  console.log('Camera health listener initialized');
}