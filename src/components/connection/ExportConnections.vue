<script setup lang="ts">
import { ref, computed } from 'vue'
import { useConnectionStore } from '@/stores/connectionStore'
import { exportConnections, type ExportFormat } from '@/services/export'

const emit = defineEmits<{
  close: []
}>()

const connectionStore = useConnectionStore()

const format = ref<ExportFormat>('json')
const selectAll = ref(true)
const selectedIds = ref<Set<string>>(new Set(connectionStore.connections.map((c) => c.id)))
const exporting = ref(false)
const error = ref('')
const exportedContent = ref('')

const allSelected = computed(() =>
  connectionStore.connections.length > 0 && selectedIds.value.size === connectionStore.connections.length
)

function toggleAll() {
  if (allSelected.value) {
    selectedIds.value = new Set()
    selectAll.value = false
  } else {
    selectedIds.value = new Set(connectionStore.connections.map((c) => c.id))
    selectAll.value = true
  }
}

function toggleConnection(id: string) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) {
    next.delete(id)
  } else {
    next.add(id)
  }
  selectedIds.value = next
}

async function handleExport() {
  if (selectedIds.value.size === 0) return
  exporting.value = true
  error.value = ''
  exportedContent.value = ''

  try {
    const ids = allSelected.value ? undefined : Array.from(selectedIds.value)
    const content = await exportConnections(format.value, ids)
    exportedContent.value = content
  } catch (e) {
    error.value = String(e)
  } finally {
    exporting.value = false
  }
}

async function copyToClipboard() {
  try {
    await navigator.clipboard.writeText(exportedContent.value)
  } catch {
    // Fallback: select all in textarea
  }
}

function downloadFile() {
  const ext = format.value === 'json' ? 'json' : 'txt'
  const filename = `sevenssh-export.${ext}`
  const blob = new Blob([exportedContent.value], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal export-modal">
      <div class="modal-header">
        <h3>Export Connections</h3>
        <button class="btn-icon" @click="emit('close')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label>Export Format</label>
          <div class="format-options">
            <label class="format-option" :class="{ active: format === 'json' }">
              <input type="radio" v-model="format" value="json" />
              <span class="format-label">JSON</span>
              <span class="format-desc">Portable format for backup/transfer</span>
            </label>
            <label class="format-option" :class="{ active: format === 'ssh_config' }">
              <input type="radio" v-model="format" value="ssh_config" />
              <span class="format-label">SSH Config</span>
              <span class="format-desc">Standard ~/.ssh/config format</span>
            </label>
          </div>
        </div>

        <div class="form-group">
          <div class="select-header">
            <label class="checkbox-label">
              <input type="checkbox" :checked="allSelected" @change="toggleAll" />
              <span>Connections ({{ selectedIds.size }}/{{ connectionStore.connections.length }})</span>
            </label>
          </div>
          <div class="connection-list-wrap">
            <div
              v-for="conn in connectionStore.connections"
              :key="conn.id"
              class="conn-row"
              :class="{ selected: selectedIds.has(conn.id) }"
              @click="toggleConnection(conn.id)"
            >
              <input type="checkbox" :checked="selectedIds.has(conn.id)" @click.stop="toggleConnection(conn.id)" />
              <span class="conn-row-name">{{ conn.name }}</span>
              <span class="conn-row-host">{{ conn.host }}</span>
            </div>
          </div>
        </div>

        <div v-if="exportedContent" class="export-result">
          <div class="result-actions">
            <button class="btn-sm" @click="copyToClipboard">Copy</button>
            <button class="btn-sm" @click="downloadFile">Download</button>
          </div>
          <textarea class="export-preview" readonly :value="exportedContent" />
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" @click="emit('close')">Close</button>
        <button
          class="btn-primary"
          :disabled="selectedIds.size === 0 || exporting"
          @click="handleExport"
        >
          {{ exporting ? 'Exporting...' : 'Export' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.export-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  font-size: 15px;
  font-weight: 600;
}

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group > label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  font-weight: 500;
}

.format-options {
  display: flex;
  gap: 8px;
}

.format-option {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.format-option input {
  display: none;
}

.format-option:hover {
  border-color: var(--text-muted);
}

.format-option.active {
  border-color: var(--accent);
  background: rgba(137, 180, 250, 0.06);
}

.format-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.format-desc {
  font-size: 11px;
  color: var(--text-muted);
}

.select-header {
  display: flex;
  align-items: center;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
}

.checkbox-label input[type="checkbox"] {
  accent-color: var(--accent);
}

.connection-list-wrap {
  max-height: 180px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
}

.conn-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  cursor: pointer;
  transition: background var(--transition-fast);
  border-bottom: 1px solid color-mix(in srgb, var(--border-color) 50%, transparent);
}

.conn-row:last-child {
  border-bottom: none;
}

.conn-row:hover {
  background: var(--bg-hover);
}

.conn-row.selected {
  background: rgba(137, 180, 250, 0.06);
}

.conn-row input[type="checkbox"] {
  accent-color: var(--accent);
  flex-shrink: 0;
}

.conn-row-name {
  font-size: 13px;
  color: var(--text-primary);
  flex: 1;
}

.conn-row-host {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.export-result {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.result-actions {
  display: flex;
  gap: 6px;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-sm:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.export-preview {
  width: 100%;
  height: 140px;
  padding: 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
  resize: vertical;
  outline: none;
}

.error-msg {
  color: var(--error);
  font-size: var(--font-size-sm);
  padding: 8px;
  background: rgba(243, 139, 168, 0.1);
  border-radius: var(--radius-sm);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
}

.btn-icon {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 6px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
}

.btn-icon:hover {
  color: var(--text-primary);
}

.btn-primary {
  padding: 8px 20px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 8px 20px;
  background: var(--bg-surface);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
}

.btn-secondary:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
