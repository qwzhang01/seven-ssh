<script setup lang="ts">
withDefaults(defineProps<{
  size?: number
  variant?: 'icon' | 'full'
  glow?: boolean
}>(), {
  size: 48,
  variant: 'full',
  glow: false,
})
</script>

<template>
  <img
    :src="variant === 'icon' ? '/icon.png' : '/logo.png'"
    :width="size"
    :height="size"
    alt="SevenSSH"
    class="app-logo"
    :class="{ 'app-logo--glow': glow }"
    draggable="false"
  />
</template>

<style scoped>
.app-logo {
  display: block;
  object-fit: contain;
  flex-shrink: 0;
}

.app-logo--glow {
  filter: drop-shadow(0 0 12px rgba(137, 220, 235, 0.35));
}
</style>
