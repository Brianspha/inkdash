import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useGameStore = defineStore('game', () => {
  const address = ref('')
  const isLoading = ref(false)
  const showMenu = ref(true)
  const gameActive = ref(false)
  const score = ref(0)
  const distance = ref(0)
  const restart = ref(false)
  const canMove = ref(false)
  const canDash = ref(false)
  return { restart, address, isLoading, showMenu, gameActive, score, distance, canMove, canDash }
})
