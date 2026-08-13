<script setup lang="ts">
import type { DangerWarning } from '@/services/ai'

const props = defineProps<{
  warning: DangerWarning
}>()

const emit = defineEmits<{
  proceed: []
  cancel: []
}>()

const isCritical = props.warning.level === 'critical'
</script>

<template>
  <div class="danger-banner" :class="{ critical: isCritical }">
    <div class="danger-content">
      <div class="danger-icon">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
      </div>
      <div class="danger-text">
        <div class="danger-level">{{ isCritical ? 'CRITICAL' : 'WARNING' }}: Dangerous Command Detected</div>
        <div class="danger-message">{{ warning.message }}</div>
      </div>
    </div>
    <div class="danger-actions">
      <button class="btn-cancel" @click="emit('cancel')">Cancel</button>
      <button class="btn-proceed" @click="emit('proceed')">Proceed Anyway</button>
    </div>
  </div>
</template>

<style scoped>
.danger-banner {
  padding: 10px 14px;
  background: color-mix(in srgb, var(--warning, #f59e0b) 12%, var(--bg-primary));
  border: 1px solid color-mix(in srgb, var(--warning, #f59e0b) 50%, transparent);
  border-radius: var(--radius-md);
  margin: 4px 8px;
}

.danger-banner.critical {
  background: color-mix(in srgb, var(--error, #ef4444) 12%, var(--bg-primary));
  border-color: color-mix(in srgb, var(--error, #ef4444) 50%, transparent);
}

.danger-content {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
}

.danger-icon {
  flex-shrink: 0;
  color: var(--warning, #f59e0b);
}

.critical .danger-icon {
  color: var(--error, #ef4444);
}

.danger-text {
  flex: 1;
  min-width: 0;
}

.danger-level {
  font-size: 12px;
  font-weight: 700;
  color: var(--warning, #f59e0b);
  margin-bottom: 4px;
}

.critical .danger-level {
  color: var(--error, #ef4444);
}

.danger-message {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}

.danger-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn-cancel {
  padding: 4px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
}

.btn-cancel:hover {
  background: var(--bg-hover);
}

.btn-proceed {
  padding: 4px 12px;
  background: color-mix(in srgb, var(--error, #ef4444) 20%, transparent);
  border: 1px solid var(--error, #ef4444);
  border-radius: var(--radius-sm);
  color: var(--error, #ef4444);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
}

.btn-proceed:hover {
  background: color-mix(in srgb, var(--error, #ef4444) 30%, transparent);
}
</style>
