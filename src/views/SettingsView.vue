<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settingsStore'
import { TERMINAL_THEMES } from '@/styles/themes'
import * as securityService from '@/services/security'
import AiSettings from '@/components/ai/AiSettings.vue'
import ShortcutSettings from '@/components/settings/ShortcutSettings.vue'

const emit = defineEmits<{
  close: []
  locked: []
}>()

const { t } = useI18n()
const settingsStore = useSettingsStore()

const activeSection = ref('general')

const sections = [
  { id: 'general', label: 'settings.general', icon: 'M12 15a3 3 0 100-6 3 3 0 000 6zM19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09a1.65 1.65 0 00-1-1.51 1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09a1.65 1.65 0 001.51-1 1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06a1.65 1.65 0 00.33 1.82V9a2 2 0 010 4h-.09' },
  { id: 'terminal', label: 'settings.terminal', icon: 'M4 17l6-6-6-6M12 19h8' },
  { id: 'connection', label: 'settings.connection', icon: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93z' },
  { id: 'shortcuts', label: 'settings.shortcuts', icon: 'M18 3a3 3 0 00-3 3v12a3 3 0 003 3 3 3 0 003-3 3 3 0 00-3-3H6a3 3 0 00-3 3 3 3 0 003 3 3 3 0 003-3V6a3 3 0 00-3-3 3 3 0 00-3 3 3 3 0 003 3h12a3 3 0 003-3 3 3 0 00-3-3z' },
  { id: 'ai', label: 'settings.ai', icon: 'M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1v3h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2v-3h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2z' },
  { id: 'security', label: 'settings.security', icon: 'M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z' },
  { id: 'about', label: 'settings.about', icon: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z' },
]

const autoLockOptions = [
  { value: 0, labelKey: 'settings.disabled' },
  { value: 60, label: '1 minute' },
  { value: 300, label: '5 minutes' },
  { value: 600, label: '10 minutes' },
  { value: 1800, label: '30 minutes' },
  { value: 3600, label: '1 hour' },
]

const clipboardTimeoutOptions = [
  { value: 0, labelKey: 'settings.disabled' },
  { value: 10, label: '10 seconds' },
  { value: 30, label: '30 seconds' },
  { value: 60, label: '1 minute' },
  { value: 120, label: '2 minutes' },
  { value: 300, label: '5 minutes' },
]

async function handleLockNow() {
  await securityService.lockApp()
  emit('locked')
}
</script>

<template>
  <div class="settings-view">
    <div class="settings-sidebar">
      <div class="settings-sidebar-header">
        <button class="btn-close" :title="t('common.close')" @click="emit('close')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M19 12H5M12 19l-7-7 7-7" />
          </svg>
        </button>
        <h2 class="settings-title">{{ t('settings.title') }}</h2>
      </div>
      <nav class="settings-nav">
        <button
          v-for="section in sections"
          :key="section.id"
          class="nav-item"
          :class="{ active: activeSection === section.id }"
          @click="activeSection = section.id"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path :d="section.icon" />
          </svg>
          <span>{{ t(section.label) }}</span>
        </button>
      </nav>
      <div class="settings-nav-footer">
        <span class="version-badge">v0.1.0</span>
      </div>
    </div>

    <div class="settings-content">
      <!-- General Section -->
      <div v-show="activeSection === 'general'" class="section">
        <h3 class="section-title">{{ t('settings.general') }}</h3>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.language') }}</span>
            <span class="label-desc">{{ t('settings.languageDesc') }}</span>
          </div>
          <div class="setting-control">
            <select
              :value="settingsStore.language"
              class="select-control"
              @change="settingsStore.setLanguage(($event.target as HTMLSelectElement).value as 'zh' | 'en')"
            >
              <option value="zh">中文</option>
              <option value="en">English</option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.appTheme') }}</span>
            <span class="label-desc">{{ t('settings.appThemeDesc') }}</span>
          </div>
          <div class="setting-control">
            <div class="radio-group">
              <label class="radio-item" :class="{ active: settingsStore.appTheme === 'dark' }">
                <input type="radio" name="appTheme" value="dark" :checked="settingsStore.appTheme === 'dark'" @change="settingsStore.setAppTheme('dark')" />
                <span class="radio-label">{{ t('settings.dark') }}</span>
              </label>
              <label class="radio-item" :class="{ active: settingsStore.appTheme === 'light' }">
                <input type="radio" name="appTheme" value="light" :checked="settingsStore.appTheme === 'light'" @change="settingsStore.setAppTheme('light')" />
                <span class="radio-label">{{ t('settings.light') }}</span>
              </label>
              <label class="radio-item" :class="{ active: settingsStore.appTheme === 'system' }">
                <input type="radio" name="appTheme" value="system" :checked="settingsStore.appTheme === 'system'" @change="settingsStore.setAppTheme('system')" />
                <span class="radio-label">{{ t('settings.system') }}</span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- Terminal Section -->
      <div v-show="activeSection === 'terminal'" class="section">
        <h3 class="section-title">{{ t('settings.terminal') }}</h3>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.theme') }}</span>
            <span class="label-desc">{{ t('settings.themeDesc') }}</span>
          </div>
          <div class="setting-control">
            <select
              :value="settingsStore.terminalTheme"
              class="select-control"
              @change="settingsStore.setTheme(($event.target as HTMLSelectElement).value)"
            >
              <option
                v-for="theme in TERMINAL_THEMES"
                :key="theme.name"
                :value="theme.name"
              >
                {{ theme.label }}
              </option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.fontSize') }}</span>
            <span class="label-desc">{{ t('settings.fontSizeDesc') }}</span>
          </div>
          <div class="setting-control">
            <div class="number-input">
              <button
                class="number-btn"
                :disabled="settingsStore.fontSize <= 8"
                @click="settingsStore.setFontSize(settingsStore.fontSize - 1)"
              >−</button>
              <input
                type="number"
                :value="settingsStore.fontSize"
                min="8"
                max="24"
                class="number-field"
                @change="settingsStore.setFontSize(Number(($event.target as HTMLInputElement).value))"
              />
              <button
                class="number-btn"
                :disabled="settingsStore.fontSize >= 24"
                @click="settingsStore.setFontSize(settingsStore.fontSize + 1)"
              >+</button>
            </div>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.fontFamily') }}</span>
            <span class="label-desc">{{ t('settings.fontFamilyDesc') }}</span>
          </div>
          <div class="setting-control">
            <input
              type="text"
              :value="settingsStore.fontFamily"
              class="text-input"
              @change="settingsStore.setFontFamily(($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.cursorStyle') }}</span>
          </div>
          <div class="setting-control">
            <div class="radio-group">
              <label class="radio-item" :class="{ active: settingsStore.cursorStyle === 'block' }">
                <input type="radio" name="cursorStyle" value="block" :checked="settingsStore.cursorStyle === 'block'" @change="settingsStore.setCursorStyle('block')" />
                <span class="radio-label">Block</span>
              </label>
              <label class="radio-item" :class="{ active: settingsStore.cursorStyle === 'underline' }">
                <input type="radio" name="cursorStyle" value="underline" :checked="settingsStore.cursorStyle === 'underline'" @change="settingsStore.setCursorStyle('underline')" />
                <span class="radio-label">Underline</span>
              </label>
              <label class="radio-item" :class="{ active: settingsStore.cursorStyle === 'bar' }">
                <input type="radio" name="cursorStyle" value="bar" :checked="settingsStore.cursorStyle === 'bar'" @change="settingsStore.setCursorStyle('bar')" />
                <span class="radio-label">Bar</span>
              </label>
            </div>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.cursorBlink') }}</span>
            <span class="label-desc">{{ t('settings.cursorBlinkDesc') }}</span>
          </div>
          <div class="setting-control">
            <button
              class="toggle-switch"
              :class="{ on: settingsStore.cursorBlink }"
              role="switch"
              :aria-checked="settingsStore.cursorBlink"
              @click="settingsStore.setCursorBlink(!settingsStore.cursorBlink)"
            >
              <span class="toggle-thumb" />
            </button>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.scrollback') }}</span>
            <span class="label-desc">{{ t('settings.scrollbackDesc') }}</span>
          </div>
          <div class="setting-control">
            <input
              type="number"
              :value="settingsStore.scrollback"
              min="100"
              step="1000"
              class="number-field wide"
              @change="settingsStore.setScrollback(Number(($event.target as HTMLInputElement).value))"
            />
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.copyOnSelect') }}</span>
            <span class="label-desc">{{ t('settings.copyOnSelectDesc') }}</span>
          </div>
          <div class="setting-control">
            <button
              class="toggle-switch"
              :class="{ on: settingsStore.copyOnSelect }"
              role="switch"
              :aria-checked="settingsStore.copyOnSelect"
              @click="settingsStore.setCopyOnSelect(!settingsStore.copyOnSelect)"
            >
              <span class="toggle-thumb" />
            </button>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.sessionLogging') }}</span>
            <span class="label-desc">{{ t('settings.sessionLoggingDesc') }}</span>
          </div>
          <div class="setting-control">
            <button
              class="toggle-switch"
              :class="{ on: settingsStore.sessionLogging }"
              role="switch"
              :aria-checked="settingsStore.sessionLogging"
              @click="settingsStore.setSessionLogging(!settingsStore.sessionLogging)"
            >
              <span class="toggle-thumb" />
            </button>
          </div>
        </div>
      </div>

      <!-- Connection Section -->
      <div v-show="activeSection === 'connection'" class="section">
        <h3 class="section-title">{{ t('settings.connection') }}</h3>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.autoReconnect') }}</span>
            <span class="label-desc">{{ t('settings.autoReconnectDesc') }}</span>
          </div>
          <div class="setting-control">
            <button
              class="toggle-switch"
              :class="{ on: settingsStore.autoReconnect }"
              role="switch"
              :aria-checked="settingsStore.autoReconnect"
              @click="settingsStore.setAutoReconnect(!settingsStore.autoReconnect)"
            >
              <span class="toggle-thumb" />
            </button>
          </div>
        </div>
      </div>

      <!-- Keyboard Shortcuts Section -->
      <div v-show="activeSection === 'shortcuts'" class="section">
        <h3 class="section-title">{{ t('settings.shortcuts') }}</h3>
        <ShortcutSettings />
      </div>

      <!-- AI Section -->
      <div v-show="activeSection === 'ai'" class="section">
        <AiSettings />
      </div>

      <!-- Security Section -->
      <div v-show="activeSection === 'security'" class="section">
        <h3 class="section-title">{{ t('settings.security') }}</h3>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.autoLock') }}</span>
            <span class="label-desc">{{ t('settings.autoLockDesc') }}</span>
          </div>
          <div class="setting-control">
            <select
              :value="settingsStore.autoLockTimeout"
              class="select-control"
              @change="settingsStore.setAutoLockTimeout(Number(($event.target as HTMLSelectElement).value))"
            >
              <option
                v-for="opt in autoLockOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.labelKey ? t(opt.labelKey) : opt.label }}
              </option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.clipboardClear') }}</span>
            <span class="label-desc">{{ t('settings.clipboardClearDesc') }}</span>
          </div>
          <div class="setting-control">
            <select
              :value="settingsStore.clipboardTimeout"
              class="select-control"
              @change="settingsStore.setClipboardTimeout(Number(($event.target as HTMLSelectElement).value))"
            >
              <option
                v-for="opt in clipboardTimeoutOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.labelKey ? t(opt.labelKey) : opt.label }}
              </option>
            </select>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.changeMasterPassword') }}</span>
            <span class="label-desc">{{ t('settings.changeMasterPasswordDesc') }}</span>
          </div>
          <div class="setting-control">
            <button class="btn-secondary">{{ t('settings.changePassword') }}</button>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-label">
            <span class="label-text">{{ t('settings.lockNow') }}</span>
            <span class="label-desc">{{ t('settings.lockNowDesc') }}</span>
          </div>
          <div class="setting-control">
            <button class="btn-danger" @click="handleLockNow">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                <path d="M7 11V7a5 5 0 0110 0v4" />
              </svg>
              {{ t('settings.lock') }}
            </button>
          </div>
        </div>
      </div>

      <!-- About Section -->
      <div v-show="activeSection === 'about'" class="section">
        <h3 class="section-title">{{ t('settings.about') }}</h3>

        <div class="about-card">
          <div class="about-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 17l6-6-6-6" />
              <path d="M12 19h8" />
            </svg>
          </div>
          <h2 class="about-name">{{ t('settings.aboutName') }}</h2>
          <p class="about-version">{{ t('settings.aboutVersion') }}</p>
          <p class="about-desc">{{ t('settings.aboutDesc') }}</p>
          <div class="about-meta">
            <span>{{ t('settings.builtWith') }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  height: 100%;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.settings-sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.settings-sidebar-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-color);
}

.btn-close {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 6px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  transition: all var(--transition-fast);
}

.btn-close:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.settings-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.settings-nav {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border: none;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: var(--radius-md);
  font-size: 13px;
  transition: all var(--transition-fast);
  text-align: left;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.settings-nav-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.version-badge {
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-surface);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
}

.section {
  max-width: 600px;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 24px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
  border-bottom: 1px solid color-mix(in srgb, var(--border-color) 50%, transparent);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
  flex: 1;
}

.label-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.label-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.setting-control {
  flex-shrink: 0;
  margin-left: 24px;
}

.select-control {
  padding: 6px 28px 6px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236c7086' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  transition: border-color var(--transition-fast);
}

.select-control:hover {
  border-color: var(--text-muted);
}

.select-control:focus {
  border-color: var(--border-focus);
}

.number-input {
  display: flex;
  align-items: center;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.number-btn {
  width: 28px;
  height: 30px;
  background: var(--bg-surface);
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 15px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
}

.number-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.number-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.number-field {
  width: 52px;
  height: 30px;
  text-align: center;
  border: none;
  border-left: 1px solid var(--border-color);
  border-right: 1px solid var(--border-color);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  -moz-appearance: textfield;
}

.number-field::-webkit-inner-spin-button,
.number-field::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.number-field.wide {
  width: 80px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 6px 10px;
  text-align: left;
  background: var(--bg-surface);
}

.text-input {
  width: 220px;
  padding: 6px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 12px;
  font-family: var(--font-mono);
  outline: none;
  transition: border-color var(--transition-fast);
}

.text-input:focus {
  border-color: var(--border-focus);
}

.radio-group {
  display: flex;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.radio-item {
  cursor: pointer;
  padding: 6px 14px;
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--bg-surface);
  border-right: 1px solid var(--border-color);
  transition: all var(--transition-fast);
  user-select: none;
}

.radio-item:last-child {
  border-right: none;
}

.radio-item input {
  display: none;
}

.radio-item:hover {
  background: var(--bg-hover);
}

.radio-item.active {
  background: var(--accent);
  color: #fff;
  font-weight: 500;
}

.toggle-switch {
  position: relative;
  width: 40px;
  height: 22px;
  background: var(--bg-active);
  border: none;
  border-radius: 11px;
  cursor: pointer;
  transition: background var(--transition-normal);
  padding: 0;
}

.toggle-switch.on {
  background: var(--accent);
}

.toggle-thumb {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  transition: transform var(--transition-normal);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.toggle-switch.on .toggle-thumb {
  transform: translateX(18px);
}

.btn-secondary {
  padding: 6px 14px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: var(--bg-hover);
  border-color: var(--text-muted);
}

.btn-danger {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: color-mix(in srgb, var(--error) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--error) 40%, transparent);
  border-radius: var(--radius-md);
  color: var(--error);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-danger:hover {
  background: color-mix(in srgb, var(--error) 25%, transparent);
  border-color: var(--error);
}

.about-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 40px 24px;
  background: var(--bg-surface);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-sm);
}

.about-icon {
  margin-bottom: 16px;
  padding: 16px;
  background: var(--bg-primary);
  border-radius: 16px;
}

.about-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.about-version {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 16px;
}

.about-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  max-width: 360px;
  margin-bottom: 20px;
}

.about-meta {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
