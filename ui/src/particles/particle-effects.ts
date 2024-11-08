import { Power2, TweenMax } from 'gsap'
import * as THREE from 'three'
import { shallowRef } from 'vue'

export const particlesPool = shallowRef<Particle[]>([])
export const mesh = shallowRef<THREE.Object3D>()

export class Particle {
  mesh: THREE.Mesh<THREE.TetrahedronGeometry, THREE.MeshPhongMaterial>
  
  constructor() {
    // Make particles smaller to match the scale of the player box
    const geom = new THREE.TetrahedronGeometry(0.3, 0) // Reduced size from 3 to 0.3
    const mat = new THREE.MeshPhongMaterial({
      color: 0x009999,
      shininess: 0,
      specular: 0xFFFFFF,
      flatShading: true,
    })
    this.mesh = new THREE.Mesh(geom, mat)
    this.mesh.matrixAutoUpdate = true
  }

  explode(pos: THREE.Vector3, color: THREE.ColorRepresentation, scale: number) {
    const _p = this.mesh.parent
    this.mesh.material.color = new THREE.Color(color)
    this.mesh.material.needsUpdate = true
    this.mesh.scale.set(scale, scale, scale)

    // Tighter initial spawn radius around the player
    const randomRadius = Math.random() * 1.5 // Reduced from 20 to 1.5 units
    const randomAngle = Math.random() * Math.PI * 2

    // Calculate initial position close to the player
    const initialX = pos.x + Math.cos(randomAngle) * randomRadius
    const initialZ = pos.z + Math.sin(randomAngle) * randomRadius

    // Set initial position at player's height
    this.mesh.position.set(initialX, pos.y, initialZ)

    // Calculate target position for a smaller spread
    const targetRadius = randomRadius + 3 // Reduced from 30 to 3 units
    const targetAngle = randomAngle + (Math.random() - 0.5) * Math.PI / 2

    const targetX = pos.x + Math.cos(targetAngle) * targetRadius
    const targetZ = pos.z + Math.sin(targetAngle) * targetRadius

    // Faster animation for snappier effect
    const speed = 0.3 + Math.random() * 0.2

    // Animate rotation
    TweenMax.to(this.mesh.rotation, speed, {
      x: Math.random() * 12,
      y: Math.random() * 12,
      z: Math.random() * 12,
    })

    // Animate scale down
    TweenMax.to(this.mesh.scale, speed, {
      x: 0.1,
      y: 0.1,
      z: 0.1,
    })

    // Animate position with minimal Y variation
    TweenMax.to(this.mesh.position, speed, {
      x: targetX,
      y: pos.y + (Math.random() - 0.5) * 0.5, // Minimal Y variation
      z: targetZ,
      delay: Math.random() * 0.05, // Reduced delay for snappier effect
      ease: Power2.easeOut,
      onComplete: () => {
        if (_p)
          _p.remove(this.mesh)
        this.mesh.scale.set(1, 1, 1)
        particlesPool.value.unshift(this)
      },
    })
  }
}

export function useParticlesHolder() {
  function spawnParticles(
    pos: THREE.Vector3,
    density: number,
    color: THREE.ColorRepresentation,
    scale: number,
  ) {
    if (!mesh.value) {
      console.warn('Particle system mesh not initialized')
      return
    }

    mesh.value.updateMatrixWorld(true)

    const localPos = pos.clone()
    if (mesh.value.parent) {
      mesh.value.parent.updateMatrixWorld(true)
      localPos.applyMatrix4(mesh.value.parent.matrixWorld.invert())
    }

    const nParticles = Math.floor(density)
    for (let i = 0; i < nParticles; i++) {
      let particle: Particle
      if (particlesPool.value.length) {
        particle = particlesPool.value.pop()!
      }
      else {
        particle = new Particle()
      }

      mesh.value.add(particle.mesh)
      particle.mesh.visible = true
      particle.explode(localPos, color, scale)
    }
  }

  return { particlesPool, mesh, spawnParticles }
}