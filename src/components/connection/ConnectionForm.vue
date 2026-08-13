<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useConnectionStore } from '@/stores/connectionStore'
import type { AuthMethod, ConnectionInfo } from '@/types'

const props = defineProps<{
  editConnection?: ConnectionInfo
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const connectionStore = useConnectionStore()

const isEditing = computed(() => !!props.editConnection)

const form = ref({
  name: '',
  host: '',
  port: 22,
  username: 'root',
  auth_method: 'password' as AuthMethod,
  password: '',
  private_key_path: '',
  passphrase: '',
  group_id: '',
  note: '',
})

const saving = ref(false)
const error = ref('')

const isValid = computed(() =>
  form.value.name.trim() !== '' &&
  form.value.host.trim() !== '' &&
  form.value.username.trim() !== ''
)

onMounted(() => {
  if (props.editConnection) {
    const c = props.editConnection
    form.value = {
      name: c.name,
      host: c.host,
      port: c.port,
      username: c.username,
      auth_method: c.auth_method,
      password: '',
      private_key_path: c.private_key_path ?? '',
      passphrase: '',
      group_id: c.group_id ?? '',
      note: c.note ?? '',
    }
  }
})

async function handleSave() {
  if (!isValid.value) return
  saving.value = true
  error.value = ''

  try {
    if (isEditing.value && props.editConnection) {
      await connectionStore.updateConnection({
        id: props.editConnection.id,
        name: form.value.name,
        host: form.value.host,
        port: form.value.port,
        username: form.value.username,
        auth_method: form.value.auth_method,
        password: form.value.auth_method === 'password' && form.value.password ? form.value.password : undefined,
        private_key_path: form.value.auth_method === 'publickey' ? form.value.private_key_path : undefined,
        passphrase: form.value.passphrase || undefined,
        group_id: form.value.group_id || undefined,
        note: form.value.note || undefined,
      })
    } else {
      await connectionStore.addConnection({
        name: form.value.name,
        host: form.value.host,
        port: form.value.port,
        username: form.value.username,
        auth_method: form.value.auth_method,
        password: form.value.auth_method === 'password' ? form.value.password : undefined,
        private_key_path: form.value.auth_method === 'publickey' ? form.value.private_key_path : undefined,
        passphrase: form.value.passphrase || undefined,
        group_id: form.value.group_id || undefined,
        note: form.value.note || undefined,
      })
    }
    emit('saved')
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h3>{{ isEditing ? 'Edit Connection' : 'New Connection' }}</h3>
        <button class="btn-icon" @click="emit('close')">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label>Connection Name</label>
          <input v-model="form.name" type="text" placeholder="My Server" />
        </div>

        <div class="form-row">
          <div class="form-group flex-1">
            <label>Host</label>
            <input v-model="form.host" type="text" placeholder="192.168.1.100" />
          </div>
          <div class="form-group" style="width: 100px;">
            <label>Port</label>
            <input v-model.number="form.port" type="number" />
          </div>
        </div>

        <div class="form-group">
          <label>Username</label>
          <input v-model="form.username" type="text" placeholder="root" />
        </div>

        <div class="form-group">
          <label>Authentication</label>
          <select v-model="form.auth_method">
            <option value="password">Password</option>
            <option value="publickey">Public Key</option>
            <option value="agent">SSH Agent</option>
          </select>
        </div>

        <div v-if="form.auth_method === 'password'" class="form-group">
          <label>Password</label>
          <input
            v-model="form.password"
            type="password"
            :placeholder="isEditing ? 'Leave blank to keep unchanged' : 'Enter password'"
          />
        </div>

        <template v-if="form.auth_method === 'publickey'">
          <div class="form-group">
            <label>Private Key Path</label>
            <input v-model="form.private_key_path" type="text" placeholder="~/.ssh/id_rsa" />
          </div>
          <div class="form-group">
            <label>Passphrase (optional)</label>
            <input v-model="form.passphrase" type="password" placeholder="Key passphrase" />
          </div>
        </template>

        <div class="form-group">
          <label>Group</label>
          <select v-model="form.group_id">
            <option value="">No Group</option>
            <option v-for="group in connectionStore.groups" :key="group.id" :value="group.id">
              {{ group.name }}
            </option>
          </select>
        </div>

        <div class="form-group">
          <label>Note</label>
          <textarea v-model="form.note" rows="2" placeholder="Optional notes..." />
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" @click="emit('close')">Cancel</button>
        <button class="btn-primary" :disabled="!isValid || saving" @click="handleSave">
          {{ saving ? 'Saving...' : 'Save' }}
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

.modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  width: 480px;
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
  gap: 12px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.flex-1 {
  flex: 1;
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
.form-group select,
.form-group textarea {
  padding: 8px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  outline: none;
  transition: border-color var(--transition-fast);
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  border-color: var(--border-focus);
}

.form-group textarea {
  resize: vertical;
  font-family: inherit;
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
