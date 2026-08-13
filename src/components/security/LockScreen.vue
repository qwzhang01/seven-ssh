<script setup lang="ts">
import { ref, onMounted } from 'vue'
import * as securityService from '@/services/security'
import AppLogo from '@/components/common/AppLogo.vue'

const emit = defineEmits<{
  unlocked: []
}>()

const hasMaster = ref<boolean | null>(null)
const password = ref('')
const confirmPassword = ref('')
const error = ref('')
const loading = ref(false)
const mode = ref<'check' | 'setup' | 'unlock'>('check')

onMounted(async () => {
  try {
    const exists = await securityService.checkHasMasterPassword()
    if (exists) {
      mode.value = 'unlock'
    } else {
      mode.value = 'setup'
    }
    hasMaster.value = exists
  } catch {
    mode.value = 'setup'
    hasMaster.value = false
  }
})

async function handleSetup() {
  error.value = ''
  if (password.value.length < 6) {
    error.value = 'Password must be at least 6 characters'
    return
  }
  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }
  loading.value = true
  try {
    await securityService.setMasterPassword(password.value)
    emit('unlocked')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function handleUnlock() {
  error.value = ''
  if (!password.value) {
    error.value = 'Please enter your master password'
    return
  }
  loading.value = true
  try {
    const ok = await securityService.verifyMasterPassword(password.value)
    if (ok) {
      emit('unlocked')
    } else {
      error.value = 'Incorrect password'
      password.value = ''
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    if (mode.value === 'setup') handleSetup()
    else handleUnlock()
  }
}

async function skipSetup() {
  emit('unlocked')
}
</script>

<template>
  <div class="lock-screen" @keydown="handleKeydown">
    <div class="lock-card">
      <AppLogo :size="56" glow class="lock-logo" />

      <h1 class="lock-title">SevenSSH</h1>

      <!-- Loading state -->
      <div v-if="hasMaster === null" class="lock-loading">
        Loading...
      </div>

      <!-- Setup mode -->
      <template v-else-if="mode === 'setup'">
        <p class="lock-subtitle">Set a master password to encrypt your saved credentials</p>

        <div class="input-group">
          <input
            v-model="password"
            type="password"
            placeholder="Master password (6+ chars)"
            class="lock-input"
            autofocus
          />
        </div>

        <div class="input-group">
          <input
            v-model="confirmPassword"
            type="password"
            placeholder="Confirm password"
            class="lock-input"
          />
        </div>

        <p v-if="error" class="lock-error">{{ error }}</p>

        <button
          class="lock-btn primary"
          :disabled="loading"
          @click="handleSetup"
        >
          {{ loading ? 'Encrypting...' : 'Set Master Password' }}
        </button>

        <button class="lock-btn secondary" @click="skipSetup">
          Skip for now
        </button>
      </template>

      <!-- Unlock mode -->
      <template v-else>
        <p class="lock-subtitle">Enter your master password to unlock</p>

        <div class="input-group">
          <input
            v-model="password"
            type="password"
            placeholder="Master password"
            class="lock-input"
            autofocus
          />
        </div>

        <p v-if="error" class="lock-error">{{ error }}</p>

        <button
          class="lock-btn primary"
          :disabled="loading"
          @click="handleUnlock"
        >
          {{ loading ? 'Verifying...' : 'Unlock' }}
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.lock-screen {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  backdrop-filter: blur(20px);
}

.lock-card {
  width: 360px;
  padding: 40px 32px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  text-align: center;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.lock-logo {
  margin: 0 auto 16px;
}

.lock-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 8px;
  font-family: var(--font-ui);
}

.lock-subtitle {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0 0 24px;
  line-height: 1.5;
}

.lock-loading {
  font-size: 13px;
  color: var(--text-muted);
  padding: 20px 0;
}

.input-group {
  margin-bottom: 12px;
}

.lock-input {
  width: 100%;
  padding: 10px 14px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 14px;
  font-family: var(--font-ui);
  outline: none;
  transition: border-color var(--transition-fast);
  box-sizing: border-box;
}

.lock-input:focus {
  border-color: var(--accent);
}

.lock-input::placeholder {
  color: var(--text-muted);
}

.lock-error {
  color: var(--error);
  font-size: 12px;
  margin: 4px 0 12px;
}

.lock-btn {
  width: 100%;
  padding: 10px;
  border: none;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: all var(--transition-fast);
  margin-top: 8px;
}

.lock-btn.primary {
  background: var(--accent);
  color: var(--bg-tertiary);
  font-weight: 600;
}

.lock-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.lock-btn.primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.lock-btn.secondary {
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
}

.lock-btn.secondary:hover {
  color: var(--text-secondary);
}
</style>
