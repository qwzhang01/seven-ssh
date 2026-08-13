<script setup lang="ts">
import { ref, computed } from 'vue'
import { useConnectionStore } from '@/stores/connectionStore'
import { importSshConfig, importPuttySessions, importXshellSessions, saveImportedConnections } from '@/services/import'
import type { ImportedConnection, ImportedConnectionSave } from '@/types'

const emit = defineEmits<{
  close: []
  saved: []
}>()

const connectionStore = useConnectionStore()

type ImportSource = 'ssh' | 'putty' | 'xshell'
const activeSource = ref<ImportSource>('ssh')

const filePath = ref('~/.ssh/config')
const xshellPath = ref('')
const scanning = ref(false)
const saving = ref(false)
const error = ref('')
const successMsg = ref('')
const discovered = ref<ImportedConnection[]>([])
const selected = ref<Set<number>>(new Set())
const groupId = ref('')

const hasResults = computed(() => discovered.value.length > 0)
const selectedCount = computed(() => selected.value.size)
const allSelected = computed(() =>
  discovered.value.length > 0 && selected.value.size === discovered.value.length
)

function switchSource(source: ImportSource) {
  activeSource.value = source
  discovered.value = []
  selected.value = new Set()
  error.value = ''
  successMsg.value = ''
}

function toggleAll() {
  if (allSelected.value) {
    selected.value = new Set()
  } else {
    selected.value = new Set(discovered.value.map((_, i) => i))
  }
}

function toggleRow(index: number) {
  const next = new Set(selected.value)
  if (next.has(index)) {
    next.delete(index)
  } else {
    next.add(index)
  }
  selected.value = next
}

async function handleScan() {
  scanning.value = true
  error.value = ''
  successMsg.value = ''
  discovered.value = []
  selected.value = new Set()

  try {
    switch (activeSource.value) {
      case 'ssh': {
        const path = filePath.value.trim() || undefined
        discovered.value = await importSshConfig(path)
        break
      }
      case 'putty': {
        discovered.value = await importPuttySessions()
        break
      }
      case 'xshell': {
        if (!xshellPath.value.trim()) {
          error.value = 'Please specify the path to Xshell session file or folder.'
          scanning.value = false
          return
        }
        discovered.value = await importXshellSessions(xshellPath.value.trim())
        break
      }
    }
    if (discovered.value.length === 0) {
      error.value = 'No hosts found from the selected source.'
    } else {
      selected.value = new Set(discovered.value.map((_, i) => i))
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    scanning.value = false
  }
}

async function handleImport() {
  if (selectedCount.value === 0) return
  saving.value = true
  error.value = ''
  successMsg.value = ''

  try {
    const toSave: ImportedConnectionSave[] = []
    for (const idx of selected.value) {
      const item = discovered.value[idx]
      toSave.push({
        name: item.host_alias,
        host: item.hostname,
        port: item.port,
        username: item.username,
        auth_method: item.identity_file ? 'publickey' : 'password',
        private_key_path: item.identity_file,
        group_id: groupId.value || undefined,
      })
    }

    const count = await saveImportedConnections(toSave)
    successMsg.value = `Successfully imported ${count} connection${count !== 1 ? 's' : ''}.`
    await connectionStore.fetchConnections()
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal import-modal">
      <div class="modal-header">
        <h3>Import Connections</h3>
        <button class="btn-icon" @click="emit('close')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>

      <div class="source-tabs">
        <button
          class="source-tab"
          :class="{ active: activeSource === 'ssh' }"
          @click="switchSource('ssh')"
        >SSH Config</button>
        <button
          class="source-tab"
          :class="{ active: activeSource === 'putty' }"
          @click="switchSource('putty')"
        >PuTTY</button>
        <button
          class="source-tab"
          :class="{ active: activeSource === 'xshell' }"
          @click="switchSource('xshell')"
        >Xshell</button>
      </div>

      <div class="modal-body">
        <!-- SSH Config source -->
        <div v-if="activeSource === 'ssh'" class="form-group">
          <label>SSH Config File</label>
          <div class="input-row">
            <input
              v-model="filePath"
              type="text"
              placeholder="~/.ssh/config"
              class="flex-1"
            />
            <button class="btn-secondary" :disabled="scanning" @click="handleScan">
              {{ scanning ? 'Scanning...' : 'Scan' }}
            </button>
          </div>
        </div>

        <!-- PuTTY source -->
        <div v-if="activeSource === 'putty'" class="form-group">
          <label>PuTTY Sessions</label>
          <p class="source-desc">
            On Windows, sessions are read from the registry. On macOS/Linux, sessions are loaded from ~/.putty/sessions/.
          </p>
          <button class="btn-secondary" :disabled="scanning" @click="handleScan">
            {{ scanning ? 'Scanning...' : 'Scan PuTTY Sessions' }}
          </button>
        </div>

        <!-- Xshell source -->
        <div v-if="activeSource === 'xshell'" class="form-group">
          <label>Xshell Session File/Folder</label>
          <div class="input-row">
            <input
              v-model="xshellPath"
              type="text"
              placeholder="/path/to/sessions/ or file.xsh"
              class="flex-1"
            />
            <button class="btn-secondary" :disabled="scanning" @click="handleScan">
              {{ scanning ? 'Scanning...' : 'Scan' }}
            </button>
          </div>
          <p class="source-desc">Point to a .xsh file or a folder containing .xsh session files.</p>
        </div>

        <div v-if="hasResults" class="results-section">
          <div class="results-header">
            <label class="checkbox-label">
              <input type="checkbox" :checked="allSelected" @change="toggleAll" />
              <span>Select All ({{ discovered.length }} found)</span>
            </label>
          </div>

          <div class="results-table-wrap">
            <table class="results-table">
              <thead>
                <tr>
                  <th class="col-check"></th>
                  <th>Alias</th>
                  <th>Host</th>
                  <th>Port</th>
                  <th>User</th>
                  <th>Auth</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(conn, idx) in discovered"
                  :key="idx"
                  :class="{ 'row-selected': selected.has(idx) }"
                  @click="toggleRow(idx)"
                >
                  <td class="col-check">
                    <input type="checkbox" :checked="selected.has(idx)" @click.stop="toggleRow(idx)" />
                  </td>
                  <td class="cell-alias">{{ conn.host_alias }}</td>
                  <td class="cell-host">{{ conn.hostname }}</td>
                  <td class="cell-port">{{ conn.port }}</td>
                  <td class="cell-user">{{ conn.username }}</td>
                  <td class="cell-auth">{{ conn.identity_file ? 'Key' : 'Password' }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="form-group">
            <label>Assign to Group (optional)</label>
            <select v-model="groupId">
              <option value="">No Group</option>
              <option v-for="group in connectionStore.groups" :key="group.id" :value="group.id">
                {{ group.name }}
              </option>
            </select>
          </div>
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>
        <div v-if="successMsg" class="success-msg">{{ successMsg }}</div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" @click="emit('close')">
          {{ successMsg ? 'Done' : 'Cancel' }}
        </button>
        <button
          v-if="hasResults && !successMsg"
          class="btn-primary"
          :disabled="selectedCount === 0 || saving"
          @click="handleImport"
        >
          {{ saving ? 'Importing...' : `Import ${selectedCount} Selected` }}
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

.import-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  width: 600px;
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

.source-tabs {
  display: flex;
  padding: 0 20px;
  border-bottom: 1px solid var(--border-color);
}

.source-tab {
  padding: 10px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.source-tab:hover {
  color: var(--text-primary);
}

.source-tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
  font-weight: 500;
}

.source-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin: 4px 0 8px;
  line-height: 1.4;
}

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  font-weight: 500;
}

.form-group input,
.form-group select {
  padding: 8px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  outline: none;
  transition: border-color var(--transition-fast);
}

.form-group input:focus,
.form-group select:focus {
  border-color: var(--border-focus);
}

.input-row {
  display: flex;
  gap: 8px;
}

.flex-1 {
  flex: 1;
}

.results-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.results-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
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

.results-table-wrap {
  max-height: 240px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
}

.results-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--font-size-sm);
}

.results-table th {
  position: sticky;
  top: 0;
  background: var(--bg-surface);
  text-align: left;
  padding: 6px 10px;
  color: var(--text-secondary);
  font-weight: 500;
  border-bottom: 1px solid var(--border-color);
}

.results-table td {
  padding: 6px 10px;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-color);
}

.results-table tbody tr {
  cursor: pointer;
  transition: background var(--transition-fast);
}

.results-table tbody tr:hover {
  background: var(--bg-hover);
}

.results-table tbody tr:last-child td {
  border-bottom: none;
}

.row-selected {
  background: rgba(137, 180, 250, 0.08);
}

.col-check {
  width: 32px;
  text-align: center;
}

.col-check input[type="checkbox"] {
  accent-color: var(--accent);
}

.cell-alias {
  font-weight: 500;
}

.cell-host {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
}

.cell-port {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 12px;
}

.cell-user {
  color: var(--text-secondary);
}

.cell-auth {
  color: var(--text-muted);
  font-size: 11px;
}

.error-msg {
  color: var(--error);
  font-size: var(--font-size-sm);
  padding: 8px;
  background: rgba(243, 139, 168, 0.1);
  border-radius: var(--radius-sm);
}

.success-msg {
  color: var(--success);
  font-size: var(--font-size-sm);
  padding: 8px;
  background: rgba(166, 227, 161, 0.1);
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

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
