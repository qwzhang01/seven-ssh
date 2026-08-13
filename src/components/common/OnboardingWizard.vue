<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settingsStore'
import AppLogo from '@/components/common/AppLogo.vue'

const emit = defineEmits<{ complete: [] }>()
const { t } = useI18n()
const settingsStore = useSettingsStore()

const currentStep = ref(0)
const selectedTheme = ref<'dark' | 'light'>(settingsStore.appTheme === 'light' ? 'light' : 'dark')

const steps = ['welcome', 'setup', 'import', 'theme', 'done']

function next() {
  if (currentStep.value < steps.length - 1) {
    currentStep.value++
  }
}

function back() {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

function skip() {
  next()
}

function selectTheme(theme: 'dark' | 'light') {
  selectedTheme.value = theme
  settingsStore.setAppTheme(theme)
}

function finish() {
  settingsStore.setOnboardingCompleted(true)
  emit('complete')
}
</script>

<template>
  <div class="onboarding-overlay">
    <div class="onboarding-card">
      <!-- Step indicators -->
      <div class="step-dots">
        <span
          v-for="(_, idx) in steps"
          :key="idx"
          class="dot"
          :class="{ active: idx === currentStep, done: idx < currentStep }"
        />
      </div>

      <!-- Step 1: Welcome -->
      <div v-if="currentStep === 0" class="step-content">
        <AppLogo :size="48" glow class="step-logo" />
        <h2 class="step-title">{{ t('onboarding.welcome') }}</h2>
        <p class="step-desc">{{ t('onboarding.welcomeDesc') }}</p>
        <button class="btn-primary" @click="next">{{ t('onboarding.next') }}</button>
      </div>

      <!-- Step 2: Quick Setup -->
      <div v-else-if="currentStep === 1" class="step-content">
        <h2 class="step-title">{{ t('onboarding.quickSetup') }}</h2>
        <p class="step-desc">{{ t('onboarding.quickSetupDesc') }}</p>
        <div class="setup-hint">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" />
            <path d="M7 11V7a5 5 0 0110 0v4" />
          </svg>
          <span>{{ t('lockScreen.setupSubtitle') }}</span>
        </div>
        <div class="step-actions">
          <button class="btn-secondary" @click="skip">{{ t('onboarding.skip') }}</button>
          <button class="btn-primary" @click="next">{{ t('onboarding.next') }}</button>
        </div>
      </div>

      <!-- Step 3: Import -->
      <div v-else-if="currentStep === 2" class="step-content">
        <h2 class="step-title">{{ t('onboarding.importStep') }}</h2>
        <p class="step-desc">{{ t('onboarding.importStepDesc') }}</p>
        <div class="import-options">
          <button class="option-btn" @click="next">
            <svg width="20" height="20" viewBox="0 0 16 16" fill="none">
              <path d="M8 10V2M8 10l-3-3M8 10l3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M2 12v1a1 1 0 001 1h10a1 1 0 001-1v-1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            {{ t('onboarding.importFromConfig') }}
          </button>
          <button class="option-btn secondary" @click="skip">
            {{ t('onboarding.skipImport') }}
          </button>
        </div>
      </div>

      <!-- Step 4: Theme -->
      <div v-else-if="currentStep === 3" class="step-content">
        <h2 class="step-title">{{ t('onboarding.themeStep') }}</h2>
        <p class="step-desc">{{ t('onboarding.themeStepDesc') }}</p>
        <div class="theme-choices">
          <button
            class="theme-card"
            :class="{ selected: selectedTheme === 'dark' }"
            @click="selectTheme('dark')"
          >
            <div class="theme-preview dark-preview">
              <div class="preview-sidebar" />
              <div class="preview-content">
                <div class="preview-bar" />
                <div class="preview-body" />
              </div>
            </div>
            <span>{{ t('settings.dark') }}</span>
          </button>
          <button
            class="theme-card"
            :class="{ selected: selectedTheme === 'light' }"
            @click="selectTheme('light')"
          >
            <div class="theme-preview light-preview">
              <div class="preview-sidebar" />
              <div class="preview-content">
                <div class="preview-bar" />
                <div class="preview-body" />
              </div>
            </div>
            <span>{{ t('settings.light') }}</span>
          </button>
        </div>
        <button class="btn-primary" @click="next">{{ t('onboarding.next') }}</button>
      </div>

      <!-- Step 5: Done -->
      <div v-else-if="currentStep === 4" class="step-content">
        <div class="step-icon done-icon">
          <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2" stroke-linecap="round">
            <path d="M20 6L9 17l-5-5" />
          </svg>
        </div>
        <h2 class="step-title">{{ t('onboarding.done') }}</h2>
        <p class="step-desc">{{ t('onboarding.doneDesc') }}</p>
        <button class="btn-primary" @click="finish">{{ t('onboarding.getStarted') }}</button>
      </div>

      <!-- Back button (after step 1) -->
      <button v-if="currentStep > 0 && currentStep < 4" class="btn-back" @click="back">
        {{ t('onboarding.back') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.onboarding-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  backdrop-filter: blur(12px);
}

.onboarding-card {
  position: relative;
  width: 440px;
  padding: 40px 36px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  box-shadow: var(--shadow-lg);
}

.step-dots {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 32px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--bg-active);
  transition: all var(--transition-normal);
}

.dot.active {
  background: var(--accent);
  transform: scale(1.3);
}

.dot.done {
  background: var(--success);
}

.step-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.step-icon {
  margin-bottom: 20px;
  padding: 20px;
  background: var(--bg-surface);
  border-radius: 50%;
}

.step-logo {
  margin-bottom: 20px;
}

.step-title {
  font-size: 22px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.step-desc {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 28px;
  max-width: 340px;
}

.btn-primary {
  padding: 10px 24px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.btn-primary:hover {
  background: var(--accent-hover);
}

.btn-secondary {
  padding: 10px 24px;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: 14px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: var(--bg-hover);
}

.step-actions {
  display: flex;
  gap: 12px;
}

.setup-hint {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: var(--bg-surface);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13px;
  margin-bottom: 24px;
}

.import-options {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.option-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 20px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.option-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
}

.option-btn.secondary {
  background: transparent;
  color: var(--text-muted);
  border-color: transparent;
}

.option-btn.secondary:hover {
  color: var(--text-secondary);
}

.theme-choices {
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
}

.theme-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: var(--bg-surface);
  border: 2px solid var(--border-color);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--transition-fast);
  color: var(--text-secondary);
  font-size: 13px;
}

.theme-card:hover {
  border-color: var(--text-muted);
}

.theme-card.selected {
  border-color: var(--accent);
  color: var(--text-primary);
}

.theme-preview {
  width: 120px;
  height: 72px;
  border-radius: var(--radius-sm);
  display: flex;
  overflow: hidden;
}

.dark-preview {
  background: #1e1e2e;
}

.dark-preview .preview-sidebar {
  width: 30%;
  background: #181825;
  border-right: 1px solid #313244;
}

.dark-preview .preview-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.dark-preview .preview-bar {
  height: 12px;
  background: #11111b;
  border-bottom: 1px solid #313244;
}

.dark-preview .preview-body {
  flex: 1;
  background: #1e1e2e;
}

.light-preview {
  background: #ffffff;
}

.light-preview .preview-sidebar {
  width: 30%;
  background: #f5f5f5;
  border-right: 1px solid #d4d4d4;
}

.light-preview .preview-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.light-preview .preview-bar {
  height: 12px;
  background: #ebebeb;
  border-bottom: 1px solid #d4d4d4;
}

.light-preview .preview-body {
  flex: 1;
  background: #ffffff;
}

.done-icon {
  background: color-mix(in srgb, var(--success) 15%, transparent);
}

.btn-back {
  position: absolute;
  bottom: 16px;
  left: 36px;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: color var(--transition-fast);
}

.btn-back:hover {
  color: var(--text-secondary);
}
</style>
