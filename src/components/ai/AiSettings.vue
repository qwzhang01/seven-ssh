<script setup lang="ts">
import { ref, onMounted } from 'vue'

import type { AiConfig } from '@/services/ai'
import * as aiService from '@/services/ai'

const provider = ref('openai')
const apiKey = ref('')
const baseUrl = ref('')
const model = ref('gpt-4o-mini')
const temperature = ref(0.7)
const maxTokens = ref(2048)
const privacyRedaction = ref(true)
const testStatus = ref<'idle' | 'testing' | 'success' | 'error'>('idle')
const testMessage = ref('')
const saving = ref(false)

onMounted(async () => {
  try {
    const config = await aiService.aiGetConfig()
    provider.value = config.provider || 'openai'
    apiKey.value = config.api_key || ''
    baseUrl.value = config.base_url || ''
    model.value = config.model || 'gpt-4o-mini'
    temperature.value = config.temperature ?? 0.7
    maxTokens.value = config.max_tokens ?? 2048
  } catch { /* use defaults */ }
})

async function save() {
  saving.value = true
  try {
    const config: AiConfig = {
      provider: provider.value,
      api_key: apiKey.value || null,
      base_url: baseUrl.value || null,
      model: model.value,
      temperature: temperature.value,
      max_tokens: maxTokens.value,
    }
    await aiService.aiSaveConfig(config)
    saving.value = false
  } catch {
    saving.value = false
  }
}

async function testConnection() {
  testStatus.value = 'testing'
  testMessage.value = ''

  const config: AiConfig = {
    provider: provider.value,
    api_key: apiKey.value || null,
    base_url: baseUrl.value || null,
    model: model.value,
    temperature: temperature.value,
    max_tokens: maxTokens.value,
  }

  try {
    await aiService.aiChat(
      [{ role: 'user', content: 'Say "Connection successful!" in exactly those words.' }],
      config,
    )
    testStatus.value = 'success'
    testMessage.value = 'Connection successful!'
  } catch (err) {
    testStatus.value = 'error'
    testMessage.value = String(err)
  }
}

const providerOptions = [
  { value: 'openai', label: 'OpenAI' },
  { value: 'ollama', label: 'Ollama (Local)' },
  { value: 'custom', label: 'Custom (OpenAI-compatible)' },
]
</script>

<template>
  <div class="ai-settings-section">
    <h3 class="section-title">AI Assistant</h3>

    <div class="setting-row">
      <div class="setting-label">
        <span class="label-text">Provider</span>
        <span class="label-desc">LLM service to use</span>
      </div>
      <div class="setting-control">
        <select v-model="provider" class="select-control" @change="save">
          <option v-for="opt in providerOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </div>
    </div>

    <div v-if="provider !== 'ollama'" class="setting-row">
      <div class="setting-label">
        <span class="label-text">API Key</span>
        <span class="label-desc">Your provider API key</span>
      </div>
      <div class="setting-control">
        <input
          v-model="apiKey"
          type="password"
          class="text-input"
          placeholder="sk-..."
          @change="save"
        />
      </div>
    </div>

    <div v-if="provider === 'ollama' || provider === 'custom'" class="setting-row">
      <div class="setting-label">
        <span class="label-text">Base URL</span>
        <span class="label-desc">{{ provider === 'ollama' ? 'Default: http://localhost:11434' : 'OpenAI-compatible API endpoint' }}</span>
      </div>
      <div class="setting-control">
        <input
          v-model="baseUrl"
          type="text"
          class="text-input"
          :placeholder="provider === 'ollama' ? 'http://localhost:11434' : 'https://api.example.com'"
          @change="save"
        />
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-label">
        <span class="label-text">Model</span>
        <span class="label-desc">Model name or ID</span>
      </div>
      <div class="setting-control">
        <input
          v-model="model"
          type="text"
          class="text-input"
          :placeholder="provider === 'ollama' ? 'llama3' : 'gpt-4o-mini'"
          @change="save"
        />
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-label">
        <span class="label-text">Temperature</span>
        <span class="label-desc">Creativity (0 = focused, 2 = creative)</span>
      </div>
      <div class="setting-control slider-control">
        <input
          v-model.number="temperature"
          type="range"
          min="0"
          max="2"
          step="0.1"
          class="range-input"
          @change="save"
        />
        <span class="range-value">{{ temperature.toFixed(1) }}</span>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-label">
        <span class="label-text">Max Tokens</span>
        <span class="label-desc">Maximum response length</span>
      </div>
      <div class="setting-control">
        <input
          v-model.number="maxTokens"
          type="number"
          min="256"
          max="16384"
          step="256"
          class="number-field wide"
          @change="save"
        />
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-label">
        <span class="label-text">Privacy Redaction</span>
        <span class="label-desc">Redact IPs, passwords, and keys before sending to AI</span>
      </div>
      <div class="setting-control">
        <button
          class="toggle-switch"
          :class="{ on: privacyRedaction }"
          role="switch"
          :aria-checked="privacyRedaction"
          @click="privacyRedaction = !privacyRedaction"
        >
          <span class="toggle-thumb" />
        </button>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-label">
        <span class="label-text">Test Connection</span>
        <span class="label-desc">Verify your AI provider is reachable</span>
      </div>
      <div class="setting-control">
        <button
          class="btn-test"
          :class="{ testing: testStatus === 'testing' }"
          :disabled="testStatus === 'testing'"
          @click="testConnection"
        >
          {{ testStatus === 'testing' ? 'Testing...' : 'Test' }}
        </button>
      </div>
    </div>

    <div v-if="testMessage" class="test-result" :class="testStatus">
      {{ testMessage }}
    </div>
  </div>
</template>

<style scoped>
.ai-settings-section {
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
}

.text-input:focus {
  border-color: var(--border-focus);
}

.number-field.wide {
  width: 80px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 6px 10px;
  text-align: left;
  background: var(--bg-surface);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}

.slider-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.range-input {
  width: 120px;
  accent-color: var(--accent);
}

.range-value {
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 28px;
  text-align: right;
  font-family: var(--font-mono);
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

.btn-test {
  padding: 6px 14px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
}

.btn-test:hover:not(:disabled) {
  background: var(--bg-hover);
}

.btn-test:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.test-result {
  margin-top: 8px;
  padding: 8px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
}

.test-result.success {
  background: color-mix(in srgb, #22c55e 15%, transparent);
  color: #22c55e;
  border: 1px solid color-mix(in srgb, #22c55e 40%, transparent);
}

.test-result.error {
  background: color-mix(in srgb, var(--error, #ef4444) 15%, transparent);
  color: var(--error, #ef4444);
  border: 1px solid color-mix(in srgb, var(--error, #ef4444) 40%, transparent);
}
</style>
