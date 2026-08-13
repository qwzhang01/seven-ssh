import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { TransferItem, TransferStatus } from '@/types'

export const useSftpStore = defineStore('sftp', () => {
  const activeSftpSessionId = ref<string | null>(null)
  const currentPath = ref('/')
  const transfers = ref<TransferItem[]>([])
  const showTransferPanel = ref(false)

  const abortControllers = new Map<string, AbortController>()
  const eventCleanups = new Map<string, () => void>()

  const activeTransfers = computed(() =>
    transfers.value.filter((t) => t.status === 'active' || t.status === 'pending')
  )

  const completedTransfers = computed(() =>
    transfers.value.filter(
      (t) => t.status === 'completed' || t.status === 'failed' || t.status === 'cancelled'
    )
  )

  const pausedTransfers = computed(() =>
    transfers.value.filter((t) => t.status === 'paused')
  )

  const transferCounts = computed(() => ({
    active: transfers.value.filter((t) => t.status === 'active').length,
    pending: transfers.value.filter((t) => t.status === 'pending').length,
    paused: transfers.value.filter((t) => t.status === 'paused').length,
    completed: transfers.value.filter((t) => t.status === 'completed').length,
    failed: transfers.value.filter((t) => t.status === 'failed').length,
    cancelled: transfers.value.filter((t) => t.status === 'cancelled').length,
    total: transfers.value.length,
  }))

  function addTransfer(item: TransferItem) {
    transfers.value.unshift({ ...item, startedAt: Date.now() })
    showTransferPanel.value = true
  }

  function registerAbortController(transferId: string, controller: AbortController) {
    abortControllers.set(transferId, controller)
  }

  function registerEventCleanup(transferId: string, cleanup: () => void) {
    eventCleanups.set(transferId, cleanup)
  }

  function updateTransferProgress(
    transferId: string,
    bytesTransferred: number,
    progress: number,
    totalBytes?: number,
  ) {
    const item = transfers.value.find((t) => t.id === transferId)
    if (!item || item.status === 'cancelled' || item.status === 'paused') return

    const now = Date.now()
    const elapsed = item.startedAt ? (now - item.startedAt) / 1000 : 0
    item.speed = elapsed > 0 ? bytesTransferred / elapsed : 0
    item.bytesTransferred = bytesTransferred
    item.progress = progress
    if (totalBytes && totalBytes > 0) item.totalBytes = totalBytes
    item.status = 'active'
  }

  function updateTransferStatus(transferId: string, status: TransferStatus, error?: string) {
    const item = transfers.value.find((t) => t.id === transferId)
    if (item) {
      item.status = status
      if (status === 'completed') {
        item.progress = 100
        item.speed = 0
      }
      if (error) item.error = error
      if (status === 'completed' || status === 'failed' || status === 'cancelled') {
        cleanupTransfer(transferId)
      }
    }
  }

  function cancelTransfer(transferId: string) {
    const controller = abortControllers.get(transferId)
    if (controller) controller.abort()
    updateTransferStatus(transferId, 'cancelled')
  }

  function pauseTransfer(transferId: string) {
    const item = transfers.value.find((t) => t.id === transferId)
    if (item && item.status === 'active') {
      item.status = 'paused'
      item.speed = 0
    }
  }

  function resumeTransfer(transferId: string) {
    const item = transfers.value.find((t) => t.id === transferId)
    if (item && item.status === 'paused') {
      item.status = 'active'
      item.startedAt = Date.now()
    }
  }

  function retryTransfer(transferId: string): TransferItem | null {
    const item = transfers.value.find((t) => t.id === transferId)
    if (!item || (item.status !== 'failed' && item.status !== 'cancelled')) return null

    const newTransfer: TransferItem = {
      ...item,
      id: crypto.randomUUID(),
      status: 'pending',
      bytesTransferred: 0,
      progress: 0,
      speed: 0,
      error: undefined,
      startedAt: Date.now(),
    }

    removeTransfer(transferId)
    transfers.value.unshift(newTransfer)
    return newTransfer
  }

  function removeTransfer(transferId: string) {
    cleanupTransfer(transferId)
    transfers.value = transfers.value.filter((t) => t.id !== transferId)
  }

  function clearCompleted() {
    const removedIds = transfers.value
      .filter((t) => t.status === 'completed' || t.status === 'failed' || t.status === 'cancelled')
      .map((t) => t.id)

    for (const id of removedIds) cleanupTransfer(id)

    transfers.value = transfers.value.filter(
      (t) => t.status === 'active' || t.status === 'pending' || t.status === 'paused'
    )
  }

  function cleanupTransfer(transferId: string) {
    abortControllers.delete(transferId)
    const cleanup = eventCleanups.get(transferId)
    if (cleanup) {
      cleanup()
      eventCleanups.delete(transferId)
    }
  }

  function setPath(path: string) {
    currentPath.value = path
  }

  return {
    activeSftpSessionId,
    currentPath,
    transfers,
    showTransferPanel,
    activeTransfers,
    completedTransfers,
    pausedTransfers,
    transferCounts,
    addTransfer,
    registerAbortController,
    registerEventCleanup,
    updateTransferProgress,
    updateTransferStatus,
    cancelTransfer,
    pauseTransfer,
    resumeTransfer,
    retryTransfer,
    removeTransfer,
    clearCompleted,
    setPath,
  }
})
