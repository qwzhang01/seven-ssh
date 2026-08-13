<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'

import type { RemoteFileEntry, DragTransferPayload, SortField, SortOrder } from '@/types'
import { useSftpStore } from '@/stores/sftpStore'
import * as sftpService from '@/services/sftp'
import * as settingsService from '@/services/settings'
import ContextMenu from '@/components/common/ContextMenu.vue'
import type { MenuItem } from '@/components/common/ContextMenu.vue'

const props = defineProps<{
  sessionId: string
  connectionId?: string
}>()

const emit = defineEmits<{
  download: [entry: RemoteFileEntry]
  editFile: [entry: RemoteFileEntry]
  dragStart: [payload: DragTransferPayload]
  fileDrop: [payload: DragTransferPayload]
}>()

const sftpStore = useSftpStore()
const currentPath = ref('/')
const entries = ref<RemoteFileEntry[]>([])
const loading = ref(false)
const error = ref('')
const pathInput = ref('/')
const selectedEntries = ref<Set<string>>(new Set())
const showNewFolderInput = ref(false)
const newFolderName = ref('')
const renamingEntry = ref<string | null>(null)
const renameValue = ref('')

const sortField = ref<SortField>('name')
const sortOrder = ref<SortOrder>('asc')

// Drag & drop
const isDragOver = ref(false)

// Bookmarks
const bookmarks = ref<string[]>([])
const showBookmarks = ref(false)

// Context menu
const contextMenu = ref<{ show: boolean; x: number; y: number; entry: RemoteFileEntry | null }>({
  show: false,
  x: 0,
  y: 0,
  entry: null,
})

const bookmarkKey = computed(() => {
  return props.connectionId ? `sftp_bookmarks_${props.connectionId}` : ''
})

const sortedEntries = computed(() => {
  const sorted = [...entries.value]
  sorted.sort((a, b) => {
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1

    let cmp = 0
    switch (sortField.value) {
      case 'name':
        cmp = a.name.toLowerCase().localeCompare(b.name.toLowerCase())
        break
      case 'size':
        cmp = a.size - b.size
        break
      case 'date':
        cmp = (a.modified ?? 0) - (b.modified ?? 0)
        break
    }
    return sortOrder.value === 'asc' ? cmp : -cmp
  })
  return sorted
})

const contextMenuItems = computed<MenuItem[]>(() => {
  const entry = contextMenu.value.entry
  if (!entry) return []
  const items: MenuItem[] = []

  if (!entry.is_dir) {
    items.push({ id: 'edit', label: 'Edit File', icon: '📝' })
    items.push({ id: 'download', label: 'Download', icon: '⬇' })
    items.push({ id: 'divider1', label: '', divider: true })
  }

  items.push({ id: 'rename', label: 'Rename', icon: '✏️' })
  items.push({ id: 'delete', label: 'Delete', icon: '🗑', danger: true })
  return items
})

async function loadBookmarks() {
  if (!bookmarkKey.value) return
  try {
    const settings = await settingsService.getSettings()
    const raw = settings[bookmarkKey.value]
    if (raw) {
      bookmarks.value = JSON.parse(raw)
    }
  } catch {
    bookmarks.value = []
  }
}

async function saveBookmarks() {
  if (!bookmarkKey.value) return
  try {
    await settingsService.updateSetting(bookmarkKey.value, JSON.stringify(bookmarks.value))
  } catch (err) {
    console.error('Failed to save bookmarks:', err)
  }
}

function addBookmark() {
  if (!bookmarks.value.includes(currentPath.value)) {
    bookmarks.value.push(currentPath.value)
    saveBookmarks()
  }
}

function removeBookmark(path: string) {
  bookmarks.value = bookmarks.value.filter((b) => b !== path)
  saveBookmarks()
}

function navigateToBookmark(path: string) {
  showBookmarks.value = false
  loadDir(path)
}

async function loadDir(path: string) {
  loading.value = true
  error.value = ''
  selectedEntries.value.clear()

  try {
    const resolved = await sftpService.sftpRealpath(props.sessionId, path)
    currentPath.value = resolved
    pathInput.value = resolved
    sftpStore.setPath(resolved)
    entries.value = await sftpService.sftpListDir(props.sessionId, resolved)
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

function navigateUp() {
  const parts = currentPath.value.split('/').filter(Boolean)
  parts.pop()
  loadDir('/' + parts.join('/'))
}

function openEntry(entry: RemoteFileEntry) {
  if (entry.is_dir) {
    loadDir(entry.path)
  } else {
    emit('editFile', entry)
  }
}

function toggleSelect(entry: RemoteFileEntry, event: MouseEvent) {
  if (event.metaKey || event.ctrlKey) {
    if (selectedEntries.value.has(entry.path)) {
      selectedEntries.value.delete(entry.path)
    } else {
      selectedEntries.value.add(entry.path)
    }
  } else {
    selectedEntries.value.clear()
    selectedEntries.value.add(entry.path)
  }
}

function goToPath() {
  loadDir(pathInput.value)
}

function toggleSort(field: SortField) {
  if (sortField.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortField.value = field
    sortOrder.value = 'asc'
  }
}

function sortIndicator(field: SortField): string {
  if (sortField.value !== field) return ''
  return sortOrder.value === 'asc' ? ' ▲' : ' ▼'
}

async function createFolder() {
  if (!newFolderName.value.trim()) return
  try {
    const path = currentPath.value.endsWith('/')
      ? currentPath.value + newFolderName.value
      : currentPath.value + '/' + newFolderName.value
    await sftpService.sftpMkdir(props.sessionId, path)
    showNewFolderInput.value = false
    newFolderName.value = ''
    await loadDir(currentPath.value)
  } catch (err) {
    error.value = String(err)
  }
}

async function deleteSelected() {
  for (const path of selectedEntries.value) {
    const entry = entries.value.find((e) => e.path === path)
    if (!entry) continue
    try {
      await sftpService.sftpRemove(props.sessionId, path, entry.is_dir)
    } catch (err) {
      error.value = String(err)
    }
  }
  selectedEntries.value.clear()
  await loadDir(currentPath.value)
}

function startRename(entry: RemoteFileEntry) {
  renamingEntry.value = entry.path
  renameValue.value = entry.name
}

async function confirmRename(entry: RemoteFileEntry) {
  if (!renameValue.value.trim() || renameValue.value === entry.name) {
    renamingEntry.value = null
    return
  }
  try {
    const parentPath = currentPath.value.endsWith('/')
      ? currentPath.value
      : currentPath.value + '/'
    await sftpService.sftpRename(
      props.sessionId,
      entry.path,
      parentPath + renameValue.value,
    )
    renamingEntry.value = null
    await loadDir(currentPath.value)
  } catch (err) {
    error.value = String(err)
  }
}

function downloadEntry(entry: RemoteFileEntry) {
  emit('download', entry)
}

// Context menu
function showContextMenu(event: MouseEvent, entry: RemoteFileEntry) {
  event.preventDefault()
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    entry,
  }
}

function handleContextMenuSelect(id: string) {
  const entry = contextMenu.value.entry
  if (!entry) return
  contextMenu.value.show = false

  switch (id) {
    case 'edit':
      emit('editFile', entry)
      break
    case 'download':
      downloadEntry(entry)
      break
    case 'rename':
      startRename(entry)
      break
    case 'delete':
      selectedEntries.value.clear()
      selectedEntries.value.add(entry.path)
      deleteSelected()
      break
  }
}

// Drag & Drop
function onDragStart(event: DragEvent, entry: RemoteFileEntry) {
  if (!event.dataTransfer) return
  const selected = selectedEntries.value.has(entry.path)
    ? entries.value.filter((e) => selectedEntries.value.has(e.path))
    : [entry]

  const payload: DragTransferPayload = {
    source: 'remote',
    entries: selected.map((e) => ({
      name: e.name,
      path: e.path,
      is_dir: e.is_dir,
      size: e.size,
    })),
  }
  event.dataTransfer.setData('application/json', JSON.stringify(payload))
  event.dataTransfer.effectAllowed = 'copy'
  emit('dragStart', payload)
}

function onDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy'
  }
  isDragOver.value = true
}

function onDragLeave(event: DragEvent) {
  const target = event.currentTarget as HTMLElement
  const related = event.relatedTarget as Node | null
  if (target && related && target.contains(related)) return
  isDragOver.value = false
}

function onDrop(event: DragEvent) {
  event.preventDefault()
  isDragOver.value = false

  if (!event.dataTransfer) return

  const jsonData = event.dataTransfer.getData('application/json')
  if (jsonData) {
    try {
      const payload = JSON.parse(jsonData) as DragTransferPayload
      if (payload.source === 'local') {
        emit('fileDrop', payload)
      }
    } catch { /* ignore parse errors */ }
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '-'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let size = bytes
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024
    i++
  }
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`
}

function formatPermissions(perms: number): string {
  const rwx = (n: number) =>
    (n & 4 ? 'r' : '-') + (n & 2 ? 'w' : '-') + (n & 1 ? 'x' : '-')
  return rwx((perms >> 6) & 7) + rwx((perms >> 3) & 7) + rwx(perms & 7)
}

function formatDate(ts?: number): string {
  if (!ts) return '-'
  return new Date(ts * 1000).toLocaleDateString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function refresh() {
  loadDir(currentPath.value)
}

defineExpose({ refresh, currentPath })

watch(() => props.sessionId, () => loadDir('/'))
onMounted(() => {
  loadDir('/')
  loadBookmarks()
})
</script>

<template>
  <div
    class="file-browser"
    :class="{ 'drag-over': isDragOver }"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <div class="browser-toolbar">
      <button class="tool-btn" @click="navigateUp" title="Up">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M8 3v10M4 7l4-4 4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
      <button class="tool-btn" @click="refresh" title="Refresh">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M13.5 8a5.5 5.5 0 1 1-1.6-3.9M13.5 2v4h-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
      <input
        v-model="pathInput"
        class="path-input"
        @keydown.enter="goToPath"
        spellcheck="false"
      />

      <!-- Bookmark button -->
      <div class="bookmark-wrapper">
        <button
          class="tool-btn bookmark-btn"
          :class="{ active: bookmarks.includes(currentPath) }"
          @click="showBookmarks = !showBookmarks"
          title="Bookmarks"
        >★</button>
        <div v-if="showBookmarks" class="bookmark-dropdown">
          <div class="bookmark-header">
            <span>Bookmarks</span>
            <button class="bookmark-add-btn" @click="addBookmark" title="Bookmark current path">
              + Add Current
            </button>
          </div>
          <div v-if="bookmarks.length === 0" class="bookmark-empty">No bookmarks</div>
          <div
            v-for="bm in bookmarks"
            :key="bm"
            class="bookmark-item"
            @click="navigateToBookmark(bm)"
          >
            <span class="bookmark-path">{{ bm }}</span>
            <button
              class="bookmark-remove"
              @click.stop="removeBookmark(bm)"
              title="Remove"
            >✕</button>
          </div>
        </div>
      </div>

      <button class="tool-btn" @click="showNewFolderInput = !showNewFolderInput" title="New Folder">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M8 4v8M4 8h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
      <button
        class="tool-btn danger"
        :disabled="selectedEntries.size === 0"
        @click="deleteSelected"
        title="Delete"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M2 4h12M5 4V3a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v1m2 0v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h10z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>

    <!-- New folder input -->
    <div v-if="showNewFolderInput" class="new-folder-row">
      <input
        v-model="newFolderName"
        class="new-folder-input"
        placeholder="Folder name..."
        @keydown.enter="createFolder"
        @keydown.escape="showNewFolderInput = false"
      />
      <button class="tool-btn" @click="createFolder">OK</button>
    </div>

    <!-- Error -->
    <div v-if="error" class="browser-error">{{ error }}</div>

    <!-- Drop overlay -->
    <div v-if="isDragOver" class="drop-overlay">
      <div class="drop-label">Drop files here to upload</div>
    </div>

    <!-- File list -->
    <div class="file-list" :class="{ loading }">
      <div class="file-header">
        <span class="col-name sortable" @click="toggleSort('name')">
          Name{{ sortIndicator('name') }}
        </span>
        <span class="col-size sortable" @click="toggleSort('size')">
          Size{{ sortIndicator('size') }}
        </span>
        <span class="col-perm">Permissions</span>
        <span class="col-date sortable" @click="toggleSort('date')">
          Modified{{ sortIndicator('date') }}
        </span>
      </div>

      <div
        v-for="entry in sortedEntries"
        :key="entry.path"
        class="file-row"
        :class="{
          selected: selectedEntries.has(entry.path),
          directory: entry.is_dir,
        }"
        draggable="true"
        @click="toggleSelect(entry, $event)"
        @dblclick="openEntry(entry)"
        @contextmenu.prevent="showContextMenu($event, entry)"
        @dragstart="onDragStart($event, entry)"
      >
        <span class="col-name">
          <span class="file-icon">{{ entry.is_dir ? '📂' : '📄' }}</span>
          <template v-if="renamingEntry === entry.path">
            <input
              v-model="renameValue"
              class="rename-input"
              @keydown.enter="confirmRename(entry)"
              @keydown.escape="renamingEntry = null"
              @blur="confirmRename(entry)"
            />
          </template>
          <template v-else>
            <span class="file-name">{{ entry.name }}</span>
          </template>
        </span>
        <span class="col-size">{{ entry.is_dir ? '-' : formatSize(entry.size) }}</span>
        <span class="col-perm">{{ formatPermissions(entry.permissions) }}</span>
        <span class="col-date">{{ formatDate(entry.modified) }}</span>

        <button
          v-if="!entry.is_dir"
          class="row-action"
          title="Download"
          @click.stop="downloadEntry(entry)"
        >⬇</button>
      </div>

      <div v-if="!loading && entries.length === 0" class="empty-dir">
        Empty directory
      </div>
    </div>

    <!-- Context menu -->
    <ContextMenu
      v-if="contextMenu.show"
      :items="contextMenuItems"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @select="handleContextMenuSelect"
      @close="contextMenu.show = false"
    />
  </div>
</template>

<style scoped>
.file-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  position: relative;
}

.file-browser.drag-over {
  outline: 2px dashed var(--accent);
  outline-offset: -2px;
}

.drop-overlay {
  position: absolute;
  inset: 0;
  background: rgba(137, 180, 250, 0.08);
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drop-label {
  padding: 12px 24px;
  background: var(--bg-secondary);
  border: 2px dashed var(--accent);
  border-radius: var(--radius-md);
  color: var(--accent);
  font-size: 13px;
  font-weight: 500;
}

.browser-toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.tool-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 6px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  min-height: 28px;
}

.tool-btn:hover { color: var(--text-primary); background: var(--bg-hover); }
.tool-btn:disabled { opacity: 0.4; cursor: default; }
.tool-btn.danger:hover { color: var(--error); }

.bookmark-btn { font-size: 14px; }
.bookmark-btn.active { color: var(--warning); }

.bookmark-wrapper {
  position: relative;
}

.bookmark-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  width: 280px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  z-index: 100;
  overflow: hidden;
}

.bookmark-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-color);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.bookmark-add-btn {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  font-size: 11px;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}
.bookmark-add-btn:hover { background: var(--bg-hover); }

.bookmark-empty {
  padding: 12px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.bookmark-item {
  display: flex;
  align-items: center;
  padding: 6px 10px;
  cursor: pointer;
  transition: background var(--transition-fast);
}
.bookmark-item:hover { background: var(--bg-hover); }

.bookmark-path {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-remove {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 11px;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  opacity: 0;
  transition: all var(--transition-fast);
  min-width: 22px;
  min-height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.bookmark-item:hover .bookmark-remove { opacity: 1; }
.bookmark-remove:hover { color: var(--error); background: var(--bg-active); }

.path-input {
  flex: 1;
  padding: 4px 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  font-family: monospace;
  outline: none;
}

.path-input:focus { border-color: var(--border-focus); }

.new-folder-row {
  display: flex;
  gap: 4px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-color);
}

.new-folder-input {
  flex: 1;
  padding: 3px 6px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.browser-error {
  padding: 6px 8px;
  background: rgba(255, 80, 80, 0.1);
  color: var(--error);
  font-size: 12px;
  border-bottom: 1px solid var(--border-color);
}

.file-list {
  flex: 1;
  overflow-y: auto;
  font-size: 12px;
}

.file-list.loading { opacity: 0.5; }

.file-header {
  display: flex;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 500;
  color: var(--text-muted);
  position: sticky;
  top: 0;
  background: var(--bg-primary);
  z-index: 1;
}

.sortable {
  cursor: pointer;
  user-select: none;
}
.sortable:hover { color: var(--text-primary); }

.file-row {
  display: flex;
  align-items: center;
  padding: 3px 8px;
  cursor: default;
  position: relative;
  transition: background var(--transition-fast);
}

.file-row:hover { background: var(--bg-hover); }
.file-row.selected { background: rgba(137, 180, 250, 0.15); }
.file-row[draggable="true"] { cursor: grab; }

.col-name {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  overflow: hidden;
}

.file-icon { flex-shrink: 0; font-size: 14px; }

.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-primary);
}

.directory .file-name { color: var(--accent); }

.rename-input {
  flex: 1;
  padding: 1px 4px;
  background: var(--bg-surface);
  border: 1px solid var(--accent);
  border-radius: 2px;
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.col-size { width: 70px; text-align: right; color: var(--text-muted); flex-shrink: 0; }
.col-perm { width: 85px; text-align: center; color: var(--text-muted); font-family: monospace; flex-shrink: 0; }
.col-date { width: 100px; text-align: right; color: var(--text-muted); flex-shrink: 0; }

.row-action {
  position: absolute;
  right: 6px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  opacity: 0;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  min-width: 24px;
  min-height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.file-row:hover .row-action { opacity: 1; }
.row-action:hover { background: var(--bg-hover); color: var(--accent); }

.empty-dir {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
}
</style>
