<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { stat as localStat } from '@tauri-apps/plugin-fs'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type { RemoteFileEntry, LocalFileEntry, TransferItem, DragTransferPayload } from '@/types'
import { useSftpStore } from '@/stores/sftpStore'
import * as sftpService from '@/services/sftp'
import LocalFileBrowser from '@/components/sftp/LocalFileBrowser.vue'
import RemoteFileBrowser from '@/components/sftp/RemoteFileBrowser.vue'
import TransferQueue from '@/components/sftp/TransferQueue.vue'
import FileEditor from '@/components/sftp/FileEditor.vue'

const props = defineProps<{
  sftpSessionId: string
  connectionName: string
  connectionId?: string
}>()

const sftpStore = useSftpStore()
const dividerDragging = ref(false)
const leftPaneWidth = ref(50)
const remoteBrowserRef = ref<InstanceType<typeof RemoteFileBrowser>>()
const containerRef = ref<HTMLDivElement>()

// File editor state
const editingFile = ref<RemoteFileEntry | null>(null)
const showEditor = ref(false)

// OS drop listener
let unlistenFileDrop: UnlistenFn | null = null

// ============ Transfer helpers ============

function startTransferListeners(transferId: string, direction: 'upload' | 'download') {
  let unlistenProgress: (() => void) | null = null
  let unlistenComplete: (() => void) | null = null

  const setup = async () => {
    unlistenProgress = await sftpService.onTransferProgress(transferId, (data) => {
      sftpStore.updateTransferProgress(
        transferId,
        data.bytes_transferred,
        data.progress,
        data.total_bytes,
      )
    })

    unlistenComplete = await sftpService.onTransferComplete(transferId, () => {
      sftpStore.updateTransferStatus(transferId, 'completed')
      if (direction === 'upload') {
        remoteBrowserRef.value?.refresh()
      }
    })

    sftpStore.registerEventCleanup(transferId, () => {
      unlistenProgress?.()
      unlistenComplete?.()
    })
  }

  setup()
  return { direction }
}

// ============ Download ============

async function handleDownload(entry: RemoteFileEntry) {
  try {
    const savePath = await save({
      defaultPath: entry.name,
      title: `Save "${entry.name}"`,
    })
    if (!savePath) return

    await handleDownloadWithResume(entry.path, savePath, entry.name, entry.size)
  } catch (err) {
    console.error('Download failed:', err)
  }
}

async function executeDownload(
  remotePath: string,
  localPath: string,
  fileName: string,
  totalBytes: number,
  resumeOffset = 0,
) {
  const transferId = crypto.randomUUID()
  const controller = new AbortController()

  sftpStore.addTransfer({
    id: transferId,
    direction: 'download',
    localPath,
    remotePath,
    fileName,
    totalBytes,
    bytesTransferred: resumeOffset,
    progress: totalBytes > 0 ? (resumeOffset / totalBytes) * 100 : 0,
    status: 'pending',
    sftpSessionId: props.sftpSessionId,
  })

  sftpStore.registerAbortController(transferId, controller)
  startTransferListeners(transferId, 'download')

  const downloadFn = resumeOffset > 0
    ? sftpService.sftpDownloadWithResume(
        props.sftpSessionId, remotePath, localPath, transferId, resumeOffset,
      )
    : sftpService.sftpDownload(
        props.sftpSessionId, remotePath, localPath, transferId,
      )

  downloadFn.catch((err) => {
    if (controller.signal.aborted) return
    sftpStore.updateTransferStatus(transferId, 'failed', String(err))
  })
}

// ============ Upload ============

async function handleUpload() {
  try {
    const selected = await open({
      multiple: true,
      title: 'Select files to upload',
    })
    if (!selected) return

    const paths = Array.isArray(selected) ? selected : [selected]
    for (const filePath of paths) {
      const fileName = filePath.split(/[/\\]/).pop() ?? filePath
      const remoteDest = sftpStore.currentPath.endsWith('/')
        ? sftpStore.currentPath + fileName
        : sftpStore.currentPath + '/' + fileName

      await executeUpload(filePath, remoteDest, fileName)
    }
  } catch (err) {
    console.error('Upload failed:', err)
  }
}

async function executeUpload(
  localPath: string,
  remotePath: string,
  fileName: string,
  resumeOffset = 0,
) {
  let totalBytes = 0
  try {
    const info = await localStat(localPath)
    totalBytes = info.size ?? 0
  } catch { /* size unknown */ }

  const transferId = crypto.randomUUID()
  const controller = new AbortController()

  sftpStore.addTransfer({
    id: transferId,
    direction: 'upload',
    localPath,
    remotePath,
    fileName,
    totalBytes,
    bytesTransferred: resumeOffset,
    progress: totalBytes > 0 ? (resumeOffset / totalBytes) * 100 : 0,
    status: 'pending',
    sftpSessionId: props.sftpSessionId,
  })

  sftpStore.registerAbortController(transferId, controller)
  startTransferListeners(transferId, 'upload')

  const uploadFn = resumeOffset > 0
    ? sftpService.sftpUploadWithResume(
        props.sftpSessionId, localPath, remotePath, transferId, resumeOffset,
      )
    : sftpService.sftpUpload(
        props.sftpSessionId, localPath, remotePath, transferId,
      )

  uploadFn.catch((err) => {
    if (controller.signal.aborted) return
    sftpStore.updateTransferStatus(transferId, 'failed', String(err))
  })
}

// ============ Resume Transfer (断点续传) ============

async function handleUploadWithResume(localPath: string, remotePath: string, fileName: string) {
  let offset = 0
  try {
    const remoteStat = await sftpService.sftpStat(props.sftpSessionId, remotePath)
    if (!remoteStat.is_dir && remoteStat.size > 0) {
      const localInfo = await localStat(localPath)
      const localSize = localInfo.size ?? 0
      if (remoteStat.size < localSize) {
        offset = remoteStat.size
      }
    }
  } catch {
    // Remote file doesn't exist, start from 0
  }
  await executeUpload(localPath, remotePath, fileName, offset)
}

async function handleDownloadWithResume(remotePath: string, localPath: string, fileName: string, remoteSize: number) {
  let offset = 0
  try {
    const localInfo = await localStat(localPath)
    const localSize = localInfo.size ?? 0
    if (localSize > 0 && localSize < remoteSize) {
      offset = localSize
    }
  } catch {
    // Local file doesn't exist, start from 0
  }
  await executeDownload(remotePath, localPath, fileName, remoteSize, offset)
}

// ============ Upload from local browser ============

async function handleUploadFiles(files: LocalFileEntry[]) {
  for (const file of files) {
    if (file.is_dir) continue
    const remoteDest = sftpStore.currentPath.endsWith('/')
      ? sftpStore.currentPath + file.name
      : sftpStore.currentPath + '/' + file.name

    await handleUploadWithResume(file.path, remoteDest, file.name)
  }
}

// ============ Drag & Drop between panes ============

function handleRemoteDrop(payload: DragTransferPayload) {
  if (payload.source !== 'local') return
  for (const entry of payload.entries) {
    if (entry.is_dir) continue
    const remoteDest = sftpStore.currentPath.endsWith('/')
      ? sftpStore.currentPath + entry.name
      : sftpStore.currentPath + '/' + entry.name
    handleUploadWithResume(entry.path, remoteDest, entry.name)
  }
}

// ============ File Editor ============

function handleEditFile(entry: RemoteFileEntry) {
  editingFile.value = entry
  showEditor.value = true
}

function closeEditor() {
  showEditor.value = false
  editingFile.value = null
}

function handleEditorSaved() {
  remoteBrowserRef.value?.refresh()
}

// ============ Retry ============

function handleRetry(item: TransferItem) {
  const newTransfer = sftpStore.retryTransfer(item.id)
  if (!newTransfer) return

  const controller = new AbortController()
  sftpStore.registerAbortController(newTransfer.id, controller)
  startTransferListeners(newTransfer.id, newTransfer.direction)

  const invokeTransfer = newTransfer.direction === 'upload'
    ? sftpService.sftpUpload(
        newTransfer.sftpSessionId,
        newTransfer.localPath,
        newTransfer.remotePath,
        newTransfer.id,
      )
    : sftpService.sftpDownload(
        newTransfer.sftpSessionId,
        newTransfer.remotePath,
        newTransfer.localPath,
        newTransfer.id,
      )

  invokeTransfer.catch((err) => {
    if (controller.signal.aborted) return
    sftpStore.updateTransferStatus(newTransfer.id, 'failed', String(err))
  })
}

// ============ Resizable divider ============

function startResize(event: MouseEvent) {
  event.preventDefault()
  dividerDragging.value = true
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.body.style.webkitUserSelect = 'none'

  const onMouseMove = (e: MouseEvent) => {
    e.preventDefault()
    if (!containerRef.value) return
    const rect = containerRef.value.getBoundingClientRect()
    const pct = ((e.clientX - rect.left) / rect.width) * 100
    leftPaneWidth.value = Math.max(20, Math.min(80, pct))
  }

  const onMouseUp = () => {
    dividerDragging.value = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    document.body.style.webkitUserSelect = ''
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

// ============ OS file drop (drag from Finder/Explorer) ============

onMounted(async () => {
  try {
    unlistenFileDrop = await listen<{ paths: string[] }>('tauri://drag-drop', (event) => {
      const paths = event.payload.paths
      if (!paths || paths.length === 0) return

      for (const filePath of paths) {
        const fileName = filePath.split(/[/\\]/).pop() ?? filePath
        const remoteDest = sftpStore.currentPath.endsWith('/')
          ? sftpStore.currentPath + fileName
          : sftpStore.currentPath + '/' + fileName

        executeUpload(filePath, remoteDest, fileName)
      }
    })
  } catch {
    // OS drag-drop may not be available
  }
})

onBeforeUnmount(() => {
  unlistenFileDrop?.()
})
</script>

<template>
  <div class="sftp-view">
    <div class="sftp-header">
      <span class="sftp-title">SFTP — {{ connectionName }}</span>
      <button class="sftp-btn" @click="handleUpload">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M8 12V3m-4 4l4-4 4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        Upload
      </button>
      <button
        class="sftp-btn"
        @click="sftpStore.showTransferPanel = !sftpStore.showTransferPanel"
      >
        Transfers
        <span v-if="sftpStore.activeTransfers.length > 0" class="badge">
          {{ sftpStore.activeTransfers.length }}
        </span>
      </button>
    </div>

    <!-- Main body: dual pane or editor overlay -->
    <div class="sftp-body" ref="containerRef">
      <template v-if="showEditor && editingFile">
        <FileEditor
          :session-id="sftpSessionId"
          :remote-path="editingFile.path"
          :file-name="editingFile.name"
          :file-size="editingFile.size"
          @close="closeEditor"
          @saved="handleEditorSaved"
        />
      </template>

      <template v-else>
        <!-- Left pane: Local -->
        <div class="pane local-pane" :style="{ width: leftPaneWidth + '%' }">
          <div class="pane-label">LOCAL</div>
          <LocalFileBrowser
            @upload-files="handleUploadFiles"
          />
        </div>

        <!-- Resizable divider -->
        <div
          class="pane-divider"
          :class="{ dragging: dividerDragging }"
          @mousedown="startResize"
        >
          <div class="divider-grip" />
        </div>

        <!-- Right pane: Remote -->
        <div class="pane remote-pane" :style="{ width: (100 - leftPaneWidth) + '%' }">
          <div class="pane-label">REMOTE</div>
          <RemoteFileBrowser
            ref="remoteBrowserRef"
            :session-id="sftpSessionId"
            :connection-id="connectionId"
            @download="handleDownload"
            @edit-file="handleEditFile"
            @file-drop="handleRemoteDrop"
          />
        </div>

        <!-- Overlay during divider drag to prevent content interference -->
        <div v-if="dividerDragging" class="sftp-resize-overlay" />
      </template>
    </div>

    <TransferQueue @retry="handleRetry" />
  </div>
</template>

<style scoped>
.sftp-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.sftp-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.sftp-title {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.sftp-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 12px;
  transition: all var(--transition-fast);
}

.sftp-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.badge {
  background: var(--accent);
  color: var(--bg-primary);
  font-size: 10px;
  padding: 0 5px;
  border-radius: 10px;
  min-width: 16px;
  text-align: center;
}

.sftp-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
}

.pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.pane-label {
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.local-pane .pane-label { color: var(--success); }
.remote-pane .pane-label { color: var(--accent); }

.pane-divider {
  width: 5px;
  cursor: col-resize;
  background: var(--border-color);
  flex-shrink: 0;
  position: relative;
  transition: background var(--transition-fast);
}

.pane-divider:hover,
.pane-divider.dragging {
  background: var(--accent);
}

.divider-grip {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 3px;
  height: 24px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 2px;
}

.divider-grip::before,
.divider-grip::after {
  content: '';
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--text-muted);
}

.pane-divider:hover .divider-grip::before,
.pane-divider:hover .divider-grip::after,
.pane-divider.dragging .divider-grip::before,
.pane-divider.dragging .divider-grip::after {
  background: var(--bg-primary);
}

.sftp-resize-overlay {
  position: fixed;
  inset: 0;
  z-index: 9;
  cursor: col-resize;
}
</style>
