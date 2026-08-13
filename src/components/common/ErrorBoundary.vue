<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const hasError = ref(false)
const errorMessage = ref('')
const showContent = ref(true)

onErrorCaptured((err) => {
  hasError.value = true
  errorMessage.value = err instanceof Error ? err.message : String(err)
  console.error('[ErrorBoundary]', err)
  return false
})

function retry() {
  hasError.value = false
  errorMessage.value = ''
  showContent.value = false
  requestAnimationFrame(() => {
    showContent.value = true
  })
}

async function copyError() {
  try {
    await navigator.clipboard.writeText(errorMessage.value)
  } catch {
    // fallback: ignore
  }
}
</script>

<template>
  <div v-if="hasError" class="error-boundary">
    <div class="error-card">
      <svg class="error-icon" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <circle cx="12" cy="12" r="10" />
        <path d="M12 8v4M12 16h.01" />
      </svg>
      <h3 class="error-title">{{ t('common.errorBoundary') }}</h3>
      <p class="error-desc">{{ t('common.errorBoundaryDesc') }}</p>
      <p class="error-message">{{ errorMessage }}</p>
      <div class="error-actions">
        <button class="btn-retry" @click="retry">{{ t('common.tryAgain') }}</button>
        <button class="btn-report" @click="copyError">{{ t('common.report') }}</button>
      </div>
    </div>
  </div>
  <slot v-else-if="showContent" />
</template>

<style scoped>
.error-boundary {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  min-height: 120px;
  padding: 24px;
}

.error-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 32px 24px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  max-width: 400px;
  box-shadow: var(--shadow-md);
}

.error-icon {
  color: var(--error);
  margin-bottom: 12px;
}

.error-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.error-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.error-message {
  font-size: 12px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  background: var(--bg-primary);
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 16px;
}

.error-actions {
  display: flex;
  gap: 8px;
}

.btn-retry {
  padding: 8px 16px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  font-size: 13px;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.btn-retry:hover {
  background: var(--accent-hover);
}

.btn-report {
  padding: 8px 16px;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-report:hover {
  background: var(--bg-hover);
  border-color: var(--text-muted);
}
</style>
