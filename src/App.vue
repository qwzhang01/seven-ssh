<script setup lang="ts">
import { onMounted } from 'vue'
import { useConnectionStore } from '@/stores/connectionStore'

const connectionStore = useConnectionStore()

onMounted(async () => {
  await connectionStore.init()
})
</script>

<template>
  <router-view />
</template>
