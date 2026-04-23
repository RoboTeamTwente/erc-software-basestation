<script lang="ts">
    import { Canvas } from '@threlte/core'
    import * as THREE from 'three'
    import { onMount } from 'svelte'
    import { setLoadFailed, wasLoadFailed } from '../state/model_debug'
    import ModelViewer from './model_viewer.svelte'

    const { modelPath = 'models/chibiRover.glb' }: { modelPath?: string } = $props()

    let ready = $state(false)
    let error = $state(wasLoadFailed())
    let wrapperEl: HTMLDivElement
    let viewerRef: ModelViewer = $state(undefined as any)

    function handleError() {
        if (error) return

        viewerRef?.killRenderer()

        const canvas = wrapperEl?.querySelector('canvas')
        if (canvas) {
            const ctx = canvas.getContext('webgl2') ?? canvas.getContext('webgl')
            ctx?.getExtension('WEBGL_lose_context')?.loseContext()
            canvas.style.display = 'none'
        }

        setLoadFailed(true)
        error = true
        ready = false
    }

    let mountTimer: ReturnType<typeof setTimeout>

    function startModel() {
        clearTimeout(mountTimer)
        const startedAt = Date.now()

        mountTimer = setTimeout(() => {
            if (error) return
            ready = true

            setTimeout(() => {
                window.dispatchEvent(new Event('resize'))

                const canvas = wrapperEl?.querySelector('canvas')
                if (canvas) {
                    canvas.addEventListener('webglcontextlost', (e) => {
                        e.preventDefault()
                        if (Date.now() - startedAt < 1500) return
                        handleError()
                    })
                }

                if (wrapperEl) {
                    wrapperEl.style.width = 'calc(100% - 1px)'
                    requestAnimationFrame(() => { wrapperEl.style.width = '100%' })
                }
            }, 50)
        }, 100)
    }

    onMount(() => {
        if (error) return
        startModel()
        return () => clearTimeout(mountTimer)
    })

    function retry() {
        setLoadFailed(false)
        error = false
        ready = false
        setTimeout(() => startModel(), 50)
    }
</script>

<div class="container">
    <!-- <h1 class="heading">3D Model</h1> -->

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
                />
            </Canvas>
        {/if}
    </div>
</div>