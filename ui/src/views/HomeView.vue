<template>
  <div class="main-container">
    <!-- 3D Scene Container -->
    <div class="game-container">
      <TresCanvas clear-color="#f8e5d1" window-size :camera-controls="false">
        <TresPerspectiveCamera :position="cameraPosition" :look-at="lookAtPosition" />
        <TresScene>
          <TresAmbientLight :intensity="0.5" />
          <TresDirectionalLight :position="[5, 5, 5]" :intensity="1" />

          <template v-for="(plane, index) in planes" :key="index">
            <Plane :position="plane.position" :rotation="[-Math.PI / 2, 0, 0]" :args="[10, 10]">
              <TresMeshStandardMaterial :color="plane.color" :emissive="plane.glowing ? '#ffff00' : '#000000'"
                :emissiveIntensity="plane.glowing ? 0.5 : 0" />

              <!-- Stones -->
              <template v-for="(stone, stoneIndex) in plane.stones" :key="stoneIndex">
                <TresMesh :ref="el => setStoneRef(plane.index, stoneIndex, el)"
                  :position="getStonePosition(stone, plane)" @ready="initStoneCollider(stone)">
                  <TresSphereGeometry :args="[0.2, 16, 16]" />
                  <TresMeshStandardMaterial color="gray" />
                </TresMesh>
              </template>

              <!-- Collectibles -->
              <template v-if="plane.collectible">
                <TresMesh :ref="el => setCollectibleRef(plane.index, plane.collectible.index, el)"
                  :position="getCollectiblePosition(plane.collectible, plane)"
                  @ready="initCollectibleCollider(plane.collectible)">
                  <TresSphereGeometry :args="[0.2, 16, 16]" />
                  <TresMeshStandardMaterial :visible="plane.collectible?.visible" color="green" />
                </TresMesh>
              </template>
            </Plane>
          </template>

          <TresMesh :ref="player" :position="playerPosition" cast-shadow>
            <TresBoxGeometry :rotation="playerRotation" :args="[1, 1, 1]" />
            <TresMeshStandardMaterial color="purple" />
          </TresMesh>
          <Particles />
        </TresScene>
      </TresCanvas>
    </div>

    <GameHUD> </GameHUD>
  </div>
</template>
<script setup lang="ts">
import GameHUD from '@/components/GameHUD.vue'
import { ref, onMounted, onUnmounted, shallowRef, watch } from 'vue'
import { TresCanvas, } from '@tresjs/core'
import { Plane, Box, TransformControls, OrbitControls, } from '@tresjs/cientos'
import { useRenderLoop } from '@tresjs/core'
import { Box3, Vector3, Sphere, Mesh } from 'three'
import Particles from "../components/Particles.vue"
import { useParticlesHolder } from '../particles/particle-effects'
import { useGameStore } from '@/stores/gamestore'
const store = useGameStore()
const cameraShake = ref({
  intensity: 0,
  decay: 0.9,
  offset: { x: 0, y: 0, z: 0 }
})
const SHAKE_CONFIGS = {
  collectible: { intensity: 0.3, decay: 0.85 },
  stone: { intensity: 0.5, decay: 0.8 }
}
const { spawnParticles } = useParticlesHolder()
const startTime = ref(0)
const completionTime = ref(0)

const currentLevel = ref(1)
const energy = ref(100)
const stoneMeshes = ref<Map<string, Mesh>>(new Map())
const collectibleMeshes = ref<Map<string, Mesh>>(new Map())
const planes = ref([] as Plane[])
const currentPlaneIndex = ref(0)
const totalInitialPlanes = 49
const player = shallowRef()
const playerPosition = ref([0, 0.5, 2])
const playerRotation = ref([0, Math.PI, 0])
const isMoving = ref(false)
const isDashing = ref(false)
const moveStartTime = ref(0)
const movementDuration = 200
const dashDuration = 100
const { onLoop } = useRenderLoop()
const planeLength = 10
const colors = [
  '#d4bfb3', '#c1a797', '#b17f6a', '#915f4a',
  '#d4bfb3', '#c1a797', '#b17f6a', '#915f4a'
]
const STONES_PER_CIRCLE = 4
const CIRCLE_RADIUS = 3
const STONE_ROTATION_SPEED = 0.03

type Stone = {
  angle: number
  radius: number
  height: number
  movementPattern: string
  movementOffset: number
  movementDirection: number
  collider?: any
}
type Collectible = {
  radius: number
  height: number
  color: string
  index: number
  angle: number,
  picked: boolean
  visible: boolean,
  collider?: any
}
type Plane = {
  position: number[]
  color: string
  index: number
  stones: Stone[]
  collectible?: Collectible
  glowing: boolean
}
type Tree = {
  position: number[]
  scale: number[]
}
const cameraOffset = [0, 5, 8]
const cameraPosition = ref<[number, number, number]>([
  playerPosition.value[0] + cameraOffset[0],
  playerPosition.value[1] + cameraOffset[1],
  playerPosition.value[2] + cameraOffset[2]
])
const lookAtPosition = ref<[number, number, number]>([...playerPosition.value] as [number, number, number])


const addCameraShake = (type: 'collectible' | 'stone') => {
  const config = SHAKE_CONFIGS[type]
  cameraShake.value.intensity = config.intensity
  cameraShake.value.decay = config.decay
}
const updateCamera = () => {
  const shakeX = (Math.random() - 0.5) * 2 * cameraShake.value.intensity
  const shakeY = (Math.random() - 0.5) * 2 * cameraShake.value.intensity
  const shakeZ = (Math.random() - 0.5) * 2 * cameraShake.value.intensity

  cameraShake.value.offset = {
    x: shakeX,
    y: shakeY,
    z: shakeZ
  }

  cameraShake.value.intensity *= cameraShake.value.decay

  cameraPosition.value = [
    playerPosition.value[0] + cameraOffset[0] + cameraShake.value.offset.x,
    playerPosition.value[1] + cameraOffset[1] + cameraShake.value.offset.y,
    playerPosition.value[2] + cameraOffset[2] + cameraShake.value.offset.z
  ]

  lookAtPosition.value = [
    playerPosition.value[0] + cameraShake.value.offset.x,
    playerPosition.value[1] + cameraShake.value.offset.y,
    playerPosition.value[2] + cameraShake.value.offset.z
  ]
}

const generateStones = (): Stone[] => {
  return Array.from({ length: Math.floor(Math.random() * STONES_PER_CIRCLE + 1) }, (_, i) => {
    const angleStep = (2 * Math.PI) / STONES_PER_CIRCLE
    return {
      angle: i * angleStep,
      radius: CIRCLE_RADIUS,
      height: 1,
    } as Stone
  })
}
const getStonePosition = (stone: Stone, plane: Plane): [number, number, number] => {
  const x = stone.radius * Math.cos(stone.angle)
  const y = stone.radius * Math.sin(stone.angle) - 3
  const z = 2
  return [x, y, z]
}
const getCollectiblePosition = (collectible: Collectible, plane: Plane): [number, number, number] => {
  const x = plane.position[0]
  const y = plane.position[1]
  const z = plane.position[2]
  return [0, -2.1, 1.4]
}
const updateStones = () => {
  planes.value.forEach((plane, planeIndex) => {
    plane.stones.forEach((stone, stoneIndex) => {
      stone.angle += STONE_ROTATION_SPEED
      if (stone.angle >= 2 * Math.PI) {
        stone.angle -= 2 * Math.PI
      }

      const stoneMesh = stoneMeshes.value.get(getStoneKey(planeIndex, stoneIndex))
      if (stoneMesh) {
        const newPos = getStonePosition(stone, plane)
        stoneMesh.position.set(newPos[0], newPos[1], newPos[2])
      }
    })
  })
}
const updateCollectibles = () => {
  planes.value.forEach((plane, planeIndex) => {
    const collectible = plane.collectible
    if (!collectible) return

    collectible.angle += STONE_ROTATION_SPEED
    if (collectible.angle >= 2 * Math.PI) {
      collectible.angle -= 2 * Math.PI
    }

    const collectibleMesh = collectibleMeshes.value.get(getStoneKey(planeIndex, collectible.index))
    if (collectibleMesh) {
      const newPos = getCollectiblePosition(collectible, plane)
      collectibleMesh.position.set(newPos[0], newPos[1], newPos[2])
    }
  })
}
const generateCollectible = (index: number): Collectible => {
  const angleStep = (2 * Math.PI) / STONES_PER_CIRCLE
  return {
    height: 1,
    color: '#ff0000',
    angle: index * angleStep,
    radius: CIRCLE_RADIUS,
    index: index,
    picked: false,
    visible: index === 1
  }
}
const initializePlanes = () => {
  planes.value = Array.from({ length: totalInitialPlanes }, (_, i) => ({
    position: [0, -0.5, -i * planeLength],
    color: colors[i % colors.length],
    index: i,
    stones: i == 0 ? [] : generateStones(),
    collectible: i == 0 ? undefined : generateCollectible(i)
  }))
}
const spwanNewPlanes = () => {
  if (currentPlaneIndex.value == planes.value.length - 5) {
    const newPlane = {
      position: [0, -0.5, -planes.value.length * planeLength],
      color: colors[planes.value.length % colors.length],
      index: planes.value.length,
      stones: generateStones(),
      collectible: generateCollectible(planes.value.length)
    }
    planes.value.push(newPlane)
    planes.value.splice(0, planes.value.length - 1)
  }

}
const checkGameEnd = () => {
  const lastPlaneIndex = planes.value.length - 2
  const playerPlaneIndex = Math.floor(Math.abs(playerPosition.value[2]) / planeLength)
  if (playerPlaneIndex >= lastPlaneIndex && store.gameActive) {
    endGame()
  }
}
const initStoneCollider = (stone: Stone) => {
  if (!stone.collider) {
    stone.collider = {
      boundingSphere: new Sphere(new Vector3(), 0.2),
      lastCollisionTime: 0
    }
  }
}
const initCollectibleCollider = (collectible: Collectible) => {
  if (!collectible.collider) {
    collectible.collider = {
      boundingSphere: new Sphere(new Vector3(), 0.4),
      lastCollisionTime: 0
    }
  }
}
const endGame = () => {
  completionTime.value = performance.now() - startTime.value
  store.gameActive = false
  store.gameActive = false
  planes.value = []
  console.log("player has died", store.gameActive)
}

const moveToNextPlane = () => {
  if (!store.canMove || isMoving.value || !store.gameActive || store.showMenu) return

  isMoving.value = true
  moveStartTime.value = performance.now()

  const duration = isDashing.value ? dashDuration : movementDuration
  const startZ = playerPosition.value[2]
  const targetZ = startZ - planeLength

  const animate = (currentTime: number) => {
    const elapsed = currentTime - moveStartTime.value
    const progress = Math.min(elapsed / duration, 1)

    const easeProgress = isDashing.value
      ? progress
      : progress * (2 - progress)

    const newZ = startZ + (targetZ - startZ) * easeProgress

    playerPosition.value = [
      playerPosition.value[0],
      playerPosition.value[1],
      newZ
    ]
    store.distance += Math.abs(Math.floor(newZ * 10))
    updateCamera()
    if (progress < 1 && store.gameActive) {
      requestAnimationFrame(animate)
    } else {
      playerPosition.value[2] = targetZ
      isMoving.value = false
      isDashing.value = false
      currentPlaneIndex.value++
      spawnParticles(new Vector3(playerPosition.value[0], 0, playerPosition.value[2] - 10), 10, "purple", 0.8)
      checkGameEnd()
    }
  }

  requestAnimationFrame(animate)
}

const handleMouseDown = (event: { button: number }) => {
  if (event.button === 0) {
    store.canMove = true
    if (store.canDash) {
      isDashing.value = true
      moveToNextPlane()
    }

  }
}
const handleMouseUp = (event: { button: number }) => {
  if (event.button === 0) {
    store.canMove = false
  }
}



const getStoneKey = (planeIndex: number, stoneIndex: number) => `${planeIndex}-${stoneIndex}`
const getCollectibleKey = (planeIndex: number, collectibleIndex: number) => `collectible-${planeIndex}-${collectibleIndex}`
const setStoneRef = (planeIndex: number, stoneIndex: number, el: any) => {
  if (el) {
    stoneMeshes.value.set(getStoneKey(planeIndex, stoneIndex), el)
  }
}
const setCollectibleRef = (planeIndex: number, collectibleIndex: number, el: any) => {
  if (el) {
    collectibleMeshes.value.set(getCollectibleKey(planeIndex, collectibleIndex), el)
  }
}
const checkCollisionsStones = () => {
  const playerBox = new Box3().setFromCenterAndSize(
    new Vector3(...playerPosition.value),
    new Vector3(2, 2, 2)
  )

  const currentTime = performance.now()
  let collision = false

  planes.value.forEach((plane, planeIndex) => {
    if (planeIndex === 0) return
    plane.stones.forEach((stone, stoneIndex) => {
      const stoneMesh = stoneMeshes.value.get(getStoneKey(planeIndex, stoneIndex))
      if (!stoneMesh || !stone.collider) return

      const worldPosition = new Vector3()
      stoneMesh.getWorldPosition(worldPosition)

      if (stone.collider.boundingSphere) {
        stone.collider.boundingSphere.center.copy(worldPosition)
      } else {
        console.warn('Stone collider not initialized properly.')
        return
      }
      if (checkSphereBoxCollision(stone.collider.boundingSphere, playerBox)) {
        collision = true
        addCameraShake('stone')
        stone.collider.lastCollisionTime = currentTime
        endGame()
      }
    })
  })

  return collision
}
const checkCollisionsCollectible = () => {
  const playerBox = new Box3().setFromCenterAndSize(
    new Vector3(...playerPosition.value),
    new Vector3(1, 1, 1)
  )

  const currentTime = performance.now()
  let collision = false

  planes.value.forEach((plane, planeIndex) => {
    if (planeIndex === 0) return

    const collectible = plane.collectible
    if (!collectible || collectible.picked) return

    const collectibleMesh = collectibleMeshes.value.get(getCollectibleKey(planeIndex, collectible.index))
    if (collectible.visible && !collectibleMesh || !collectible.collider) {
      return
    }

    const worldPosition = new Vector3()
    collectibleMesh.getWorldPosition(worldPosition)

    if (collectible.collider.boundingSphere) {
      collectible.collider.boundingSphere.center.copy(worldPosition)

      if (checkSphereBoxCollision(collectible.collider.boundingSphere, playerBox)) {
        collision = true
        collectible.collider.lastCollisionTime = currentTime
        console.log("changeddd", planeIndex, planes.value[planeIndex].collectible.visible)
        if (!collectible.picked) {
          collectible.picked = true
          store.score += 1
          // @ts-ignore
          planes.value[planeIndex].collectible.visible = false
          console.log("changeddd", worldPosition)
        }
      }
    }
  })

  return collision
}
const checkSphereBoxCollision = (sphere: Sphere, box: Box3): boolean => {
  const closestPoint = new Vector3(
    Math.max(box.min.x, Math.min(sphere.center.x, box.max.x)),
    Math.max(box.min.y, Math.min(sphere.center.y, box.max.y)),
    Math.max(box.min.z, Math.min(sphere.center.z, box.max.z))
  )

  const distanceSquared = closestPoint.distanceToSquared(sphere.center)

  return distanceSquared <= (sphere.radius * sphere.radius)
}
const shouldMakeVisible = () => {
  planes.value.forEach((plane, index) => {
    if (index == 0) return
    if (index - 1 == currentPlaneIndex.value && planes.value[index].collectible) {
      planes.value[index].collectible.visible = true
    }
  })
}

const restartGame = () => {
  store.restart = false;
  currentPlaneIndex.value = 0
  store.gameActive = true
  currentLevel.value = 1
  playerPosition.value = [0, 0.5, 2]
  store.distance = 0
  isMoving.value = false
  isDashing.value = false
  startTime.value = performance.now()
  energy.value = 100
  store.score = 0
  planes.value = []
  initializePlanes()
  updateCamera()

}
onLoop((deltaTime) => {
  if (store.restart) {
    restartGame()
  }
  if (store.gameActive) {
    updateStones()
    updateCollectibles()
    checkCollisionsStones()
    checkCollisionsCollectible()
    shouldMakeVisible()
    updateCamera()
  }
})

onMounted(() => {
  restartGame()
  store.showMenu = true
  startTime.value = performance.now()
  window.addEventListener('mousedown', handleMouseDown)
  window.addEventListener('mouseup', handleMouseUp)
  window.addEventListener('contextmenu', (e) => e.preventDefault())
})
onUnmounted(() => {
  window.removeEventListener('mousedown', handleMouseDown)
  window.removeEventListener('mouseup', handleMouseUp)
  window.removeEventListener('contextmenu', (e) => e.preventDefault())
})
</script>
<style scoped>
.main-container {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.game-container {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

.game-hud {
  position: fixed;
  top: 10%;
  right: 35%;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 50;
}

.game-header {
  text-align: center;
  margin-bottom: 1rem;
  padding-top: 1rem;
}

.game-header h1 {
  font-family: 'Georgia', serif;
  font-weight: 400;
  font-size: 2.5rem;
  letter-spacing: 0.1em;
  color: black;
  margin: 0;
}

.sub-heading {
  font-size: 1rem;
  letter-spacing: 0.2em;
  color: black;
}

.stats-bar {
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: var(--bg-light, #f8e5d1);
  padding: 1rem;
  border-radius: 30px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  gap: 2rem;
  margin: 0 auto;
  width: fit-content;
  pointer-events: all;
}

.menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.5);
  pointer-events: all;
}

.game-over-menu {
  background: white;
  padding: 2rem;
  border-radius: 20px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
  width: 80%;
  max-width: 400px;
  text-align: center;
}

.game-over-menu h2 {
  font-size: 2rem;
  margin: 0 0 1rem;
  color: var(--primary-color, #c09b79);
}

.game-over-menu button {
  background-color: var(--text-dark, #6e5339);
  color: white;
  border: none;
  border-radius: 10px;
  padding: 10px 20px;
  font-size: 1rem;
  cursor: pointer;
  transition: background-color 0.3s ease;
  margin: 0.5rem;
}

.game-over-menu button:hover {
  background-color: var(--secondary-color, #b17f6a);
}

.level {
  color: #000;
}

.level-value {
  color: #000;
}

/* Mobile layout */
@media (max-width: 768px) {
  .game-header h1 {
    font-size: 1.5rem;
  }

  .sub-heading {
    font-size: 0.8rem;
  }

  .stats-bar {
    max-width: 90%;
    padding: 0.5rem;
  }

  .game-over-menu {
    width: 90%;
    padding: 1.5rem;
  }
}
</style>