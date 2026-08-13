<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { SearchAddon } from '@xterm/addon-search'
import { WebLinksAddon } from '@xterm/addon-web-links'
import '@xterm/xterm/css/xterm.css'

import type { TerminalPane } from '@/types'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { getTheme } from '@/styles/themes'
import * as sshService from '@/services/ssh'
import ContextMenu from '@/components/common/ContextMenu.vue'
import type { MenuItem } from '@/components/common/ContextMenu.vue'

const props = defineProps<{
  pane: TerminalPane
  isActive: boolean
  syncInput: boolean
  siblingPanes: TerminalPane[]
}>()

const emit = defineEmits<{
  focus: []
  close: []
}>()

const sessionStore = useSessionStore()
const settingsStore = useSettingsStore()

const terminalRef = ref<HTMLDivElement>()
const showSearch = ref(false)
const searchText = ref('')
const searchInputRef = ref<HTMLInputElement>()
const isDisconnected = ref(false)
const reconnecting = ref(false)
const reconnectAttempts = ref(0)

const MAX_RECONNECT_RETRIES = 3
const RECONNECT_DELAY_MS = 3000

// Context menu state
const ctxMenu = ref<{ show: boolean; x: number; y: number }>({ show: false, x: 0, y: 0 })
const hasSelection = ref(false)

let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let searchAddon: SearchAddon | null = null
let unlistenOutput: (() => void) | null = null
let unlistenStatus: (() => void) | null = null
let resizeObserver: ResizeObserver | null = null
let reconnectTimer: ReturnType<typeof setTimeout> | null = null

const currentTheme = computed(() => getTheme(settingsStore.terminalTheme))

const termCtxItems = computed<MenuItem[]>(() => [
  { id: 'copy', label: 'Copy', icon: '📋', disabled: !hasSelection.value },
  { id: 'paste', label: 'Paste', icon: '📌' },
  { id: 'divider-1', label: '', divider: true },
  { id: 'select-all', label: 'Select All', icon: '☰' },
  { id: 'clear', label: 'Clear Terminal', icon: '🧹' },
  { id: 'divider-2', label: '', divider: true },
  { id: 'search', label: 'Search (⌘F)', icon: '🔍' },
])

async function handleCtxAction(id: string) {
  switch (id) {
    case 'copy': {
      const sel = terminal?.getSelection()
      if (sel) await navigator.clipboard.writeText(sel).catch(() => {})
      break
    }
    case 'paste': {
      try {
        const text = await navigator.clipboard.readText()
        if (text && props.pane.sessionId) {
          writeToSession(text)
        }
      } catch { /* clipboard denied */ }
      break
    }
    case 'select-all':
      terminal?.selectAll()
      break
    case 'clear':
      terminal?.clear()
      break
    case 'search':
      toggleSearch()
      break
  }
  terminal?.focus()
}

function writeToSession(text: string) {
  const encoder = new TextEncoder()
  const bytes = Array.from(encoder.encode(text))
  if (props.syncInput && props.siblingPanes.length > 0) {
    for (const sibling of props.siblingPanes) {
      if (sibling.sessionId) sshService.sshWrite(sibling.sessionId, bytes)
    }
  } else if (props.pane.sessionId) {
    sshService.sshWrite(props.pane.sessionId, bytes)
  }
}

async function initTerminal() {
  if (!terminalRef.value) return

  terminal = new Terminal({
    theme: currentTheme.value.theme,
    fontFamily: settingsStore.fontFamily,
    fontSize: settingsStore.fontSize,
    lineHeight: 1.2,
    cursorBlink: settingsStore.cursorBlink,
    cursorStyle: settingsStore.cursorStyle,
    scrollback: settingsStore.scrollback,
    allowProposedApi: true,
  })

  fitAddon = new FitAddon()
  searchAddon = new SearchAddon()
  const webLinksAddon = new WebLinksAddon()

  terminal.loadAddon(fitAddon)
  terminal.loadAddon(searchAddon)
  terminal.loadAddon(webLinksAddon)

  terminal.open(terminalRef.value)
  fitAddon.fit()

  resizeObserver = new ResizeObserver(() => {
    fitAddon?.fit()
  })
  resizeObserver.observe(terminalRef.value)

  terminal.writeln('\x1b[36m● Connecting...\x1b[0m')
  sessionStore.updatePaneStatus(props.pane.id, 'connecting')

  try {
    const sessionId = await sshService.sshConnect(props.pane.connectionId)
    sessionStore.updatePaneSessionId(props.pane.id, sessionId)
    sessionStore.updatePaneStatus(props.pane.id, 'connected')
    terminal.clear()

    unlistenOutput = await sshService.onTerminalOutput(sessionId, (data) => {
      const bytes = new Uint8Array(data)
      terminal?.write(bytes)
    })

    unlistenStatus = await sshService.onTerminalStatus(sessionId, (status) => {
      if (status === 'disconnected') {
        sessionStore.updatePaneStatus(props.pane.id, 'disconnected')
        isDisconnected.value = true
        terminal?.writeln('\r\n\x1b[33m● Connection closed.\x1b[0m')
        if (settingsStore.autoReconnect && reconnectAttempts.value < MAX_RECONNECT_RETRIES) {
          scheduleReconnect()
        }
      }
    })

    // Track selection for context menu
    terminal.onSelectionChange(() => {
      const sel = terminal?.getSelection() ?? ''
      hasSelection.value = sel.length > 0
      if (settingsStore.copyOnSelect && sel) {
        navigator.clipboard.writeText(sel).catch(() => {})
      }
    })

    // Right-click → show context menu
    terminalRef.value?.addEventListener('contextmenu', handleContextMenu)

    terminal.onData((data) => {
      if (!props.pane.sessionId) return
      writeToSession(data)
    })

    terminal.onResize(({ cols, rows }) => {
      if (!props.pane.sessionId) return
      sshService.sshResize(props.pane.sessionId, cols, rows)
    })

    if (fitAddon) {
      const dims = fitAddon.proposeDimensions()
      if (dims) {
        await sshService.sshResize(sessionId, dims.cols, dims.rows)
      }
    }

    terminal.focus()
  } catch (err) {
    sessionStore.updatePaneStatus(props.pane.id, 'error')
    terminal.writeln(`\r\n\x1b[31m✗ Connection failed: ${err}\x1b[0m`)
  }
}

function scheduleReconnect() {
  if (reconnectTimer) clearTimeout(reconnectTimer)
  const attempt = reconnectAttempts.value + 1
  terminal?.writeln(`\x1b[36m● Auto-reconnecting in ${RECONNECT_DELAY_MS / 1000}s (attempt ${attempt}/${MAX_RECONNECT_RETRIES})...\x1b[0m`)
  reconnectTimer = setTimeout(() => reconnect(), RECONNECT_DELAY_MS)
}

async function reconnect() {
  if (!terminal) return
  reconnecting.value = true
  reconnectAttempts.value++

  unlistenOutput?.()
  unlistenStatus?.()
  unlistenOutput = null
  unlistenStatus = null

  terminal.writeln('\x1b[36m● Reconnecting...\x1b[0m')
  sessionStore.updatePaneStatus(props.pane.id, 'connecting')

  try {
    const sessionId = await sshService.sshConnect(props.pane.connectionId)
    sessionStore.updatePaneSessionId(props.pane.id, sessionId)
    sessionStore.updatePaneStatus(props.pane.id, 'connected')
    isDisconnected.value = false
    reconnecting.value = false
    reconnectAttempts.value = 0
    terminal.writeln('\x1b[32m● Reconnected successfully.\x1b[0m')

    unlistenOutput = await sshService.onTerminalOutput(sessionId, (data) => {
      const bytes = new Uint8Array(data)
      terminal?.write(bytes)
    })

    unlistenStatus = await sshService.onTerminalStatus(sessionId, (status) => {
      if (status === 'disconnected') {
        sessionStore.updatePaneStatus(props.pane.id, 'disconnected')
        isDisconnected.value = true
        terminal?.writeln('\r\n\x1b[33m● Connection closed.\x1b[0m')
        if (settingsStore.autoReconnect && reconnectAttempts.value < MAX_RECONNECT_RETRIES) {
          scheduleReconnect()
        }
      }
    })

    if (fitAddon) {
      const dims = fitAddon.proposeDimensions()
      if (dims) {
        await sshService.sshResize(sessionId, dims.cols, dims.rows)
      }
    }
  } catch (err) {
    reconnecting.value = false
    terminal.writeln(`\x1b[31m✗ Reconnect failed: ${err}\x1b[0m`)
    if (settingsStore.autoReconnect && reconnectAttempts.value < MAX_RECONNECT_RETRIES) {
      scheduleReconnect()
    } else {
      sessionStore.updatePaneStatus(props.pane.id, 'disconnected')
    }
  }
}

function handleContextMenu(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  emit('focus')
  ctxMenu.value = { show: true, x: e.clientX, y: e.clientY }
}

function handleFocus() {
  emit('focus')
  terminal?.focus()
}

// ============ Search ============

function toggleSearch() {
  showSearch.value = !showSearch.value
  if (showSearch.value) {
    setTimeout(() => searchInputRef.value?.focus(), 50)
  } else {
    searchAddon?.clearDecorations()
    terminal?.focus()
  }
}

function onSearchInput() {
  if (searchText.value) {
    searchAddon?.findNext(searchText.value)
  } else {
    searchAddon?.clearDecorations()
  }
}

function searchNext() {
  if (searchText.value) searchAddon?.findNext(searchText.value)
}

function searchPrev() {
  if (searchText.value) searchAddon?.findPrevious(searchText.value)
}

function onSearchKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    toggleSearch()
  } else if (e.key === 'Enter') {
    if (e.shiftKey) searchPrev()
    else searchNext()
  }
}

// ============ Keyboard shortcuts ============

function handleKeydown(e: KeyboardEvent) {
  if (!props.isActive) return
  const mod = e.metaKey || e.ctrlKey
  if (mod && e.key === 'f') {
    e.preventDefault()
    toggleSearch()
  }
}

// ============ Theme reactivity ============

watch(() => settingsStore.terminalTheme, () => {
  if (terminal) {
    terminal.options.theme = currentTheme.value.theme
  }
})

watch(() => settingsStore.fontSize, (size) => {
  if (terminal) {
    terminal.options.fontSize = size
    fitAddon?.fit()
  }
})

onMounted(() => {
  initTerminal()
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (reconnectTimer) clearTimeout(reconnectTimer)
  if (terminalRef.value) {
    terminalRef.value.removeEventListener('contextmenu', handleContextMenu)
  }
  unlistenOutput?.()
  unlistenStatus?.()
  resizeObserver?.disconnect()

  if (props.pane.sessionId) {
    sshService.sshDisconnect(props.pane.sessionId)
  }
  terminal?.dispose()
})
</script>

<template>
  <div
    class="terminal-pane"
    :class="{ active: isActive }"
    @click="handleFocus"
  >
    <!-- Search bar -->
    <div v-if="showSearch" class="search-bar">
      <input
        ref="searchInputRef"
        v-model="searchText"
        type="text"
        placeholder="Search..."
        class="search-input"
        @input="onSearchInput"
        @keydown="onSearchKeydown"
      />
      <button class="search-btn" title="Previous (Shift+Enter)" @click="searchPrev">▲</button>
      <button class="search-btn" title="Next (Enter)" @click="searchNext">▼</button>
      <button class="search-btn" title="Close (Esc)" @click="toggleSearch">✕</button>
    </div>

    <div ref="terminalRef" class="terminal-wrapper" />

    <!-- Reconnect overlay -->
    <div v-if="isDisconnected && !reconnecting" class="reconnect-overlay">
      <span class="reconnect-label">Disconnected</span>
      <button class="reconnect-btn" @click="reconnect">Reconnect</button>
    </div>

    <!-- Terminal context menu -->
    <ContextMenu
      v-if="ctxMenu.show"
      :items="termCtxItems"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      @select="handleCtxAction"
      @close="ctxMenu.show = false"
    />
  </div>
</template>

<style scoped>
.terminal-pane {
  position: relative;
  width: 100%;
  height: 100%;
  background: var(--terminal-bg);
  display: flex;
  flex-direction: column;
}

.terminal-pane.active {
  outline: 1px solid var(--accent);
  outline-offset: -1px;
}

.terminal-pane:not(.active) {
  opacity: 0.92;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.search-bar .search-input {
  flex: 1;
  padding: 4px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.search-bar .search-input:focus {
  border-color: var(--border-focus);
}

.search-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px 8px;
  font-size: 11px;
  border-radius: var(--radius-sm);
  min-height: 24px;
  display: flex;
  align-items: center;
}

.search-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.terminal-wrapper {
  flex: 1;
  min-height: 0;
  padding: 4px;
}

.terminal-wrapper :deep(.xterm) {
  height: 100%;
}

.terminal-wrapper :deep(.xterm-viewport) {
  overflow-y: auto !important;
}

.reconnect-overlay {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 10;
}

.reconnect-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.reconnect-btn {
  padding: 4px 12px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
}

.reconnect-btn:hover {
  background: var(--accent-hover);
}
</style>
