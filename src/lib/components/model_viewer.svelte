<script lang="ts">
  import { T, useTask, useThrelte } from '@threlte/core'
  import { OrbitControls } from '@threlte/extras'
  import { invoke } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import * as THREE from 'three'

  const {
    modelPath = 'models/your-model.glb',
    onerror,
    onlog,
  }: {
    modelPath?: string
    onerror?: () => void
    onlog?: (msg: string) => void
  } = $props()

  const log = (msg: string) => onlog?.(msg)

  // Load the GLB via Tauri invoke (Rust reads the file) instead of HTTP fetch.
  // This bypasses WebKit's network loader (WebLoaderStrategy.cpp) where the bug lives.
  let scene: THREE.Group | null = $state(null)

  async function loadModel() {
    try {
      log(`loading model via Tauri invoke: ${modelPath}`)
      const bytes: number[] = await invoke('load_model', { path: modelPath })
      const buffer = new Uint8Array(bytes).buffer

      const { GLTFLoader } = await import('three/examples/jsm/loaders/GLTFLoader.js')
      const loader = new GLTFLoader()

      loader.parse(buffer, '', (gltf) => {
        log('gltf parsed — applying materials')

        gltf.scene.traverse((child: THREE.Object3D) => {
          if ((child as THREE.Mesh).isMesh) {
            (child as THREE.Mesh).material = new THREE.MeshStandardMaterial({
              color: new THREE.Color('#5A1C74'),
              roughness: 0.5,
              metalness: 0.1,
              emissive: new THREE.Color('#000000'),
            })
          } else if (child instanceof THREE.LineSegments || child instanceof THREE.Line) {
            ;(child as THREE.Line).material = new THREE.LineBasicMaterial({
              color: new THREE.Color('#5A1C74'),
            })
          }
        })

        fitCamera(gltf.scene)
        scene = gltf.scene as unknown as THREE.Group
        scaleTarget = 1
        log('model ready')
      }, (err) => {
        log(`GLTFLoader.parse error: ${err}`)
        onerror?.()
      })
    } catch (err) {
      log(`invoke error: ${err}`)
      onerror?.()
    }
  }

  // ── Camera fit ────────────────────────────────────────────────────────────
  let cameraRef: THREE.PerspectiveCamera
  let controlsRef: any
  let camPos: [number, number, number] = $state([0, 1.5, 4])
  let ctrlTarget: [number, number, number] = $state([0, 0, 0])
  let minDist = $state(1)
  let maxDist = $state(20)

  function fitCamera(root: THREE.Object3D) {
    const box    = new THREE.Box3().setFromObject(root)
    const size   = box.getSize(new THREE.Vector3())
    const center = box.getCenter(new THREE.Vector3())

    const maxDim      = Math.max(size.x, size.y, size.z)
    const fovRad      = ((cameraRef?.fov ?? 45) * Math.PI) / 180
    const fitDistance = (maxDim / 2 / Math.tan(fovRad / 2)) * 1.5

    const pos = new THREE.Vector3(center.x, center.y + size.y * 0.15, center.z + fitDistance)
    camPos     = [pos.x, pos.y, pos.z]
    ctrlTarget = [center.x, center.y, center.z]
    minDist    = fitDistance * 0.2
    maxDist    = fitDistance * 5

    if (cameraRef) {
      cameraRef.position.copy(pos)
      cameraRef.lookAt(center)
      cameraRef.updateProjectionMatrix()
    }
    if (controlsRef) {
      controlsRef.target.copy(center)
      controlsRef.update()
    }
  }

  // ── Scale-in animation ────────────────────────────────────────────────────
  let scale = $state(0)
  let scaleTarget = $state(0)

  let stopped = false
  const { stop: stopTask } = useTask((delta) => {
    if (stopped) return
    scale += (scaleTarget - scale) * Math.min(1, delta * 6)
  })

  const { renderer } = useThrelte()

  // Load once on mount — using onMount instead of $effect so interactions
  // can't accidentally trigger a reload by changing reactive state
  onMount(() => { loadModel() })

  // ── Kill switch ───────────────────────────────────────────────────────────
  export function killRenderer() {
    log(`killRenderer | renderer=${!!renderer}`)
    stopped = true
    stopTask()
    if (renderer) {
      renderer.setAnimationLoop(null)
      renderer.dispose()
      log('renderer disposed')
    } else {
      log('WARNING: no renderer')
    }
  }
</script>

<T.Color attach="background" args={['#D9D9D9']} />

<T.PerspectiveCamera bind:ref={cameraRef} makeDefault fov={45} position={camPos}>
  <OrbitControls
    bind:ref={controlsRef}
    enableDamping
    dampingFactor={0.05}
    autoRotate={true}
    autoRotateSpeed={1.2}
    minDistance={minDist}
    maxDistance={maxDist}
    maxPolarAngle={Math.PI / 1.8}
    target={ctrlTarget}
  />
</T.PerspectiveCamera>

<T.AmbientLight intensity={0.7} color="#f0ecf5" />
<T.DirectionalLight
  position={[5, 8, 5]}
  intensity={1.6}
  color="#e8dff2"
  castShadow
  shadow.mapSize.width={2048}
  shadow.mapSize.height={2048}
  shadow.camera.near={0.1}
  shadow.camera.far={50}
/>
<T.DirectionalLight position={[-4, 3, -4]} intensity={0.6} color="#ffffff" />
<T.PointLight position={[0, -2, 2]} intensity={0.3} color="#5A1C74" distance={20} />

<T.Mesh receiveShadow rotation.x={-Math.PI / 2} position.y={-0.01}>
  <T.CircleGeometry args={[6, 64]} />
  <T.ShadowMaterial opacity={0.12} />
</T.Mesh>

{#if scene === null}
  <!-- Loading placeholder while Rust reads the file -->
  <T.Mesh>
    <T.TorusGeometry args={[0.4, 0.06, 16, 64]} />
    <T.MeshStandardMaterial color="#5A1C74" wireframe />
  </T.Mesh>
{:else}
  <T.Group scale={scale} castShadow>
    <T is={scene} castShadow receiveShadow />
  </T.Group>
{/if}