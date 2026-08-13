<script setup lang="ts">
import { useSftpStore } from '@/stores/sftpStore'

const sftpStore = useSftpStore()

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  let size = bytes
  while (size >= 1024 && i < units.length - 1) { size /= 1024; i++ }
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`
}

function formatSpeed(bytesPerSec: number): string {
  if (!bytesPerSec || bytesPerSec <= 0) return ''
  return `${formatSize(bytesPerSec)}/s`
}

function directionLabel(dir: string): string {
  return dir === 'upload' ? '↑' : '↓'
}

function statusLabel(status: string): string {
  switch (status) {
    case 'active': return 'Transferring'
    case 'completed': return 'Completed'
    case 'failed': return 'Failed'
    case 'cancelled': return 'Cancelled'
    case 'pending': return 'Pending'
    case 'paused': return 'Paused'
    default: return status
  }
}
</script>

<template>
  <div class="transfer-queue" v-if="sftpStore.showTransferPanel">
    <div class="queue-header">
      <span class="queue-title">
        Transfers
        <span class="queue-badge" v-if="sftpStore.transferCounts.active + sftpStore.transferCounts.pending > 0">
          {{ sftpStore.transferCounts.active + sftpStore.transferCounts.pending }}
        </span>
      </span>
      <button
        v-if="sftpStore.completedTransfers.length > 0"
        class="queue-action-btn"
        @click="sftpStore.clearCompleted()"
        title="Clear completed, failed, and cancelled transfers"
      >Clear Done</button>
      <button class="queue-close-btn" @click="sftpStore.showTransferPanel = false">✕</button>
    </div>

    <div class="queue-list">
      <div
        v-for="item in sftpStore.transfers"
        :key="item.id"
        class="transfer-item"
        :class="[item.status, item.direction]"
      >
        <span class="transfer-direction" :title="item.direction">
          {{ directionLabel(item.direction) }}
        </span>

        <div class="transfer-body">
          <div class="transfer-top-row">
            <span class="transfer-name" :title="item.remotePath">{{ item.fileName }}</span>
            <span class="transfer-status-label" :class="item.status">
              {{ statusLabel(item.status) }}
            </span>
          </div>

          <div class="transfer-progress-bar" v-if="item.status === 'active' || item.status === 'paused'">
            <div
              class="progress-fill"
              :class="{ paused: item.status === 'paused' }"
              :style="{ width: item.progress + '%' }"
            />
          </div>

          <div class="transfer-bottom-row">
            <span class="transfer-size">
              {{ formatSize(item.bytesTransferred) }}
              <template v-if="item.totalBytes > 0"> / {{ formatSize(item.totalBytes) }}</template>
              <template v-if="item.status === 'active'">
                — {{ item.progress.toFixed(0) }}%
              </template>
            </span>
            <span class="transfer-speed" v-if="item.status === 'active' && item.speed">
              {{ formatSpeed(item.speed) }}
            </span>
            <span class="transfer-error" v-if="item.status === 'failed' && item.error">
              {{ item.error }}
            </span>
          </div>
        </div>

        <div class="transfer-actions">
          <!-- Active: Pause + Cancel -->
          <template v-if="item.status === 'active'">
            <button
              class="action-btn pause"
              @click="sftpStore.pauseTransfer(item.id)"
              title="Pause"
            >⏸</button>
            <button
              class="action-btn cancel"
              @click="sftpStore.cancelTransfer(item.id)"
              title="Cancel"
            >✕</button>
          </template>

          <!-- Pending: Cancel -->
          <template v-if="item.status === 'pending'">
            <button
              class="action-btn cancel"
              @click="sftpStore.cancelTransfer(item.id)"
              title="Cancel"
            >✕</button>
          </template>

          <!-- Paused: Resume + Cancel -->
          <template v-if="item.status === 'paused'">
            <button
              class="action-btn resume"
              @click="sftpStore.resumeTransfer(item.id)"
              title="Resume"
            >▶</button>
            <button
              class="action-btn cancel"
              @click="sftpStore.cancelTransfer(item.id)"
              title="Cancel"
            >✕</button>
          </template>

          <!-- Failed: Retry + Remove -->
          <template v-if="item.status === 'failed'">
            <button
              class="action-btn retry"
              @click="$emit('retry', item)"
              title="Retry"
            >↻</button>
            <button
              class="action-btn remove"
              @click="sftpStore.removeTransfer(item.id)"
              title="Remove"
            >✕</button>
          </template>

          <!-- Completed: Remove -->
          <template v-if="item.status === 'completed'">
            <button
              class="action-btn remove"
              @click="sftpStore.removeTransfer(item.id)"
              title="Remove"
            >✕</button>
          </template>

          <!-- Cancelled: Remove -->
          <template v-if="item.status === 'cancelled'">
            <button
              class="action-btn retry"
              @click="$emit('retry', item)"
              title="Retry"
            >↻</button>
            <button
              class="action-btn remove"
              @click="sftpStore.removeTransfer(item.id)"
              title="Remove"
            >✕</button>
          </template>
        </div>
      </div>

      <div v-if="sftpStore.transfers.length === 0" class="queue-empty">
        No transfers
      </div>
    </div>
  </div>
</template>

<style scoped>
.transfer-queue {
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  max-height: 240px;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.queue-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--border-color);
}

.queue-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
}

.queue-badge {
  background: var(--accent);
  color: var(--bg-primary);
  font-size: 10px;
  font-weight: 600;
  padding: 0 5px;
  border-radius: 10px;
  min-width: 16px;
  text-align: center;
  line-height: 16px;
}

.queue-action-btn {
  background: none;
  border: 1px solid var(--border-color);
  color: var(--text-muted);
  cursor: pointer;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.queue-action-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
  border-color: var(--text-muted);
}

.queue-close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 13px;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  line-height: 1;
  min-width: 24px;
  min-height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.queue-close-btn:hover { color: var(--text-primary); background: var(--bg-hover); }

.queue-list {
  flex: 1;
  overflow-y: auto;
  padding: 2px 0;
}

.transfer-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 10px;
  transition: background var(--transition-fast);
}

.transfer-item:hover { background: var(--bg-hover); }

.transfer-direction {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  margin-top: 1px;
}

.transfer-item.upload .transfer-direction {
  color: var(--success);
  background: rgba(166, 227, 161, 0.1);
}

.transfer-item.download .transfer-direction {
  color: var(--accent);
  background: rgba(137, 180, 250, 0.1);
}

.transfer-body {
  flex: 1;
  min-width: 0;
}

.transfer-top-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.transfer-name {
  font-size: 12px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.transfer-status-label {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
  text-transform: uppercase;
  font-weight: 500;
  letter-spacing: 0.3px;
}

.transfer-status-label.active { color: var(--accent); background: rgba(137, 180, 250, 0.12); }
.transfer-status-label.pending { color: var(--text-muted); background: var(--bg-surface); }
.transfer-status-label.paused { color: var(--warning); background: rgba(249, 226, 175, 0.12); }
.transfer-status-label.completed { color: var(--success); background: rgba(166, 227, 161, 0.12); }
.transfer-status-label.failed { color: var(--error); background: rgba(243, 139, 168, 0.12); }
.transfer-status-label.cancelled { color: var(--text-muted); background: var(--bg-surface); }

.transfer-progress-bar {
  height: 3px;
  background: var(--bg-surface);
  border-radius: 2px;
  margin: 4px 0 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-fill.paused { background: var(--warning); }

.transfer-bottom-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.transfer-size {
  font-size: 11px;
  color: var(--text-muted);
}

.transfer-speed {
  font-size: 11px;
  color: var(--accent);
  font-family: var(--font-mono);
}

.transfer-error {
  font-size: 11px;
  color: var(--error);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.transfer-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.transfer-item:hover .transfer-actions { opacity: 1; }

.action-btn {
  background: none;
  border: none;
  cursor: pointer;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  font-size: 11px;
  color: var(--text-muted);
  transition: all var(--transition-fast);
}

.action-btn:hover { background: var(--bg-active); }

.action-btn.pause:hover { color: var(--warning); }
.action-btn.resume:hover { color: var(--success); }
.action-btn.retry:hover { color: var(--accent); }
.action-btn.cancel:hover { color: var(--error); }
.action-btn.remove:hover { color: var(--error); }

.transfer-item.completed .transfer-name { color: var(--text-secondary); }
.transfer-item.failed .transfer-name { color: var(--error); }
.transfer-item.cancelled .transfer-name { color: var(--text-muted); text-decoration: line-through; }

.queue-empty {
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}
</style>
