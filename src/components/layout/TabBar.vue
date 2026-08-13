<script setup lang="ts">
import { useSessionStore } from '@/stores/sessionStore'

defineProps<{ isMacos?: boolean }>()
const emit = defineEmits<{ 'open-palette': [] }>()
const sessionStore = useSessionStore()

function statusClass(tabId: string): string {
  const info = sessionStore.getTabDisplayInfo(tabId)
  if (!info) return 'status-idle'
  switch (info.status) {
    case 'connected': return 'status-connected'
    case 'connecting': return 'status-connecting'
    case 'error': return 'status-error'
    case 'disconnected': return 'status-disconnected'
    default: return 'status-idle'
  }
}

function tabLabel(tabId: string): string {
  const info = sessionStore.getTabDisplayInfo(tabId)
  if (!info) return 'Terminal'

  let label = info.connectionName

  const sameName = sessionStore.tabs.filter((t) => {
    const other = sessionStore.getTabDisplayInfo(t.id)
    return other?.connectionName === info.connectionName
  })
  if (sameName.length > 1) {
    const idx = sameName.findIndex((t) => t.id === tabId)
    label += ` #${idx + 1}`
  }

  return label
}
</script>

<template>
  <div class="tab-bar" :class="{ 'drag-region': isMacos }" :data-tauri-drag-region="isMacos || undefined">
    <div class="tabs-scroll">
      <div
        v-for="tab in sessionStore.tabs"
        :key="tab.id"
        class="tab no-drag"
        :class="{ active: sessionStore.activeTabId === tab.id }"
        @click="sessionStore.setActiveTab(tab.id)"
      >
        <span class="tab-dot" :class="statusClass(tab.id)" />
        <span class="tab-name">{{ tabLabel(tab.id) }}</span>
        <span v-if="tab.panes.length > 1" class="pane-count" :title="`${tab.panes.length} panes`">{{ tab.panes.length }}</span>
        <span v-if="tab.syncInput" class="sync-badge" title="Sync Input">⇄</span>
        <button class="tab-close" @click.stop="sessionStore.removeTab(tab.id)" title="Close">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M4 4l6 6M10 4l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </div>

    <button class="toolbar-btn no-drag" title="Command Palette (⌘⇧P)" @click="emit('open-palette')">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <rect x="1" y="3" width="14" height="2" rx="1" fill="currentColor" />
        <rect x="1" y="7" width="14" height="2" rx="1" fill="currentColor" />
        <rect x="1" y="11" width="14" height="2" rx="1" fill="currentColor" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.tab-bar {
  height: var(--tab-bar-height);
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: stretch;
  overflow: hidden;
}

.tabs-scroll {
  display: flex;
  overflow-x: auto;
  flex: 1;
}

.tabs-scroll::-webkit-scrollbar {
  height: 2px;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  min-width: 120px;
  max-width: 200px;
  cursor: pointer;
  border-right: 1px solid var(--border-color);
  transition: background var(--transition-fast);
  flex-shrink: 0;
  position: relative;
}

.tab:hover { background: var(--bg-hover); }
.tab.active { background: var(--bg-primary); }

.tab.active::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--accent);
}

.tab-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-idle { background: var(--text-muted); }
.status-connected { background: var(--success); }
.status-connecting { background: var(--accent); animation: pulse 1.5s infinite; }
.status-error { background: var(--error); }
.status-disconnected { background: var(--warning); }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.tab-name {
  flex: 1;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab.active .tab-name { color: var(--text-primary); }

.pane-count {
  font-size: 9px;
  color: var(--text-muted);
  background: var(--bg-active);
  padding: 0 4px;
  border-radius: 8px;
  min-width: 14px;
  text-align: center;
  line-height: 14px;
  flex-shrink: 0;
}

.sync-badge {
  font-size: 10px;
  color: var(--accent);
  flex-shrink: 0;
}

.tab-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  opacity: 0;
  transition: all var(--transition-fast);
  position: relative;
  z-index: 2;
}

.tab:hover .tab-close,
.tab.active .tab-close { opacity: 1; }

.tab-close:hover {
  color: var(--error);
  background: rgba(255, 80, 80, 0.15);
}

.toolbar-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0 10px;
  display: flex;
  align-items: center;
  transition: color var(--transition-fast);
}

.toolbar-btn:hover {
  color: var(--text-primary);
}

.drag-region {
  -webkit-app-region: drag;
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
