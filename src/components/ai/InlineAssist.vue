<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'

import type { ChatMessage, AiConfig } from '@/services/ai'
import * as aiService from '@/services/ai'

const emit = defineEmits<{
  close: []
  insert: [cmd: string]
}>()

const query = ref('')
const response = ref('')
const isLoading = ref(false)
const error = ref('')
const inputRef = ref<HTMLInputElement>()

let aiConfig: AiConfig | null = null
let unlistenReqId: (() => void) | null = null
let unlistenChunk: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

onMounted(async () => {
  try {
    aiConfig = await aiService.aiGetConfig()
  } catch {
    error.value = 'Failed to load AI config'
  }
  inputRef.value?.focus()
})

onBeforeUnmount(cleanup)

function cleanup() {
  unlistenReqId?.()
  unlistenChunk?.()
  unlistenComplete?.()
  unlistenReqId = null
  unlistenChunk = null
  unlistenComplete = null
}

function copyCode() {
  const text = response.value.replace(/^```\w*\n?/, '').replace(/\n?```$/, '').trim()
  navigator.clipboard.writeText(text)
}

async function submit() {
  const text = query.value.trim()
  if (!text || isLoading.value) return

  if (!aiConfig || (!aiConfig.api_key && aiConfig.provider !== 'ollama')) {
    error.value = 'AI not configured. Open Settings → AI Assistant.'
    return
  }

  error.value = ''
  response.value = ''
  isLoading.value = true

  cleanup()

  const messages: ChatMessage[] = [
    {
      role: 'user',
      content: `Give me a concise, copy-pasteable command for: ${text}\nRespond with ONLY the command, no explanation.`,
    },
  ]

  unlistenReqId = await aiService.onAiRequestId(async (requestId) => {
    unlistenChunk = await aiService.onAiChunk(requestId, (chunk) => {
      response.value += chunk
    })
    unlistenComplete = await aiService.onAiComplete(requestId, (full) => {
      isLoading.value = false
      if (full.startsWith('__ERROR__:')) {
        error.value = full.slice(10)
        response.value = ''
      }
      cleanup()
    })
  })

  try {
    await aiService.aiChat(messages, aiConfig)
  } catch (err) {
    isLoading.value = false
    error.value = String(err)
    cleanup()
  }
}

function insertCommand() {
  const cmd = response.value.replace(/^```\w*\n?/, '').replace(/\n?```$/, '').trim()
  if (cmd) emit('insert', cmd)
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  } else if (e.key === 'Enter') {
    e.preventDefault()
    submit()
  }
}
</script>

<template>
  <div class="inline-assist-overlay" @click.self="emit('close')">
    <div class="inline-assist">
      <div class="ia-input-row">
        <svg class="ia-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          ref="inputRef"
          v-model="query"
          type="text"
          class="ia-input"
          placeholder="Quick command... (e.g. 'find files larger than 1GB')"
          @keydown="onKeydown"
        />
        <button class="ia-send" :disabled="!query.trim() || isLoading" @click="submit">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6" />
          </svg>
        </button>
      </div>

      <div v-if="isLoading" class="ia-loading">
        <span class="dot" /><span class="dot" /><span class="dot" />
      </div>

      <div v-if="error" class="ia-error">{{ error }}</div>

      <div v-if="response && !isLoading" class="ia-response">
        <pre class="ia-code">{{ response.replace(/^```\w*\n?/, '').replace(/\n?```$/, '').trim() }}</pre>
        <div class="ia-response-actions">
          <button class="ia-btn" @click="insertCommand">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
            Insert
          </button>
          <button class="ia-btn" @click="copyCode()">
            Copy
          </button>
        </div>
      </div>

      <div class="ia-hint">
        <kbd>Enter</kbd> to ask &middot; <kbd>Esc</kbd> to close
      </div>
    </div>
  </div>
</template>

<style scoped>
.inline-assist-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 20vh;
  background: rgba(0, 0, 0, 0.3);
}

.inline-assist {
  width: 480px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.ia-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
}

.ia-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.ia-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
}

.ia-input::placeholder {
  color: var(--text-muted);
}

.ia-send {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  padding: 6px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  min-height: 28px;
}

.ia-send:hover:not(:disabled) {
  background: var(--bg-hover);
}

.ia-send:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.ia-loading {
  display: flex;
  gap: 4px;
  align-items: center;
  justify-content: center;
  padding: 12px;
}

.ia-loading .dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--text-muted);
  animation: bounce 1.4s infinite ease-in-out both;
}

.ia-loading .dot:nth-child(1) { animation-delay: -0.32s; }
.ia-loading .dot:nth-child(2) { animation-delay: -0.16s; }

@keyframes bounce {
  0%, 80%, 100% { transform: scale(0); }
  40% { transform: scale(1); }
}

.ia-error {
  padding: 8px 14px;
  font-size: 12px;
  color: var(--error, #ef4444);
}

.ia-response {
  padding: 10px 14px;
}

.ia-code {
  margin: 0;
  padding: 8px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-family: var(--font-mono, monospace);
  color: var(--text-primary);
  overflow-x: auto;
  white-space: pre-wrap;
}

.ia-response-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
  justify-content: flex-end;
}

.ia-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
}

.ia-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.ia-hint {
  padding: 6px 14px;
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.ia-hint kbd {
  padding: 1px 4px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  font-size: 10px;
  font-family: inherit;
}
</style>
