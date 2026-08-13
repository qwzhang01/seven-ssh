import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { SessionTab, TerminalPane, ConnectionStatus } from '@/types'
import * as sshService from '@/services/ssh'

export const useSessionStore = defineStore('session', () => {
  const tabs = ref<SessionTab[]>([])
  const activeTabId = ref<string | null>(null)
  const activePaneId = ref<string | null>(null)

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeTabId.value) ?? null
  )

  const activePane = computed(() => {
    if (!activeTab.value) return null
    return activeTab.value.panes.find((p) => p.id === activePaneId.value)
      ?? activeTab.value.panes[0] ?? null
  })

  function addTab(pane: TerminalPane) {
    const tabId = crypto.randomUUID()
    const tab: SessionTab = {
      id: tabId,
      panes: [pane],
      splitDirection: 'none',
      syncInput: false,
    }
    tabs.value.push(tab)
    activeTabId.value = tabId
    activePaneId.value = pane.id
  }

  function removeTab(tabId: string) {
    const idx = tabs.value.findIndex((t) => t.id === tabId)
    if (idx === -1) return

    // Disconnect all panes' SSH sessions before removing
    const tab = tabs.value[idx]
    for (const pane of tab.panes) {
      if (pane.sessionId) {
        sshService.sshDisconnect(pane.sessionId)
      }
    }

    tabs.value.splice(idx, 1)
    if (activeTabId.value === tabId) {
      if (tabs.value.length > 0) {
        const newIdx = Math.min(idx, tabs.value.length - 1)
        activeTabId.value = tabs.value[newIdx].id
        activePaneId.value = tabs.value[newIdx].panes[0]?.id ?? null
      } else {
        activeTabId.value = null
        activePaneId.value = null
      }
    }
  }

  function setActiveTab(tabId: string) {
    activeTabId.value = tabId
    const tab = tabs.value.find((t) => t.id === tabId)
    if (tab && tab.panes.length > 0) {
      if (!tab.panes.find((p) => p.id === activePaneId.value)) {
        activePaneId.value = tab.panes[0].id
      }
    }
  }

  function setActivePane(paneId: string) {
    activePaneId.value = paneId
  }

  function splitPane(direction: 'horizontal' | 'vertical') {
    const tab = activeTab.value
    if (!tab) return
    if (tab.panes.length >= 4) return

    const sourcePane = activePane.value
    if (!sourcePane) return

    const newPane: TerminalPane = {
      id: crypto.randomUUID(),
      connectionId: sourcePane.connectionId,
      connectionName: sourcePane.connectionName,
      status: 'idle',
      host: sourcePane.host,
    }

    tab.panes.push(newPane)

    if (tab.panes.length === 2) {
      tab.splitDirection = direction
    } else if (tab.panes.length > 2) {
      tab.splitDirection = 'grid'
    }

    activePaneId.value = newPane.id
  }

  function removePane(paneId: string) {
    const tab = activeTab.value
    if (!tab) return
    if (tab.panes.length <= 1) return

    const idx = tab.panes.findIndex((p) => p.id === paneId)
    if (idx === -1) return

    tab.panes.splice(idx, 1)

    if (tab.panes.length === 1) {
      tab.splitDirection = 'none'
    } else if (tab.panes.length === 2) {
      if (tab.splitDirection === 'grid') {
        tab.splitDirection = 'horizontal'
      }
    }

    if (activePaneId.value === paneId) {
      activePaneId.value = tab.panes[0]?.id ?? null
    }
  }

  function toggleSyncInput() {
    const tab = activeTab.value
    if (tab) tab.syncInput = !tab.syncInput
  }

  function updatePaneStatus(paneId: string, status: ConnectionStatus) {
    for (const tab of tabs.value) {
      const pane = tab.panes.find((p) => p.id === paneId)
      if (pane) { pane.status = status; break }
    }
  }

  function updatePaneSessionId(paneId: string, sessionId: string) {
    for (const tab of tabs.value) {
      const pane = tab.panes.find((p) => p.id === paneId)
      if (pane) { pane.sessionId = sessionId; break }
    }
  }

  function getAllPanesInTab(tabId: string): TerminalPane[] {
    const tab = tabs.value.find((t) => t.id === tabId)
    return tab?.panes ?? []
  }

  // Legacy compatibility: get first pane's info for tab-level display
  function getTabDisplayInfo(tabId: string) {
    const tab = tabs.value.find((t) => t.id === tabId)
    if (!tab || tab.panes.length === 0) return null
    const firstPane = tab.panes[0]
    const anyConnected = tab.panes.some((p) => p.status === 'connected')
    return {
      connectionName: firstPane.connectionName,
      status: anyConnected ? 'connected' : firstPane.status,
      host: firstPane.host,
      paneCount: tab.panes.length,
    }
  }

  return {
    tabs,
    activeTabId,
    activePaneId,
    activeTab,
    activePane,
    addTab,
    removeTab,
    setActiveTab,
    setActivePane,
    splitPane,
    removePane,
    toggleSyncInput,
    updatePaneStatus,
    updatePaneSessionId,
    getAllPanesInTab,
    getTabDisplayInfo,
  }
})
