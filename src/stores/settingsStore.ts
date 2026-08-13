import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import i18n from '@/i18n'

export const useSettingsStore = defineStore('settings', () => {
  const terminalTheme = ref('catppuccin-mocha')
  const fontSize = ref(14)
  const fontFamily = ref("'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace")
  const scrollback = ref(10000)
  const cursorBlink = ref(true)
  const cursorStyle = ref<'block' | 'underline' | 'bar'>('block')
  const copyOnSelect = ref(true)
  const sessionLogging = ref(false)
  const autoReconnect = ref(true)
  const autoLockTimeout = ref(300)
  const clipboardTimeout = ref(30)
  const language = ref<'zh' | 'en'>('zh')
  const appTheme = ref<'dark' | 'light' | 'system'>('dark')
  const onboardingCompleted = ref(false)

  let initialized = false

  async function persist(key: string, value: string) {
    try {
      await invoke('update_settings', { key, value })
    } catch {
      // DB write failure is non-critical
    }
  }

  function setTheme(name: string) {
    terminalTheme.value = name
    persist('terminalTheme', name)
  }

  function setFontSize(size: number) {
    fontSize.value = Math.max(8, Math.min(24, size))
    persist('fontSize', String(fontSize.value))
  }

  function setLanguage(lang: 'zh' | 'en') {
    language.value = lang
    i18n.global.locale.value = lang
    persist('language', lang)
  }

  function setAppTheme(theme: 'dark' | 'light' | 'system') {
    appTheme.value = theme
    persist('appTheme', theme)
    applyTheme(theme)
  }

  function applyTheme(theme: 'dark' | 'light' | 'system') {
    let resolved: 'dark' | 'light' = 'dark'
    if (theme === 'system') {
      resolved = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    } else {
      resolved = theme
    }
    document.documentElement.setAttribute('data-theme', resolved)
  }

  function setOnboardingCompleted(value: boolean) {
    onboardingCompleted.value = value
    persist('onboarding_completed', String(value))
  }

  async function loadFromDB() {
    if (initialized) return
    initialized = true
    try {
      const settings: Record<string, string> = await invoke('get_settings')
      if (settings.terminalTheme) terminalTheme.value = settings.terminalTheme
      if (settings.fontSize) fontSize.value = Number(settings.fontSize) || 14
      if (settings.fontFamily) fontFamily.value = settings.fontFamily
      if (settings.scrollback) scrollback.value = Number(settings.scrollback) || 10000
      if (settings.cursorBlink !== undefined) cursorBlink.value = settings.cursorBlink === 'true'
      if (settings.cursorStyle) cursorStyle.value = settings.cursorStyle as 'block' | 'underline' | 'bar'
      if (settings.copyOnSelect !== undefined) copyOnSelect.value = settings.copyOnSelect !== 'false'
      if (settings.sessionLogging !== undefined) sessionLogging.value = settings.sessionLogging === 'true'
      if (settings.autoReconnect !== undefined) autoReconnect.value = settings.autoReconnect !== 'false'
      if (settings.autoLockTimeout) autoLockTimeout.value = Number(settings.autoLockTimeout) || 300
      if (settings.clipboardTimeout !== undefined) clipboardTimeout.value = Number(settings.clipboardTimeout) ?? 30
      if (settings.language) language.value = settings.language as 'zh' | 'en'
      if (settings.appTheme) appTheme.value = settings.appTheme as 'dark' | 'light' | 'system'
      if (settings.onboarding_completed) onboardingCompleted.value = settings.onboarding_completed === 'true'

      i18n.global.locale.value = language.value
      applyTheme(appTheme.value)
    } catch {
      // Use defaults on error
      applyTheme(appTheme.value)
    }
  }

  function setFontFamily(value: string) {
    fontFamily.value = value
    persist('fontFamily', value)
  }

  function setScrollback(value: number) {
    scrollback.value = Math.max(100, value)
    persist('scrollback', String(scrollback.value))
  }

  function setCursorBlink(value: boolean) {
    cursorBlink.value = value
    persist('cursorBlink', String(value))
  }

  function setCursorStyle(value: 'block' | 'underline' | 'bar') {
    cursorStyle.value = value
    persist('cursorStyle', value)
  }

  function setCopyOnSelect(value: boolean) {
    copyOnSelect.value = value
    persist('copyOnSelect', String(value))
  }

  function setSessionLogging(value: boolean) {
    sessionLogging.value = value
    persist('sessionLogging', String(value))
  }

  function setAutoReconnect(value: boolean) {
    autoReconnect.value = value
    persist('autoReconnect', String(value))
  }

  function setAutoLockTimeout(secs: number) {
    autoLockTimeout.value = Math.max(0, secs)
    persist('autoLockTimeout', String(autoLockTimeout.value))
  }

  function setClipboardTimeout(secs: number) {
    clipboardTimeout.value = Math.max(0, secs)
    persist('clipboardTimeout', String(clipboardTimeout.value))
  }

  // Listen for system theme changes
  if (typeof window !== 'undefined') {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (appTheme.value === 'system') {
        applyTheme('system')
      }
    })
  }

  return {
    terminalTheme,
    fontSize,
    fontFamily,
    scrollback,
    cursorBlink,
    cursorStyle,
    copyOnSelect,
    sessionLogging,
    autoReconnect,
    autoLockTimeout,
    clipboardTimeout,
    language,
    appTheme,
    onboardingCompleted,
    setTheme,
    setFontSize,
    setFontFamily,
    setScrollback,
    setCursorBlink,
    setCursorStyle,
    setCopyOnSelect,
    setSessionLogging,
    setAutoReconnect,
    setAutoLockTimeout,
    setClipboardTimeout,
    setLanguage,
    setAppTheme,
    setOnboardingCompleted,
    applyTheme,
    loadFromDB,
  }
})
