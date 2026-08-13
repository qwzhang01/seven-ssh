<script setup lang="ts">
import { computed } from 'vue'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { getTheme } from '@/styles/themes'

const sessionStore = useSessionStore()
const settingsStore = useSettingsStore()

const statusText = computed(() => {
  const pane = sessionStore.activePane
  if (!pane) return 'No active session'
  switch (pane.status) {
    case 'connected': return `Connected to ${pane.host}`
    case 'connecting': return `Connecting to ${pane.host}...`
    case 'disconnected': return `Disconnected from ${pane.host}`
    case 'error': return `Connection error: ${pane.host}`
    default: return `Ready — ${pane.host}`
  }
})

const sessionCount = computed(() => {
  let active = 0
  for (const tab of sessionStore.tabs) {
    active += tab.panes.filter((p) => p.status === 'connected').length
  }
  return `${active} active`
})

const syncLabel = computed(() => {
  const tab = sessionStore.activeTab
  if (tab?.syncInput && tab.panes.length > 1) return '⇄ Sync'
  return ''
})

const themeLabel = computed(() => getTheme(settingsStore.terminalTheme).label)

const splitLabel = computed(() => {
  const tab = sessionStore.activeTab
  if (!tab || tab.splitDirection === 'none') return ''
  return `${tab.panes.length} panes`
})
</script>

<template>
  <div class="status-bar">
    <div class="status-left">
      <span class="status-text">{{ statusText }}</span>
      <span v-if="splitLabel" class="status-item split">{{ splitLabel }}</span>
      <span v-if="syncLabel" class="status-item sync">{{ syncLabel }}</span>
    </div>
    <div class="status-right">
      <span class="status-item">{{ themeLabel }}</span>
      <span class="status-item">{{ sessionCount }}</span>
      <span class="status-item">UTF-8</span>
    </div>
  </div>
</template>

<style scoped>
.status-bar {
  height: var(--status-bar-height);
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  flex-shrink: 0;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-text,
.status-item {
  font-size: 11px;
  color: var(--text-muted);
}

.status-item.sync {
  color: var(--accent);
}

.status-item.split {
  color: var(--text-secondary);
}
</style>
