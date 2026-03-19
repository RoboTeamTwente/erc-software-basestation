<script lang="ts">
  import { Canvas } from '@threlte/core'
  import ModelViewer from './model_viewer.svelte'
  import * as THREE from 'three'
  import { onMount } from 'svelte'
  import { debugLogs, appendLog, clearLogs, setLoadFailed, wasLoadFailed } from './model_debug'

  const { modelPath = 'models/your-model.glb' }: { modelPath?: string } = $props()

  let ready = $state(false)
  let error = $state(wasLoadFailed())
  let wrapperEl: HTMLDivElement
  let viewerRef: ModelViewer = $state(undefined as any)

  const log = appendLog

  function handleError() {
    log(`handleError | error=${error}`)
    if (error) return

    viewerRef?.killRenderer()

    const canvas = wrapperEl?.querySelector('canvas')
    if (canvas) {
      const ctx = canvas.getContext('webgl2') ?? canvas.getContext('webgl')
      ctx?.getExtension('WEBGL_lose_context')?.loseContext()
      canvas.style.display = 'none'
      log('canvas hidden')
    }

    setLoadFailed(true)
    log('error persisted — setting error=true ready=false')
    error = true
    ready = false
  }

  let healthInterval: ReturnType<typeof setInterval>
  let mountTimer: ReturnType<typeof setTimeout>

  function startModel() {
    clearTimeout(mountTimer)
    clearInterval(healthInterval)
    log('startModel — scheduling canvas mount')

    mountTimer = setTimeout(() => {
      if (error) return
      log('setting ready=true')
      ready = true

      setTimeout(() => {
        window.dispatchEvent(new Event('resize'))

        const canvas = wrapperEl?.querySelector('canvas')
        log(`post-mount canvas: ${!!canvas}`)

        if (canvas) {
          canvas.addEventListener('webglcontextlost', (e) => {
            e.preventDefault()
            log('webglcontextlost fired')
            handleError()
          })

          let ticks = 0
          healthInterval = setInterval(() => {
            if (error || !ready) { clearInterval(healthInterval); return }
            ticks++
            if (ticks < 3) return

            const gl = canvas.getContext('webgl2') ?? canvas.getContext('webgl') as WebGLRenderingContext | null
            if (!gl) return

            const w = canvas.width
            const h = canvas.height
            const points = [
              [Math.floor(w / 2), Math.floor(h / 2)],
              [Math.floor(w / 4), Math.floor(h / 4)],
              [Math.floor(w * 3 / 4), Math.floor(h / 4)],
              [Math.floor(w / 4), Math.floor(h * 3 / 4)],
              [Math.floor(w * 3 / 4), Math.floor(h * 3 / 4)],
            ]
            const px = new Uint8Array(4)
            const allBlank = points.every(([x, y]) => {
              gl.readPixels(x, y, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, px)
              return px[0] === 0 && px[1] === 0 && px[2] === 0 && px[3] === 0
            })

            if (allBlank) {
              log(`health: all 5 sample points blank at tick ${ticks} — remounting canvas`)
              clearInterval(healthInterval)
              ready = false
              setTimeout(() => { if (!error) ready = true }, 50)
            }
          }, 2000)
        }

        if (wrapperEl) {
          wrapperEl.style.width = 'calc(100% - 1px)'
          requestAnimationFrame(() => { wrapperEl.style.width = '100%' })
        }
      }, 50)
    }, 100)
  }

  onMount(() => {
    if (error) {
      log('onMount — load previously failed, staying on error screen')
      return
    }
    startModel()
    return () => {
      clearTimeout(mountTimer)
      clearInterval(healthInterval)
    }
  })

  function retry() {
    log('retry — clearing error and restarting')
    setLoadFailed(false)
    error = false
    ready = false
    setTimeout(() => startModel(), 50)
  }
</script>

<div class="container">
  <h1 class="heading">3D Model</h1>

  <div class="canvas-wrapper" bind:this={wrapperEl}>
    {#if error}
      <div class="model-error">
        <span class="model-error-icon">⚠</span>
        <span>Failed to load 3D model</span>
        <button class="button secondary" onclick={retry}>Retry</button>
      </div>
    {:else if ready}
      <Canvas
        toneMapping={THREE.ACESFilmicToneMapping}
        colorSpace={THREE.SRGBColorSpace}
      >
        <ModelViewer
          bind:this={viewerRef}
          {modelPath}
          onerror={handleError}
          onlog={log}
        />
      </Canvas>
    {/if}
  </div>

  {#if !error}
    <div class="model-hint">Drag to rotate · Scroll to zoom · Right-drag to pan</div>
  {/if}
</div>

<!-- Fixed debug overlay -->
<div class="model-log">
  <div class="model-log-header">
    Debug Log
    <button class="model-log-clear" onclick={clearLogs}>clear</button>
  </div>
  <div class="model-log-entries">
    {#each $debugLogs as entry}<div>{entry}</div>{/each}
  </div>
</div>

<style>
  .model-log {
    position: fixed;
    bottom: 16px;
    right: 16px;
    width: 420px;
    max-height: 300px;
    display: flex;
    flex-direction: column;
    font-size: 11px;
    font-family: monospace;
    background: rgba(10, 10, 10, 0.92);
    color: #b0ffb0;
    border-radius: 8px;
    border: 1px solid rgba(255,255,255,0.1);
    z-index: 9999;
    overflow: hidden;
    user-select: text;
    pointer-events: all;
  }

  .model-log-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 5px 10px;
    font-weight: bold;
    font-size: 10px;
    color: #888;
    border-bottom: 1px solid rgba(255,255,255,0.08);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    flex-shrink: 0;
  }

  .model-log-clear {
    background: none;
    border: none;
    color: #555;
    cursor: pointer;
    font-family: monospace;
    font-size: 10px;
    padding: 0;
  }

  .model-log-clear:hover { color: #aaa; }

  .model-log-entries {
    overflow-y: auto;
    padding: 6px 10px;
    line-height: 1.6;
  }

  .model-error {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: #666;
    font-size: 0.95rem;
  }

  .model-error-icon {
    font-size: 2rem;
    color: var(--color-rtpurple);
  }

  .model-hint {
    position: absolute;
    bottom: 10px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 11px;
    color: #aaa;
    letter-spacing: 0.05em;
    pointer-events: none;
    white-space: nowrap;
    z-index: 2;
  }
</style>