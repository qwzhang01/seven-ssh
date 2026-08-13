<script setup lang="ts">
import { ref, nextTick, onMounted, onBeforeUnmount, watch } from 'vue'

import type { ChatMessage, AiConfig } from '@/services/ai'
import * as aiService from '@/services/ai'

const emit = defineEmits<{
  close: []
  runCommand: [cmd: string]
}>()

const messages = ref<ChatMessage[]>([])
const inputText = ref('')
const isStreaming = ref(false)
const streamingContent = ref('')
const configLoaded = ref(false)
const configError = ref('')
const messagesContainer = ref<HTMLDivElement>()
const inputRef = ref<HTMLTextAreaElement>()

let aiConfig: AiConfig | null = null
let unlistenReqId: (() => void) | null = null
let unlistenChunk: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

onMounted(async () => {
  try {
    aiConfig = await aiService.aiGetConfig()
    configLoaded.value = true
    if (!aiConfig.api_key && aiConfig.provider !== 'ollama') {
      configError.value = 'API key not configured. Go to Settings → AI Assistant to set up.'
    }
  } catch {
    configLoaded.value = true
    configError.value = 'Failed to load AI config.'
  }
  inputRef.value?.focus()
})

onBeforeUnmount(() => {
  cleanup()
})

function cleanup() {
  unlistenReqId?.()
  unlistenChunk?.()
  unlistenComplete?.()
  unlistenReqId = null
  unlistenChunk = null
  unlistenComplete = null
}

function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

watch(streamingContent, scrollToBottom)

async function sendMessage() {
  const text = inputText.value.trim()
  if (!text || isStreaming.value) return

  if (!aiConfig || (!aiConfig.api_key && aiConfig.provider !== 'ollama')) {
    configError.value = 'Please configure your AI provider in Settings → AI Assistant.'
    return
  }

  configError.value = ''
  const userMsg: ChatMessage = { role: 'user', content: text }
  messages.value.push(userMsg)
  inputText.value = ''
  isStreaming.value = true
  streamingContent.value = ''

  scrollToBottom()

  cleanup()

  unlistenReqId = await aiService.onAiRequestId(async (requestId) => {
    unlistenChunk = await aiService.onAiChunk(requestId, (chunk) => {
      streamingContent.value += chunk
    })
    unlistenComplete = await aiService.onAiComplete(requestId, (full) => {
      isStreaming.value = false
      const isError = full.startsWith('__ERROR__:')
      messages.value.push({
        role: 'assistant',
        content: isError ? `Error: ${full.slice(10)}` : streamingContent.value || full,
      })
      streamingContent.value = ''
      cleanup()
      scrollToBottom()
    })
  })

  const chatHistory = messages.value.filter((m) => m.role !== 'system')

  try {
    await aiService.aiChat(chatHistory, aiConfig)
  } catch (err) {
    isStreaming.value = false
    messages.value.push({
      role: 'assistant',
      content: `Error: ${err}`,
    })
    streamingContent.value = ''
    cleanup()
    scrollToBottom()
  }
}

function clearChat() {
  messages.value = []
  streamingContent.value = ''
  isStreaming.value = false
  cleanup()
}

function onInputKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    sendMessage()
  }
}

function copyCode(code: string) {
  navigator.clipboard.writeText(code).catch(() => {})
}

function runCode(code: string) {
  emit('runCommand', code)
}

function extractCodeBlocks(text: string): { type: 'text' | 'code'; content: string; lang: string }[] {
  const parts: { type: 'text' | 'code'; content: string; lang: string }[] = []
  const regex = /```(\w*)\n([\s\S]*?)```/g
  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push({ type: 'text', content: text.slice(lastIndex, match.index), lang: '' })
    }
    parts.push({ type: 'code', content: match[2], lang: match[1] || 'bash' })
    lastIndex = regex.lastIndex
  }
  if (lastIndex < text.length) {
    parts.push({ type: 'text', content: text.slice(lastIndex), lang: '' })
  }
  return parts
}
</script>

<template>
  <div class="ai-chat">
    <div class="ai-header">
      <div class="ai-title">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2z" />
          <circle cx="8.5" cy="14.5" r="1.5" />
          <circle cx="15.5" cy="14.5" r="1.5" />
        </svg>
        <span>AI Assistant</span>
        <span class="model-badge" v-if="aiConfig">{{ aiConfig.model }}</span>
      </div>
      <div class="ai-actions">
        <button class="icon-btn" title="Clear Chat" @click="clearChat">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
        </button>
        <button class="icon-btn" title="Close" @click="emit('close')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>

    <div ref="messagesContainer" class="ai-messages">
      <div v-if="configError" class="config-warning">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        {{ configError }}
      </div>

      <div v-if="messages.length === 0 && !isStreaming" class="empty-state">
        <div class="empty-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="1.5" stroke-linecap="round">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          </svg>
        </div>
        <p>Ask about commands, troubleshoot issues, or get scripting help.</p>
        <div class="suggestions">
          <button class="suggestion" @click="inputText = 'How do I find large files on this server?'">Find large files</button>
          <button class="suggestion" @click="inputText = 'Show me how to set up a cron job'">Set up cron job</button>
          <button class="suggestion" @click="inputText = 'How to check disk space usage?'">Check disk space</button>
        </div>
      </div>

      <template v-for="(msg, _idx) in messages" :key="_idx">
        <div class="message" :class="msg.role">
          <div class="message-header">
            <span class="role-label">{{ msg.role === 'user' ? 'You' : 'Assistant' }}</span>
          </div>
          <div class="message-body">
            <template v-for="(part, pidx) in extractCodeBlocks(msg.content)" :key="pidx">
              <p v-if="part.type === 'text'" class="text-part" v-text="part.content" />
              <div v-else class="code-block">
                <div class="code-header">
                  <span class="code-lang">{{ part.lang }}</span>
                  <div class="code-actions">
                    <button class="code-btn" title="Copy" @click="copyCode(part.content)">Copy</button>
                    <button
                      v-if="part.lang === 'bash' || part.lang === 'sh' || part.lang === ''"
                      class="code-btn run"
                      title="Run in terminal"
                      @click="runCode(part.content.trim())"
                    >Run</button>
                  </div>
                </div>
                <pre class="code-content"><code>{{ part.content }}</code></pre>
              </div>
            </template>
          </div>
        </div>
      </template>

      <div v-if="isStreaming && streamingContent" class="message assistant">
        <div class="message-header">
          <span class="role-label">Assistant</span>
          <span class="streaming-dot" />
        </div>
        <div class="message-body">
          <template v-for="(part, pidx) in extractCodeBlocks(streamingContent)" :key="pidx">
            <p v-if="part.type === 'text'" class="text-part" v-text="part.content" />
            <div v-else class="code-block">
              <div class="code-header">
                <span class="code-lang">{{ part.lang }}</span>
              </div>
              <pre class="code-content"><code>{{ part.content }}</code></pre>
            </div>
          </template>
        </div>
      </div>

      <div v-if="isStreaming && !streamingContent" class="message assistant">
        <div class="message-header">
          <span class="role-label">Assistant</span>
        </div>
        <div class="message-body thinking">
          <span class="dot" /><span class="dot" /><span class="dot" />
        </div>
      </div>
    </div>

    <div class="ai-input-area">
      <textarea
        ref="inputRef"
        v-model="inputText"
        class="ai-input"
        placeholder="Ask anything about SSH, Linux, scripting..."
        rows="1"
        :disabled="isStreaming"
        @keydown="onInputKeydown"
      />
      <button
        class="send-btn"
        :disabled="!inputText.trim() || isStreaming"
        @click="sendMessage"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="22" y1="2" x2="11" y2="13" />
          <polygon points="22 2 15 22 11 13 2 9 22 2" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.ai-chat {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  color: var(--text-primary);
  border-left: 1px solid var(--border-color);
}

.ai-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.ai-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
}

.model-badge {
  font-size: 10px;
  font-weight: 400;
  padding: 1px 6px;
  background: var(--bg-active);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
}

.ai-actions {
  display: flex;
  gap: 4px;
}

.icon-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 6px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
}

.icon-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.ai-messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.config-warning {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: color-mix(in srgb, var(--warning, #f59e0b) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--warning, #f59e0b) 40%, transparent);
  border-radius: var(--radius-md);
  font-size: 12px;
  color: var(--warning, #f59e0b);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 12px;
  text-align: center;
  padding: 24px;
}

.empty-state p {
  font-size: 13px;
  color: var(--text-muted);
  max-width: 260px;
  line-height: 1.5;
}

.empty-icon {
  padding: 12px;
  background: var(--bg-surface);
  border-radius: 12px;
}

.suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: center;
  margin-top: 4px;
}

.suggestion {
  font-size: 11px;
  padding: 4px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
}

.suggestion:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--accent);
}

.message {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.message.user {
  align-items: flex-end;
}

.message.user .message-body {
  background: var(--accent);
  color: var(--bg-primary);
  border-radius: var(--radius-md) var(--radius-md) 4px var(--radius-md);
  max-width: 85%;
}

.message.assistant .message-body {
  background: var(--bg-surface);
  border-radius: var(--radius-md) var(--radius-md) var(--radius-md) 4px;
  max-width: 95%;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.role-label {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
}

.streaming-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.message-body {
  padding: 8px 12px;
  font-size: 13px;
  line-height: 1.5;
  word-break: break-word;
  white-space: pre-wrap;
}

.message-body.thinking {
  display: flex;
  gap: 4px;
  align-items: center;
  padding: 12px 16px;
}

.thinking .dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-muted);
  animation: bounce 1.4s infinite ease-in-out both;
}

.thinking .dot:nth-child(1) { animation-delay: -0.32s; }
.thinking .dot:nth-child(2) { animation-delay: -0.16s; }

@keyframes bounce {
  0%, 80%, 100% { transform: scale(0); }
  40% { transform: scale(1); }
}

.text-part {
  margin: 0;
}

.code-block {
  margin: 6px 0;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.code-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  background: var(--bg-tertiary, var(--bg-secondary));
  font-size: 11px;
  border-bottom: 1px solid var(--border-color);
}

.code-lang {
  color: var(--text-muted);
  text-transform: uppercase;
  font-size: 10px;
  font-weight: 600;
}

.code-actions {
  display: flex;
  gap: 4px;
}

.code-btn {
  font-size: 11px;
  padding: 1px 6px;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
}

.code-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.code-btn.run {
  color: var(--accent);
  border-color: var(--accent);
}

.code-btn.run:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
}

.code-content {
  margin: 0;
  padding: 8px;
  overflow-x: auto;
  font-size: 12px;
  font-family: var(--font-mono, 'JetBrains Mono', monospace);
  background: var(--bg-primary);
  color: var(--text-primary);
}

.code-content code {
  white-space: pre;
}

.ai-input-area {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}

.ai-input {
  flex: 1;
  resize: none;
  padding: 8px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  max-height: 120px;
  line-height: 1.4;
}

.ai-input:focus {
  border-color: var(--border-focus);
}

.ai-input::placeholder {
  color: var(--text-muted);
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--accent);
  border: none;
  border-radius: var(--radius-md);
  color: var(--bg-primary);
  cursor: pointer;
  flex-shrink: 0;
}

.send-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.send-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
