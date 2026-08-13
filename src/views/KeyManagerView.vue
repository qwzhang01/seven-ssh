<script setup lang="ts">
import { ref, onMounted } from 'vue'

import type { LocalKeyInfo } from '@/services/keygen'
import { useConnectionStore } from '@/stores/connectionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import * as keygenService from '@/services/keygen'

const emit = defineEmits<{
  close: []
}>()

const connectionStore = useConnectionStore()
const settingsStore = useSettingsStore()

const keys = ref<LocalKeyInfo[]>([])
const loading = ref(false)
const error = ref('')

const showGenerateDialog = ref(false)
const showDeployDialog = ref(false)
const deployKeyPath = ref('')

const genForm = ref({
  keyType: 'ed25519' as 'ed25519' | 'rsa' | 'ecdsa',
  bits: 4096,
  passphrase: '',
  comment: '',
  savePath: '',
})

const generating = ref(false)
const genResult = ref<keygenService.KeyGenResult | null>(null)

const deleteConfirm = ref<string | null>(null)

async function loadKeys() {
  loading.value = true
  error.value = ''
  try {
    keys.value = await keygenService.listLocalKeys()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

async function handleGenerate() {
  generating.value = true
  genResult.value = null
  try {
    const result = await keygenService.generateKeyPair(
      genForm.value.keyType,
      getBitsForType(),
      genForm.value.passphrase || undefined,
      genForm.value.comment || undefined,
      genForm.value.savePath || undefined,
    )
    genResult.value = result
    await keygenService.logSecurityEvent('key_generated', `Type: ${genForm.value.keyType}, Path: ${result.private_key_path}`)
    await loadKeys()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    generating.value = false
  }
}

function getBitsForType(): number | undefined {
  switch (genForm.value.keyType) {
    case 'rsa': return genForm.value.bits
    case 'ecdsa': return genForm.value.bits
    default: return undefined
  }
}

async function handleDelete(keyPath: string) {
  try {
    await keygenService.deleteKey(keyPath)
    deleteConfirm.value = null
    await loadKeys()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function copyPublicKey(key: LocalKeyInfo) {
  if (!key.has_public_key) return
  try {
    const keys = await keygenService.listLocalKeys()
    const found = keys.find(k => k.path === key.path)
    if (!found) return

    const response = await fetch(
      `asset://localhost/${encodeURIComponent(found.public_key_path)}`
    ).catch(() => null)

    if (response) {
      const text = await response.text()
      await navigator.clipboard.writeText(text)
    } else {
      await navigator.clipboard.writeText(found.fingerprint)
    }
  } catch {
    await navigator.clipboard.writeText(key.fingerprint)
  }
  keygenService.scheduleClipboardClear(settingsStore.clipboardTimeout)
}

function openDeployDialog(key: LocalKeyInfo) {
  deployKeyPath.value = key.public_key_path
  showDeployDialog.value = true
}

async function handleDeploy(connectionId: string) {
  try {
    await keygenService.deployPublicKey(connectionId, deployKeyPath.value)
    await keygenService.logSecurityEvent('key_deployed', `Key: ${deployKeyPath.value}, Connection: ${connectionId}`)
    showDeployDialog.value = false
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

function resetGenForm() {
  genForm.value = { keyType: 'ed25519', bits: 4096, passphrase: '', comment: '', savePath: '' }
  genResult.value = null
  showGenerateDialog.value = true
}

onMounted(loadKeys)
</script>

<template>
  <div class="key-manager">
    <div class="key-manager-header">
      <div class="header-left">
        <button class="btn-back" title="Back" @click="emit('close')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M19 12H5M12 19l-7-7 7-7" />
          </svg>
        </button>
        <h2 class="title">SSH Key Manager</h2>
      </div>
      <div class="header-actions">
        <button class="btn-primary" @click="resetGenForm">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          Generate New Key
        </button>
        <button class="btn-secondary" @click="loadKeys">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 11-2.12-9.36L23 10" />
          </svg>
          Refresh
        </button>
      </div>
    </div>

    <div v-if="error" class="error-banner">
      {{ error }}
      <button class="btn-dismiss" @click="error = ''">×</button>
    </div>

    <div v-if="loading" class="loading-state">
      Scanning ~/.ssh directory...
    </div>

    <div v-else-if="keys.length === 0" class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.5">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
        <path d="M7 11V7a5 5 0 0110 0v4" />
      </svg>
      <p>No SSH keys found in ~/.ssh</p>
      <button class="btn-primary" @click="resetGenForm">Generate Your First Key</button>
    </div>

    <div v-else class="key-table-wrapper">
      <table class="key-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Type</th>
            <th>Fingerprint</th>
            <th>Created</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="key in keys" :key="key.path">
            <td class="cell-name">
              <span class="key-name">{{ key.name }}</span>
              <span v-if="key.comment" class="key-comment">{{ key.comment }}</span>
            </td>
            <td>
              <span class="badge-type">{{ key.key_type }}</span>
              <span v-if="key.bits" class="badge-bits">{{ key.bits }}b</span>
            </td>
            <td class="cell-fingerprint">
              <code>{{ key.fingerprint || '—' }}</code>
            </td>
            <td class="cell-date">{{ key.created || '—' }}</td>
            <td class="cell-actions">
              <button
                v-if="key.has_public_key"
                class="btn-icon"
                title="Copy Public Key"
                @click="copyPublicKey(key)"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
                </svg>
              </button>
              <button
                v-if="key.has_public_key"
                class="btn-icon"
                title="Deploy to Server"
                @click="openDeployDialog(key)"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" />
                </svg>
              </button>
              <button
                class="btn-icon btn-danger"
                title="Delete Key"
                @click="deleteConfirm = key.path"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6" />
                  <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
                </svg>
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Delete Confirmation Dialog -->
    <div v-if="deleteConfirm" class="modal-overlay" @click.self="deleteConfirm = null">
      <div class="modal-dialog modal-small">
        <h3>Delete Key?</h3>
        <p>This will permanently delete the private key and its public key file. This action cannot be undone.</p>
        <div class="modal-actions">
          <button class="btn-secondary" @click="deleteConfirm = null">Cancel</button>
          <button class="btn-danger-solid" @click="handleDelete(deleteConfirm!)">Delete</button>
        </div>
      </div>
    </div>

    <!-- Generate Key Dialog -->
    <div v-if="showGenerateDialog" class="modal-overlay" @click.self="showGenerateDialog = false">
      <div class="modal-dialog">
        <h3>Generate SSH Key Pair</h3>

        <div v-if="genResult" class="gen-success">
          <div class="success-icon">✓</div>
          <p>Key pair generated successfully!</p>
          <div class="gen-result-details">
            <div class="detail-row">
              <span class="detail-label">Private Key:</span>
              <code>{{ genResult.private_key_path }}</code>
            </div>
            <div class="detail-row">
              <span class="detail-label">Public Key:</span>
              <code>{{ genResult.public_key_path }}</code>
            </div>
            <div class="detail-row">
              <span class="detail-label">Fingerprint:</span>
              <code>{{ genResult.fingerprint }}</code>
            </div>
          </div>
          <button class="btn-primary" @click="showGenerateDialog = false">Done</button>
        </div>

        <form v-else @submit.prevent="handleGenerate">
          <div class="form-group">
            <label>Key Type</label>
            <select v-model="genForm.keyType">
              <option value="ed25519">ED25519 (recommended)</option>
              <option value="rsa">RSA</option>
              <option value="ecdsa">ECDSA</option>
            </select>
          </div>

          <div v-if="genForm.keyType === 'rsa'" class="form-group">
            <label>Key Size (bits)</label>
            <select v-model.number="genForm.bits">
              <option :value="2048">2048</option>
              <option :value="4096">4096 (recommended)</option>
            </select>
          </div>

          <div v-if="genForm.keyType === 'ecdsa'" class="form-group">
            <label>Curve Size</label>
            <select v-model.number="genForm.bits">
              <option :value="256">NIST P-256</option>
              <option :value="384">NIST P-384</option>
              <option :value="521">NIST P-521</option>
            </select>
          </div>

          <div class="form-group">
            <label>Passphrase (optional)</label>
            <input
              v-model="genForm.passphrase"
              type="password"
              placeholder="Leave empty for no passphrase"
              autocomplete="new-password"
            />
          </div>

          <div class="form-group">
            <label>Comment (optional)</label>
            <input
              v-model="genForm.comment"
              type="text"
              placeholder="e.g. user@hostname"
            />
          </div>

          <div class="form-group">
            <label>Save Path (optional)</label>
            <input
              v-model="genForm.savePath"
              type="text"
              :placeholder="`~/.ssh/id_${genForm.keyType}`"
            />
          </div>

          <div class="modal-actions">
            <button type="button" class="btn-secondary" @click="showGenerateDialog = false">Cancel</button>
            <button type="submit" class="btn-primary" :disabled="generating">
              {{ generating ? 'Generating...' : 'Generate' }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Deploy to Server Dialog -->
    <div v-if="showDeployDialog" class="modal-overlay" @click.self="showDeployDialog = false">
      <div class="modal-dialog">
        <h3>Deploy Public Key to Server</h3>
        <p class="deploy-info">Select a server to deploy this public key to its <code>~/.ssh/authorized_keys</code>.</p>

        <div class="connection-list">
          <button
            v-for="conn in connectionStore.connections"
            :key="conn.id"
            class="connection-item"
            @click="handleDeploy(conn.id)"
          >
            <span class="conn-name">{{ conn.name }}</span>
            <span class="conn-host">{{ conn.username }}@{{ conn.host }}:{{ conn.port }}</span>
          </button>
          <p v-if="connectionStore.connections.length === 0" class="empty-connections">
            No connections configured. Add a connection first.
          </p>
        </div>

        <div class="modal-actions">
          <button class="btn-secondary" @click="showDeployDialog = false">Cancel</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.key-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  overflow: hidden;
}

.key-manager-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border-color);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-back {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
}
.btn-back:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background var(--transition-fast);
}
.btn-primary:hover {
  background: var(--accent-hover);
}
.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: var(--bg-surface);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.btn-secondary:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 24px;
  background: rgba(243, 139, 168, 0.1);
  color: var(--error);
  font-size: 12px;
  border-bottom: 1px solid rgba(243, 139, 168, 0.2);
}

.btn-dismiss {
  background: none;
  border: none;
  color: var(--error);
  cursor: pointer;
  font-size: 16px;
  padding: 0 4px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 12px;
  color: var(--text-muted);
  font-size: 13px;
}

.key-table-wrapper {
  flex: 1;
  overflow: auto;
  padding: 0 24px 24px;
}

.key-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  margin-top: 16px;
}

.key-table th {
  text-align: left;
  padding: 8px 12px;
  color: var(--text-muted);
  font-weight: 500;
  border-bottom: 1px solid var(--border-color);
  white-space: nowrap;
}

.key-table td {
  padding: 10px 12px;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-color);
  vertical-align: middle;
}

.key-table tr:hover td {
  background: var(--bg-hover);
}

.cell-name {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.key-name {
  font-weight: 500;
  font-family: var(--font-mono);
}

.key-comment {
  font-size: 11px;
  color: var(--text-muted);
}

.badge-type {
  display: inline-block;
  padding: 2px 6px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  color: var(--text-accent);
}

.badge-bits {
  display: inline-block;
  margin-left: 4px;
  padding: 2px 4px;
  font-size: 10px;
  color: var(--text-muted);
}

.cell-fingerprint code {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
  word-break: break-all;
}

.cell-date {
  white-space: nowrap;
  color: var(--text-muted);
  font-size: 11px;
}

.cell-actions {
  display: flex;
  gap: 4px;
  white-space: nowrap;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.btn-icon:hover {
  background: var(--bg-surface);
  color: var(--text-primary);
}
.btn-icon.btn-danger:hover {
  background: rgba(243, 139, 168, 0.1);
  color: var(--error);
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(2px);
}

.modal-dialog {
  width: 440px;
  max-height: 80vh;
  overflow-y: auto;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 24px;
}
.modal-dialog.modal-small {
  width: 360px;
}

.modal-dialog h3 {
  margin: 0 0 16px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-dialog p {
  margin: 0 0 16px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.btn-danger-solid {
  padding: 6px 14px;
  background: var(--error);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
}
.btn-danger-solid:hover {
  opacity: 0.9;
}

/* Form styles */
.form-group {
  margin-bottom: 14px;
}

.form-group label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 8px 10px;
  background: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  transition: border-color var(--transition-fast);
}
.form-group input:focus,
.form-group select:focus {
  border-color: var(--border-focus);
}

.form-group input::placeholder {
  color: var(--text-muted);
}

/* Generate success */
.gen-success {
  text-align: center;
}

.success-icon {
  width: 48px;
  height: 48px;
  margin: 0 auto 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(166, 227, 161, 0.15);
  color: var(--success);
  border-radius: 50%;
  font-size: 24px;
  font-weight: 600;
}

.gen-result-details {
  text-align: left;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 12px;
  margin: 16px 0;
}

.detail-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 8px;
}
.detail-row:last-child {
  margin-bottom: 0;
}

.detail-label {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
}

.detail-row code {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
  word-break: break-all;
}

/* Deploy dialog */
.deploy-info {
  margin-bottom: 12px;
}
.deploy-info code {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-accent);
}

.connection-list {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
}

.connection-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 14px;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background var(--transition-fast);
}
.connection-item:last-child {
  border-bottom: none;
}
.connection-item:hover {
  background: var(--bg-hover);
}

.conn-name {
  font-size: 13px;
  font-weight: 500;
}

.conn-host {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.empty-connections {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}
</style>
