<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useConnectionStore } from '@/stores/connectionStore'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import type { ConnectionInfo, GroupInfo, TerminalPane } from '@/types'
import ConnectionForm from '@/components/connection/ConnectionForm.vue'
import ImportSSHConfig from '@/components/connection/ImportSSHConfig.vue'
import ExportConnections from '@/components/connection/ExportConnections.vue'
import ContextMenu from '@/components/common/ContextMenu.vue'
import type { MenuItem } from '@/components/common/ContextMenu.vue'
import AppLogo from '@/components/common/AppLogo.vue'

const emit = defineEmits<{
  'open-sftp': [conn: ConnectionInfo]
  'open-settings': []
  'open-keys': []
  'open-ai': []
}>()

const connectionStore = useConnectionStore()
const sessionStore = useSessionStore()
const settingsStore = useSettingsStore()

const searchQuery = ref('')
const showNewConnection = ref(false)
const showImport = ref(false)
const showExport = ref(false)
const editingConnection = ref<ConnectionInfo | null>(null)

// Context menu state
const ctxMenu = ref<{ show: boolean; x: number; y: number; conn: ConnectionInfo | null }>({
  show: false, x: 0, y: 0, conn: null,
})

// Drag-and-drop state
const dragType = ref<'connection' | 'group' | null>(null)
const dragId = ref<string | null>(null)
const dropTarget = ref<{ type: 'connection' | 'group' | 'section'; id: string; position: 'before' | 'after' | 'inside' } | null>(null)

const filteredConnections = computed(() => {
  if (!searchQuery.value) return connectionStore.connections
  const q = searchQuery.value.toLowerCase()
  return connectionStore.connections.filter(
    (c) =>
      c.name.toLowerCase().includes(q) ||
      c.host.toLowerCase().includes(q) ||
      c.username.toLowerCase().includes(q)
  )
})

const rootGroups = computed(() => connectionStore.getChildGroups(null))

const ctxMenuItems = computed<MenuItem[]>(() => {
  const conn = ctxMenu.value.conn
  if (!conn) return []
  const isConnected = statusDot(conn) === 'connected'
  return [
    { id: 'connect', label: isConnected ? 'New Terminal' : 'Connect', icon: '🖥' },
    { id: 'sftp', label: 'Open SFTP', icon: '📂' },
    { id: 'divider-1', label: '', divider: true },
    { id: 'edit', label: 'Edit Connection', icon: '✏️' },
    { id: 'favorite', label: conn.is_favorite ? 'Remove Favorite' : 'Add to Favorites', icon: conn.is_favorite ? '☆' : '⭐' },
    { id: 'divider-2', label: '', divider: true },
    { id: 'delete', label: 'Delete Connection', icon: '🗑', danger: true },
  ]
})

function openConnection(conn: ConnectionInfo) {
  const pane: TerminalPane = {
    id: crypto.randomUUID(),
    connectionId: conn.id,
    connectionName: conn.name,
    status: 'idle',
    host: conn.host,
  }
  sessionStore.addTab(pane)
}

function showContextMenu(e: MouseEvent, conn: ConnectionInfo) {
  e.preventDefault()
  ctxMenu.value = { show: true, x: e.clientX, y: e.clientY, conn }
}

function handleCtxAction(id: string) {
  const conn = ctxMenu.value.conn
  if (!conn) return
  switch (id) {
    case 'connect':
      openConnection(conn)
      break
    case 'sftp':
      emit('open-sftp', conn)
      break
    case 'edit':
      editingConnection.value = conn
      break
    case 'favorite':
      connectionStore.toggleFavorite(conn.id)
      break
    case 'delete':
      connectionStore.removeConnection(conn.id)
      break
  }
}

function getGroupConnections(groupId: string) {
  return filteredConnections.value.filter((c) => c.group_id === groupId)
}

function getUngroupedConnections() {
  return filteredConnections.value.filter((c) => !c.group_id)
}

onMounted(() => {
  connectionStore.init()
  settingsStore.loadFromDB()
})

function statusDot(conn: ConnectionInfo): string {
  for (const tab of sessionStore.tabs) {
    const pane = tab.panes.find((p) => p.connectionId === conn.id)
    if (pane) return pane.status
  }
  return 'idle'
}

// Drag-and-drop handlers for connections
function onConnDragStart(e: DragEvent, conn: ConnectionInfo) {
  dragType.value = 'connection'
  dragId.value = conn.id
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', conn.id)
  }
}

function onConnDragOver(e: DragEvent, targetConn: ConnectionInfo, position: 'before' | 'after') {
  e.preventDefault()
  if (dragType.value !== 'connection' || dragId.value === targetConn.id) return
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  dropTarget.value = { type: 'connection', id: targetConn.id, position }
}

function onGroupDragOver(e: DragEvent, group: GroupInfo) {
  e.preventDefault()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  if (dragType.value === 'connection') {
    dropTarget.value = { type: 'group', id: group.id, position: 'inside' }
  } else if (dragType.value === 'group' && dragId.value !== group.id) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const pos = e.clientY < rect.top + rect.height / 2 ? 'before' : 'after'
    dropTarget.value = { type: 'group', id: group.id, position: pos }
  }
}

function onGroupDragStart(e: DragEvent, group: GroupInfo) {
  dragType.value = 'group'
  dragId.value = group.id
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', group.id)
  }
}

function onDragEnd() {
  dragType.value = null
  dragId.value = null
  dropTarget.value = null
}

async function onConnDrop(e: DragEvent, targetConn: ConnectionInfo, position: 'before' | 'after') {
  e.preventDefault()
  if (dragType.value !== 'connection' || !dragId.value || dragId.value === targetConn.id) {
    onDragEnd()
    return
  }

  const siblings = connectionStore.connections
    .filter((c) => c.group_id === targetConn.group_id)
    .sort((a, b) => a.sort_order - b.sort_order)

  const targetIdx = siblings.findIndex((c) => c.id === targetConn.id)
  const insertIdx = position === 'before' ? targetIdx : targetIdx + 1

  const newOrder = siblings.filter((c) => c.id !== dragId.value)
  const dragged = connectionStore.connections.find((c) => c.id === dragId.value)
  if (!dragged) { onDragEnd(); return }

  newOrder.splice(insertIdx > newOrder.length ? newOrder.length : insertIdx, 0, dragged)

  for (let i = 0; i < newOrder.length; i++) {
    if (newOrder[i].sort_order !== i || newOrder[i].group_id !== targetConn.group_id) {
      await connectionStore.updateConnection({
        id: newOrder[i].id,
        sort_order: i,
        group_id: targetConn.group_id || undefined,
      })
    }
  }
  onDragEnd()
}

async function onGroupDrop(e: DragEvent, group: GroupInfo) {
  e.preventDefault()
  if (!dragId.value) { onDragEnd(); return }

  if (dragType.value === 'connection') {
    const conn = connectionStore.connections.find((c) => c.id === dragId.value)
    if (conn && conn.group_id !== group.id) {
      const groupConns = connectionStore.connections.filter((c) => c.group_id === group.id)
      await connectionStore.updateConnection({
        id: conn.id,
        group_id: group.id,
        sort_order: groupConns.length,
      })
    }
  } else if (dragType.value === 'group' && dragId.value !== group.id) {
    const allGroups = [...rootGroups.value].sort((a, b) => a.sort_order - b.sort_order)
    const targetIdx = allGroups.findIndex((g) => g.id === group.id)
    const pos = dropTarget.value?.position === 'before' ? targetIdx : targetIdx + 1
    const reordered = allGroups.filter((g) => g.id !== dragId.value)
    const draggedGroup = allGroups.find((g) => g.id === dragId.value)
    if (draggedGroup) {
      reordered.splice(pos > reordered.length ? reordered.length : pos, 0, draggedGroup)
      for (let i = 0; i < reordered.length; i++) {
        if (reordered[i].sort_order !== i) {
          await connectionStore.updateGroupOrder(reordered[i].id, i)
        }
      }
    }
  }
  onDragEnd()
}

function isDropIndicator(connId: string, position: 'before' | 'after'): boolean {
  return dropTarget.value?.type === 'connection' && dropTarget.value.id === connId && dropTarget.value.position === position
}

function isGroupDropTarget(groupId: string): boolean {
  return dropTarget.value?.type === 'group' && dropTarget.value.id === groupId && dropTarget.value.position === 'inside'
}

function isGroupDropIndicator(groupId: string, position: 'before' | 'after'): boolean {
  return dropTarget.value?.type === 'group' && dropTarget.value.id === groupId && dropTarget.value.position === position
}
</script>

<template>
  <div class="sidebar-container">
    <div class="sidebar-header">
      <div class="app-brand">
        <AppLogo variant="icon" :size="22" />
        <h2 class="app-title">SevenSSH</h2>
      </div>
      <div class="header-actions">
        <button class="btn-icon" title="Export Connections" @click="showExport = true">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 2v8M8 2l3 3M8 2L5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M2 12v1a1 1 0 001 1h10a1 1 0 001-1v-1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
        <button class="btn-icon" title="Import Connections" @click="showImport = true">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 10V2M8 10l-3-3M8 10l3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M2 12v1a1 1 0 001 1h10a1 1 0 001-1v-1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
        <button class="btn-icon" title="New Connection" @click="showNewConnection = true">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </div>

    <div class="search-box">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
        <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.5" />
        <path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search connections..."
        class="search-input"
      />
    </div>

    <div class="connection-list">
      <!-- Favorites -->
      <template v-if="connectionStore.favorites.length > 0">
        <div class="section-header">
          <span class="section-icon">⭐</span>
          <span>Favorites</span>
        </div>
        <div
          v-for="conn in connectionStore.favorites"
          :key="conn.id"
          class="connection-item"
          @dblclick="openConnection(conn)"
          @contextmenu="showContextMenu($event, conn)"
        >
          <span class="status-dot" :class="statusDot(conn)" />
          <span class="conn-name">{{ conn.name }}</span>
          <button class="action-btn sftp-btn" title="SFTP" @click.stop="emit('open-sftp', conn)">
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M2 4h5l2 2h5v7a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V4z" stroke="currentColor" stroke-width="1.3" fill="none"/>
            </svg>
          </button>
          <button class="action-btn connect-btn" title="Connect" @click.stop="openConnection(conn)">
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M5 3l7 5-7 5V3z" fill="currentColor"/>
            </svg>
          </button>
          <span class="conn-host">{{ conn.host }}</span>
        </div>
      </template>

      <!-- Groups -->
      <template v-for="group in rootGroups" :key="group.id">
        <details class="group-section" open>
          <summary
            class="section-header"
            :class="{ 'drop-target-group': isGroupDropTarget(group.id), 'drop-indicator-before': isGroupDropIndicator(group.id, 'before'), 'drop-indicator-after': isGroupDropIndicator(group.id, 'after') }"
            draggable="true"
            @dragstart="onGroupDragStart($event, group)"
            @dragover="onGroupDragOver($event, group)"
            @drop="onGroupDrop($event, group)"
            @dragend="onDragEnd"
          >
            <span class="section-icon">📂</span>
            <span>{{ group.name }}</span>
            <span class="conn-count">{{ getGroupConnections(group.id).length }}</span>
          </summary>
          <div
            v-for="conn in getGroupConnections(group.id)"
            :key="conn.id"
            class="connection-item"
            :class="{ 'drop-indicator-before': isDropIndicator(conn.id, 'before'), 'drop-indicator-after': isDropIndicator(conn.id, 'after') }"
            draggable="true"
            @dragstart="onConnDragStart($event, conn)"
            @dragover="onConnDragOver($event, conn, $event.offsetY < ($event.currentTarget as HTMLElement).offsetHeight / 2 ? 'before' : 'after')"
            @drop="onConnDrop($event, conn, $event.offsetY < ($event.currentTarget as HTMLElement).offsetHeight / 2 ? 'before' : 'after')"
            @dragend="onDragEnd"
            @dblclick="openConnection(conn)"
            @contextmenu="showContextMenu($event, conn)"
          >
            <span class="status-dot" :class="statusDot(conn)" />
            <span class="conn-name">{{ conn.name }}</span>
            <button class="action-btn sftp-btn" title="SFTP" @click.stop="emit('open-sftp', conn)">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <path d="M2 4h5l2 2h5v7a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V4z" stroke="currentColor" stroke-width="1.3" fill="none"/>
              </svg>
            </button>
            <button class="action-btn connect-btn" title="Connect" @click.stop="openConnection(conn)">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <path d="M5 3l7 5-7 5V3z" fill="currentColor"/>
              </svg>
            </button>
            <span class="conn-host">{{ conn.host }}</span>
          </div>
        </details>
      </template>

      <!-- Ungrouped -->
      <template v-if="getUngroupedConnections().length > 0">
        <div class="section-header">
          <span class="section-icon">🖥️</span>
          <span>Servers</span>
        </div>
        <div
          v-for="conn in getUngroupedConnections()"
          :key="conn.id"
          class="connection-item"
          :class="{ 'drop-indicator-before': isDropIndicator(conn.id, 'before'), 'drop-indicator-after': isDropIndicator(conn.id, 'after') }"
          draggable="true"
          @dragstart="onConnDragStart($event, conn)"
          @dragover="onConnDragOver($event, conn, $event.offsetY < ($event.currentTarget as HTMLElement).offsetHeight / 2 ? 'before' : 'after')"
          @drop="onConnDrop($event, conn, $event.offsetY < ($event.currentTarget as HTMLElement).offsetHeight / 2 ? 'before' : 'after')"
          @dragend="onDragEnd"
          @dblclick="openConnection(conn)"
          @contextmenu="showContextMenu($event, conn)"
        >
          <span class="status-dot" :class="statusDot(conn)" />
          <span class="conn-name">{{ conn.name }}</span>
          <button class="action-btn sftp-btn" title="SFTP" @click.stop="emit('open-sftp', conn)">
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M2 4h5l2 2h5v7a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V4z" stroke="currentColor" stroke-width="1.3" fill="none"/>
            </svg>
          </button>
          <button class="action-btn connect-btn" title="Connect" @click.stop="openConnection(conn)">
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M5 3l7 5-7 5V3z" fill="currentColor"/>
            </svg>
          </button>
          <span class="conn-host">{{ conn.host }}</span>
        </div>
      </template>

      <!-- Empty state -->
      <div v-if="connectionStore.connections.length === 0" class="empty-state">
        <p>No connections yet</p>
        <button class="btn-primary" @click="showNewConnection = true">
          Add Connection
        </button>
      </div>
    </div>

    <!-- Context Menu -->
    <ContextMenu
      v-if="ctxMenu.show"
      :items="ctxMenuItems"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      @select="handleCtxAction"
      @close="ctxMenu.show = false"
    />

    <!-- New Connection Modal -->
    <ConnectionForm
      v-if="showNewConnection"
      @close="showNewConnection = false"
      @saved="showNewConnection = false"
    />

    <!-- Edit Connection Modal -->
    <ConnectionForm
      v-if="editingConnection"
      :edit-connection="editingConnection"
      @close="editingConnection = null"
      @saved="editingConnection = null"
    />

    <!-- Import SSH Config Modal -->
    <ImportSSHConfig
      v-if="showImport"
      @close="showImport = false"
      @saved="showImport = false"
    />

    <!-- Export Connections Modal -->
    <ExportConnections
      v-if="showExport"
      @close="showExport = false"
    />

    <!-- Footer buttons -->
    <div class="sidebar-footer">
      <button class="btn-footer" title="AI Assistant (⌘⇧A)" @click="emit('open-ai')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a4 4 0 0 1 4 4v1h1a3 3 0 0 1 3 3v1a3 3 0 0 1-3 3h-1v4a4 4 0 0 1-8 0v-4H7a3 3 0 0 1-3-3v-1a3 3 0 0 1 3-3h1V6a4 4 0 0 1 4-4z" />
          <circle cx="9" cy="10" r="1" fill="currentColor" />
          <circle cx="15" cy="10" r="1" fill="currentColor" />
          <path d="M9.5 15a3.5 3.5 0 0 0 5 0" />
        </svg>
      </button>
      <button class="btn-footer" title="SSH Keys (⌘⇧K)" @click="emit('open-keys')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
        </svg>
      </button>
      <button class="btn-footer" title="Settings (⌘,)" @click="emit('open-settings')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.sidebar-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.app-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.btn-icon {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 6px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  transition: all var(--transition-fast);
}

.btn-icon:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.search-box {
  position: relative;
  padding: 8px 12px;
}

.search-icon {
  position: absolute;
  left: 22px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 6px 8px 6px 30px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  outline: none;
  transition: border-color var(--transition-fast);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-input:focus {
  border-color: var(--border-focus);
}

.connection-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  font-weight: 500;
  cursor: default;
  user-select: none;
  list-style: none;
}

.section-icon {
  font-size: 12px;
}

.conn-count {
  margin-left: auto;
  font-size: 11px;
  color: var(--text-muted);
}

.group-section {
  margin: 0;
}

.connection-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px 6px 28px;
  cursor: pointer;
  transition: background var(--transition-fast);
  user-select: none;
}

.connection-item:hover {
  background: var(--bg-hover);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--text-muted);
}

.status-dot.connected {
  background: var(--success);
}

.status-dot.connecting {
  background: var(--accent);
  animation: pulse 1.5s infinite;
}

.status-dot.error,
.status-dot.disconnected {
  background: var(--error);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.conn-name {
  font-size: var(--font-size-md);
  color: var(--text-primary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.action-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 5px 6px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  opacity: 0;
  transition: all var(--transition-fast);
  flex-shrink: 0;
  min-width: 24px;
  min-height: 24px;
}

.connection-item:hover .action-btn { opacity: 0.7; }
.action-btn:hover { opacity: 1 !important; color: var(--accent); background: var(--bg-active); }

.conn-host {
  font-size: 11px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 16px;
  color: var(--text-muted);
}

.btn-primary {
  padding: 6px 16px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  transition: background var(--transition-fast);
}

.btn-primary:hover {
  background: var(--accent-hover);
}

.sidebar-footer {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 8px 12px;
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}

.btn-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.btn-footer:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

/* Drag-and-drop indicators */
.connection-item[draggable="true"] {
  cursor: grab;
}

.connection-item[draggable="true"]:active {
  cursor: grabbing;
  opacity: 0.6;
}

.drop-indicator-before {
  box-shadow: inset 0 2px 0 0 var(--accent);
}

.drop-indicator-after {
  box-shadow: inset 0 -2px 0 0 var(--accent);
}

.drop-target-group {
  background: rgba(137, 180, 250, 0.12);
  border-radius: var(--radius-sm);
}

.section-header[draggable="true"] {
  cursor: grab;
}

.section-header[draggable="true"]:active {
  cursor: grabbing;
  opacity: 0.6;
}
</style>
