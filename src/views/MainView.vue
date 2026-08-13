<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'

import type { ConnectionInfo } from '@/types'
import Sidebar from '@/components/layout/Sidebar.vue'
import TabBar from '@/components/layout/TabBar.vue'
import StatusBar from '@/components/layout/StatusBar.vue'
import SplitContainer from '@/components/terminal/SplitContainer.vue'
import WelcomeView from '@/components/layout/WelcomeView.vue'
import CommandPalette from '@/components/terminal/CommandPalette.vue'
import LockScreen from '@/components/security/LockScreen.vue'
import SftpView from '@/views/SftpView.vue'
import SettingsView from '@/views/SettingsView.vue'
import KeyManagerView from '@/views/KeyManagerView.vue'
import AiChatView from '@/views/AiChatView.vue'
import InlineAssist from '@/components/ai/InlineAssist.vue'
import ErrorBoundary from '@/components/common/ErrorBoundary.vue'
import OnboardingWizard from '@/components/common/OnboardingWizard.vue'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import * as sftpService from '@/services/sftp'
import * as securityService from '@/services/security'
import * as sshService from '@/services/ssh'

const sessionStore = useSessionStore()
const settingsStore = useSettingsStore()
const sidebarWidth = ref(260)
const isResizing = ref(false)
const showPalette = ref(false)
const isLocked = ref(true)

// Platform detection
const isMacOS = ref(false)

// SFTP state
const sftpMode = ref(false)
const sftpSessionId = ref<string | null>(null)
const sftpConnectionName = ref('')
const sftpConnectionId = ref('')
const sftpLoading = ref(false)

// Settings state
const settingsMode = ref(false)

// Key Manager state
const keyManagerMode = ref(false)

// AI state
const showAiPanel = ref(false)
const showInlineAssist = ref(false)
const aiPanelWidth = ref(380)

let autoLockTimer: ReturnType<typeof setInterval> | null = null

const showOnboarding = computed(() => !settingsStore.onboardingCompleted && !isLocked.value)

function onUnlocked() {
  isLocked.value = false
  startAutoLockTimer()
}

function startAutoLockTimer() {
  stopAutoLockTimer()
  autoLockTimer = setInterval(async () => {
    const timeoutSecs = settingsStore.autoLockTimeout
    if (timeoutSecs <= 0) return
    try {
      const locked = await securityService.checkAutoLock(timeoutSecs)
      if (locked) {
        isLocked.value = true
        stopAutoLockTimer()
      }
    } catch { /* ignore */ }
  }, 30_000)
}

function stopAutoLockTimer() {
  if (autoLockTimer) {
    clearInterval(autoLockTimer)
    autoLockTimer = null
  }
}

function trackActivity() {
  if (!isLocked.value) {
    securityService.touchActivity().catch(() => {})
  }
}

function onResizeStart(e: MouseEvent) {
  e.preventDefault()
  isResizing.value = true
  const startX = e.clientX
  const startWidth = sidebarWidth.value

  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.body.style.webkitUserSelect = 'none'

  function onMouseMove(ev: MouseEvent) {
    ev.preventDefault()
    const delta = ev.clientX - startX
    const newWidth = startWidth + delta
    sidebarWidth.value = Math.max(200, Math.min(400, newWidth))
  }

  function onMouseUp() {
    isResizing.value = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    document.body.style.webkitUserSelect = ''
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

async function openSftp(conn: ConnectionInfo) {
  sftpLoading.value = true
  try {
    const sid = await sftpService.sftpOpen(conn.id)
    sftpSessionId.value = sid
    sftpConnectionName.value = conn.name
    sftpConnectionId.value = conn.id
    sftpMode.value = true
  } catch (err) {
    console.error('SFTP open failed:', err)
  } finally {
    sftpLoading.value = false
  }
}

async function closeSftp() {
  if (sftpSessionId.value) {
    await sftpService.sftpClose(sftpSessionId.value)
  }
  sftpMode.value = false
  sftpSessionId.value = null
}

function handleGlobalKeydown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey

  if (mod && e.shiftKey && (e.key === 'p' || e.key === 'P')) {
    e.preventDefault()
    showPalette.value = !showPalette.value
    return
  }

  if (mod && e.shiftKey && (e.key === 'h' || e.key === 'H')) {
    e.preventDefault()
    sessionStore.splitPane('horizontal')
    return
  }

  if (mod && e.shiftKey && (e.key === 'v' || e.key === 'V') && e.altKey) {
    e.preventDefault()
    sessionStore.splitPane('vertical')
    return
  }

  if (mod && e.shiftKey && (e.key === 'l' || e.key === 'L')) {
    e.preventDefault()
    handleLockApp()
    return
  }

  if (mod && e.key === ',') {
    e.preventDefault()
    settingsMode.value = !settingsMode.value
    return
  }

  if (mod && e.shiftKey && (e.key === 'k' || e.key === 'K')) {
    e.preventDefault()
    keyManagerMode.value = !keyManagerMode.value
    return
  }

  if (mod && e.shiftKey && (e.key === 'a' || e.key === 'A')) {
    e.preventDefault()
    showAiPanel.value = !showAiPanel.value
    return
  }

  if (mod && (e.key === 'i' || e.key === 'I') && !e.shiftKey) {
    e.preventDefault()
    showInlineAssist.value = !showInlineAssist.value
    return
  }

  if (e.key === 'Escape' && !showPalette.value) {
    const active = document.activeElement
    const isInputFocused = active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement
    if (!isInputFocused) {
      if (keyManagerMode.value) {
        keyManagerMode.value = false
        return
      }
      if (settingsMode.value) {
        settingsMode.value = false
        return
      }
      if (sftpMode.value) {
        closeSftp()
        return
      }
    }
  }
}

function handleAiRunCommand(cmd: string) {
  const activeTab = sessionStore.activeTab
  if (!activeTab) return
  const pane = activeTab.panes?.[0]
  if (pane?.sessionId) {
    const encoder = new TextEncoder()
    const bytes = Array.from(encoder.encode(cmd + '\n'))
    sshService.sshWrite(pane.sessionId, bytes)
  }
}

function handleInlineInsert(cmd: string) {
  handleAiRunCommand(cmd)
  showInlineAssist.value = false
}

async function handleLockApp() {
  await securityService.lockApp()
  isLocked.value = true
  stopAutoLockTimer()
}

onMounted(async () => {
  // Detect macOS platform
  const ua = navigator.userAgent.toLowerCase()
  isMacOS.value = ua.includes('macintosh') || ua.includes('mac os')

  window.addEventListener('keydown', handleGlobalKeydown)
  window.addEventListener('mousemove', trackActivity, { passive: true })
  window.addEventListener('keypress', trackActivity, { passive: true })

  try {
    const locked = await securityService.checkLocked()
    isLocked.value = locked
    if (!locked) startAutoLockTimer()
  } catch {
    isLocked.value = true
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
  window.removeEventListener('mousemove', trackActivity)
  window.removeEventListener('keypress', trackActivity)
  stopAutoLockTimer()
})
</script>

<template>
  <div class="main-layout" :class="{ resizing: isResizing, 'is-macos': isMacOS }">
    <aside class="sidebar" :style="{ width: sidebarWidth + 'px' }">
      <div v-if="isMacOS" class="macos-titlebar" data-tauri-drag-region />
      <Sidebar @open-sftp="openSftp" @open-settings="settingsMode = true" @open-keys="keyManagerMode = true" @open-ai="showAiPanel = !showAiPanel" />
    </aside>

    <div class="resize-handle" @mousedown="onResizeStart" />

    <div v-if="isResizing" class="resize-overlay" />

    <main class="workspace">
      <TabBar :is-macos="isMacOS" @open-palette="showPalette = true" />

      <div v-if="sftpLoading" class="sftp-loading">
        <span class="loading-spinner" />
        Connecting SFTP...
      </div>

      <div class="workspace-content">
        <ErrorBoundary>
          <Transition name="view-fade" mode="out-in">
            <!-- Key Manager mode -->
            <KeyManagerView v-if="keyManagerMode" key="keys" @close="keyManagerMode = false" />

            <!-- Settings mode -->
            <SettingsView
              v-else-if="settingsMode"
              key="settings"
              @close="settingsMode = false"
              @locked="isLocked = true; settingsMode = false; stopAutoLockTimer()"
            />

            <!-- SFTP mode -->
            <div v-else-if="sftpMode && sftpSessionId" key="sftp" class="sftp-wrapper">
              <div class="sftp-back-bar">
                <button class="back-btn" @click="closeSftp">
                  ← Back to Terminal
                </button>
              </div>
              <SftpView
                :sftp-session-id="sftpSessionId"
                :connection-name="sftpConnectionName"
                :connection-id="sftpConnectionId"
              />
            </div>

            <!-- Terminal mode -->
            <div v-else key="terminal" class="terminal-wrapper">
              <SplitContainer
                v-if="sessionStore.activeTab"
                :key="sessionStore.activeTab.id"
                :tab="sessionStore.activeTab"
              />
              <WelcomeView v-else />
            </div>
          </Transition>
        </ErrorBoundary>
      </div>
      <StatusBar />
    </main>

    <aside v-if="showAiPanel" class="ai-panel" :style="{ width: aiPanelWidth + 'px' }">
      <AiChatView
        @close="showAiPanel = false"
        @run-command="handleAiRunCommand"
      />
    </aside>

    <CommandPalette v-if="showPalette" @close="showPalette = false" />

    <InlineAssist
      v-if="showInlineAssist"
      @close="showInlineAssist = false"
      @insert="handleInlineInsert"
    />

    <LockScreen v-if="isLocked" @unlocked="onUnlocked" />

    <OnboardingWizard v-if="showOnboarding" @complete="settingsStore.setOnboardingCompleted(true)" />
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  width: 100%;
  height: 100vh;
  overflow: hidden;
}

.main-layout.resizing {
  cursor: col-resize;
  user-select: none;
}

.main-layout.is-macos .sidebar {
  padding-top: 0;
}

.macos-titlebar {
  height: 28px;
  flex-shrink: 0;
  -webkit-app-region: drag;
}

.sidebar {
  flex-shrink: 0;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.resize-handle {
  width: 6px;
  cursor: col-resize;
  background: transparent;
  flex-shrink: 0;
  transition: background var(--transition-fast);
  position: relative;
  z-index: 10;
}

.resize-handle::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: -3px;
  right: -3px;
}

.resize-handle:hover,
.resizing .resize-handle {
  background: var(--accent);
}

.resize-overlay {
  position: fixed;
  inset: 0;
  z-index: 9;
  cursor: col-resize;
}

.workspace {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  height: 100%;
  position: relative;
}

.workspace-content {
  flex: 1;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.sftp-wrapper,
.terminal-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.sftp-loading {
  padding: 8px 12px;
  text-align: center;
  color: var(--accent);
  font-size: 13px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.sftp-back-bar {
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.back-btn {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  font-size: 12px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast);
}

.back-btn:hover {
  background: var(--bg-hover);
}

.ai-panel {
  flex-shrink: 0;
  height: 100%;
  overflow: hidden;
  border-left: 1px solid var(--border-color);
}

/* View transitions */
.view-fade-enter-active,
.view-fade-leave-active {
  transition: opacity 0.2s ease;
}
.view-fade-enter-from,
.view-fade-leave-to {
  opacity: 0;
}
</style>
