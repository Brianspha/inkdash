<script setup lang="ts">
import { useParticlesHolder } from '../particles/particle-effects'

const { mesh } = useParticlesHolder()
</script>

<template>
  <TresObject3D ref="mesh" />
</template>