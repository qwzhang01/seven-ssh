<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { readDir, stat } from '@tauri-apps/plugin-fs'
import { homeDir, resolve as resolvePath } from '@tauri-apps/api/path'

import type { LocalFileEntry, SortField, SortOrder, DragTransferPayload } from '@/types'

const emit = defineEmits<{
  uploadFiles: [files: LocalFileEntry[]]
  dragStart: [payload: DragTransferPayload]
}>()

const currentPath = ref('')
const entries = ref<LocalFileEntry[]>([])
const loading = ref(false)
const error = ref('')
const pathInput = ref('')
const selectedEntries = ref<Set<string>>(new Set())
const sortField = ref<SortField>('name')
const sortOrder = ref<SortOrder>('asc')
const showHidden = ref(false)

const breadcrumbs = computed(() => {
  const parts = currentPath.value.split('/').filter(Boolean)
  const crumbs: Array<{ name: string; path: string }> = [{ name: '/', path: '/' }]
  let accumulated = ''
  for (const part of parts) {
    accumulated += '/' + part
    crumbs.push({ name: part, path: accumulated })
  }
  return crumbs
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

const selectedCount = computed(() => selectedEntries.value.size)

async function loadDir(path: string) {
  loading.value = true
  error.value = ''
  selectedEntries.value.clear()

  try {
    const resolved = await resolvePath(path)
    currentPath.value = resolved
    pathInput.value = resolved

    const dirEntries = await readDir(resolved)
    const results: LocalFileEntry[] = []

    for (const entry of dirEntries) {
      if (!showHidden.value && entry.name?.startsWith('.')) continue
      try {
        const fullPath = resolved.endsWith('/')
          ? resolved + entry.name
          : resolved + '/' + entry.name
        const info = await stat(fullPath)
        results.push({
          name: entry.name ?? '',
          path: fullPath,
          is_dir: entry.isDirectory ?? false,
          size: info.size ?? 0,
          modified: info.mtime ? Math.floor(info.mtime.getTime() / 1000) : undefined,
        })
      } catch {
        results.push({
          name: entry.name ?? '',
          path: resolved + '/' + entry.name,
          is_dir: entry.isDirectory ?? false,
          size: 0,
        })
      }
    }

    entries.value = results
  } catch (err) {
    const msg = String(err)
    if (msg.includes('not allowed') || msg.includes('Permissions')) {
      error.value = `Permission denied: cannot read "${path}". Try navigating to your home directory.`
    } else {
      error.value = msg.length > 200 ? msg.slice(0, 200) + '…' : msg
    }
  } finally {
    loading.value = false
  }
}

function navigateUp() {
  const parts = currentPath.value.split('/').filter(Boolean)
  parts.pop()
  loadDir('/' + parts.join('/') || '/')
}

function openEntry(entry: LocalFileEntry) {
  if (entry.is_dir) {
    loadDir(entry.path)
  }
}

function toggleSelect(entry: LocalFileEntry, event: MouseEvent) {
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

async function goHome() {
  try {
    const home = await homeDir()
    loadDir(home)
  } catch {
    loadDir('/')
  }
}

function navigateToCrumb(path: string) {
  loadDir(path)
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

function uploadSelected() {
  const selected = entries.value.filter((e) => selectedEntries.value.has(e.path) && !e.is_dir)
  if (selected.length > 0) {
    emit('uploadFiles', selected)
  }
}

function onDragStart(event: DragEvent, entry: LocalFileEntry) {
  if (!event.dataTransfer) return
  const selected = selectedEntries.value.has(entry.path)
    ? entries.value.filter((e) => selectedEntries.value.has(e.path))
    : [entry]

  const payload: DragTransferPayload = {
    source: 'local',
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

function formatDate(ts?: number): string {
  if (!ts) return '-'
  return new Date(ts * 1000).toLocaleDateString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

onMounted(async () => {
  try {
    const home = await homeDir()
    loadDir(home)
  } catch {
    loadDir('/')
  }
})
</script>

<template>
  <div class="file-browser local-browser">
    <div class="browser-toolbar">
      <button class="tool-btn" @click="navigateUp" title="Up">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M8 3v10M4 7l4-4 4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
      <button class="tool-btn" @click="loadDir(currentPath)" title="Refresh">
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
      <button
        class="tool-btn"
        :class="{ active: showHidden }"
        @click="showHidden = !showHidden; loadDir(currentPath)"
        title="Toggle hidden files"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.5"/>
          <path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" fill="none"/>
        </svg>
      </button>
      <button
        class="tool-btn upload-btn"
        :disabled="selectedCount === 0"
        @click="uploadSelected"
        title="Upload selected files"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M8 12V3m-4 4l4-4 4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>

    <!-- Breadcrumbs -->
    <div class="breadcrumbs">
      <span
        v-for="(crumb, i) in breadcrumbs"
        :key="crumb.path"
        class="crumb"
        @click="navigateToCrumb(crumb.path)"
      >
        <span class="crumb-name">{{ crumb.name }}</span>
        <span v-if="i < breadcrumbs.length - 1" class="crumb-sep">/</span>
      </span>
    </div>

    <!-- Error -->
    <div v-if="error" class="browser-error">
      <span class="error-text">{{ error }}</span>
      <button class="error-home-btn" @click="goHome" title="Go to Home Directory">⌂ Home</button>
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
        @dragstart="onDragStart($event, entry)"
      >
        <span class="col-name">
          <span class="file-icon">{{ entry.is_dir ? '📂' : '📄' }}</span>
          <span class="file-name">{{ entry.name }}</span>
        </span>
        <span class="col-size">{{ entry.is_dir ? '-' : formatSize(entry.size) }}</span>
        <span class="col-date">{{ formatDate(entry.modified) }}</span>
      </div>

      <div v-if="!loading && entries.length === 0" class="empty-dir">
        Empty directory
      </div>
    </div>

    <div v-if="selectedCount > 0" class="selection-bar">
      {{ selectedCount }} selected
      <button class="upload-selected-btn" @click="uploadSelected">
        Upload to Remote →
      </button>
    </div>
  </div>
</template>

<style scoped>
.file-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
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
.tool-btn.active { color: var(--accent); }
.tool-btn.upload-btn:hover:not(:disabled) { color: var(--success); }

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

.breadcrumbs {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  min-height: 24px;
  align-items: center;
}

.crumb {
  display: flex;
  align-items: center;
  gap: 2px;
}

.crumb-name {
  color: var(--text-secondary);
  cursor: pointer;
  padding: 1px 3px;
  border-radius: 2px;
}

.crumb-name:hover {
  color: var(--accent);
  background: var(--bg-hover);
}

.crumb-sep {
  color: var(--text-muted);
}

.browser-error {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  background: rgba(255, 80, 80, 0.1);
  color: var(--error);
  font-size: 12px;
  border-bottom: 1px solid var(--border-color);
}

.error-text {
  flex: 1;
  min-width: 0;
}

.error-home-btn {
  flex-shrink: 0;
  padding: 2px 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  white-space: nowrap;
  transition: all var(--transition-fast);
}

.error-home-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
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

.col-size { width: 70px; text-align: right; color: var(--text-muted); flex-shrink: 0; }
.col-date { width: 100px; text-align: right; color: var(--text-muted); flex-shrink: 0; }

.empty-dir {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
}

.selection-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.upload-selected-btn {
  padding: 3px 10px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.upload-selected-btn:hover {
  background: var(--accent-hover);
}
</style>
