<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'

import type { Snippet } from '@/types'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { TERMINAL_THEMES } from '@/styles/themes'
import * as sshService from '@/services/ssh'

const emit = defineEmits<{ close: [] }>()

const sessionStore = useSessionStore()
const settingsStore = useSettingsStore()

const query = ref('')
const inputRef = ref<HTMLInputElement>()
const selectedIndex = ref(0)

interface PaletteItem {
  id: string
  label: string
  category: string
  action: () => void
}

const builtinCommands: PaletteItem[] = [
  {
    id: 'split-h',
    label: 'Split Horizontally',
    category: 'Split',
    action: () => { sessionStore.splitPane('horizontal'); emit('close') },
  },
  {
    id: 'split-v',
    label: 'Split Vertically',
    category: 'Split',
    action: () => { sessionStore.splitPane('vertical'); emit('close') },
  },
  {
    id: 'sync-input',
    label: 'Toggle Sync Input',
    category: 'Split',
    action: () => { sessionStore.toggleSyncInput(); emit('close') },
  },
  ...TERMINAL_THEMES.map((t) => ({
    id: `theme-${t.name}`,
    label: `Theme: ${t.label}`,
    category: 'Theme',
    action: () => { settingsStore.setTheme(t.name); emit('close') },
  })),
  {
    id: 'font-up',
    label: 'Increase Font Size',
    category: 'Settings',
    action: () => { settingsStore.setFontSize(settingsStore.fontSize + 1); emit('close') },
  },
  {
    id: 'font-down',
    label: 'Decrease Font Size',
    category: 'Settings',
    action: () => { settingsStore.setFontSize(settingsStore.fontSize - 1); emit('close') },
  },
]

// TODO: load user snippets from DB
const userSnippets = ref<Snippet[]>([
  { id: 'snip-1', name: 'Disk Usage', command: 'df -h', category: 'System' },
  { id: 'snip-2', name: 'Memory Info', command: 'free -m || vm_stat', category: 'System' },
  { id: 'snip-3', name: 'Process List', command: 'ps aux | head -20', category: 'System' },
  { id: 'snip-4', name: 'Network Ports', command: 'ss -tlnp || netstat -tlnp', category: 'Network' },
  { id: 'snip-5', name: 'Docker Containers', command: 'docker ps', category: 'Docker' },
  { id: 'snip-6', name: 'System Load', command: 'uptime', category: 'System' },
  { id: 'snip-7', name: 'Tail Syslog', command: 'tail -f /var/log/syslog 2>/dev/null || tail -f /var/log/messages', category: 'Logs' },
])

const snippetCommands = computed<PaletteItem[]>(() =>
  userSnippets.value.map((s) => ({
    id: `snippet-${s.id}`,
    label: `${s.name}: ${s.command}`,
    category: s.category ?? 'Snippet',
    action: () => {
      const pane = sessionStore.activePane
      if (pane?.sessionId) {
        const encoder = new TextEncoder()
        const bytes = Array.from(encoder.encode(s.command + '\n'))
        sshService.sshWrite(pane.sessionId, bytes)
      }
      emit('close')
    },
  }))
)

const allItems = computed(() => [...builtinCommands, ...snippetCommands.value])

const filtered = computed(() => {
  if (!query.value) return allItems.value
  const q = query.value.toLowerCase()
  return allItems.value.filter(
    (item) =>
      item.label.toLowerCase().includes(q) ||
      item.category.toLowerCase().includes(q)
  )
})

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, filtered.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = filtered.value[selectedIndex.value]
    if (item) item.action()
  }
}

onMounted(() => {
  inputRef.value?.focus()
  window.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div class="palette-overlay" @click.self="emit('close')">
    <div class="palette">
      <input
        ref="inputRef"
        v-model="query"
        type="text"
        placeholder="Type a command or snippet..."
        class="palette-input"
        @input="selectedIndex = 0"
      />
      <div class="palette-list">
        <div
          v-for="(item, i) in filtered"
          :key="item.id"
          class="palette-item"
          :class="{ selected: i === selectedIndex }"
          @click="item.action()"
          @mouseenter="selectedIndex = i"
        >
          <span class="item-category">{{ item.category }}</span>
          <span class="item-label">{{ item.label }}</span>
        </div>
        <div v-if="filtered.length === 0" class="palette-empty">
          No matching commands
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  justify-content: center;
  padding-top: 80px;
  z-index: 2000;
}

.palette {
  width: 520px;
  max-height: 400px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.palette-input {
  padding: 12px 16px;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 15px;
  outline: none;
}

.palette-input::placeholder {
  color: var(--text-muted);
}

.palette-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.palette-item.selected {
  background: var(--bg-hover);
}

.item-category {
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-surface);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  min-width: 50px;
  text-align: center;
}

.item-label {
  font-size: var(--font-size-md);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.palette-empty {
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: var(--font-size-md);
}
</style>
