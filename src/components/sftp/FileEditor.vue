<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import * as monaco from 'monaco-editor'

import * as sftpService from '@/services/sftp'

const props = defineProps<{
  sessionId: string
  remotePath: string
  fileName: string
  fileSize: number
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const editorContainer = ref<HTMLDivElement>()
let editor: monaco.editor.IStandaloneCodeEditor | null = null
const loading = ref(false)
const saving = ref(false)
const error = ref('')
const modified = ref(false)
const showSizeWarning = ref(false)
const originalContent = ref('')

const SIZE_WARNING_THRESHOLD = 5 * 1024 * 1024

const extensionLanguageMap: Record<string, string> = {
  js: 'javascript',
  jsx: 'javascript',
  ts: 'typescript',
  tsx: 'typescript',
  py: 'python',
  rb: 'ruby',
  rs: 'rust',
  go: 'go',
  java: 'java',
  c: 'c',
  cpp: 'cpp',
  h: 'c',
  hpp: 'cpp',
  cs: 'csharp',
  php: 'php',
  swift: 'swift',
  kt: 'kotlin',
  scala: 'scala',
  sh: 'shell',
  bash: 'shell',
  zsh: 'shell',
  fish: 'shell',
  ps1: 'powershell',
  html: 'html',
  htm: 'html',
  css: 'css',
  scss: 'scss',
  less: 'less',
  json: 'json',
  xml: 'xml',
  yaml: 'yaml',
  yml: 'yaml',
  toml: 'ini',
  ini: 'ini',
  conf: 'ini',
  cfg: 'ini',
  md: 'markdown',
  sql: 'sql',
  graphql: 'graphql',
  dockerfile: 'dockerfile',
  lua: 'lua',
  r: 'r',
  perl: 'perl',
  pl: 'perl',
  vue: 'html',
  svelte: 'html',
}

function getLanguage(filename: string): string {
  const lower = filename.toLowerCase()
  if (lower === 'dockerfile' || lower === 'makefile') return lower
  const ext = lower.split('.').pop() ?? ''
  return extensionLanguageMap[ext] ?? 'plaintext'
}

async function loadFile() {
  loading.value = true
  error.value = ''
  try {
    const content = await sftpService.sftpReadFile(props.sessionId, props.remotePath)
    originalContent.value = content
    if (editor) {
      editor.setValue(content)
      modified.value = false
    }
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

async function saveFile() {
  if (!editor || saving.value) return
  saving.value = true
  error.value = ''
  try {
    const content = editor.getValue()
    await sftpService.sftpWriteFile(props.sessionId, props.remotePath, content)
    originalContent.value = content
    modified.value = false
    emit('saved')
  } catch (err) {
    error.value = String(err)
  } finally {
    saving.value = false
  }
}

function proceedOpen() {
  showSizeWarning.value = false
  loadFile()
}

function initEditor() {
  if (!editorContainer.value) return
  const lang = getLanguage(props.fileName)

  editor = monaco.editor.create(editorContainer.value, {
    value: '',
    language: lang,
    theme: 'vs-dark',
    fontSize: 13,
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
    minimap: { enabled: true },
    wordWrap: 'on',
    lineNumbers: 'on',
    renderWhitespace: 'selection',
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 2,
    padding: { top: 8 },
  })

  editor.onDidChangeModelContent(() => {
    modified.value = editor!.getValue() !== originalContent.value
  })

  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    saveFile()
  })
}

onMounted(async () => {
  if (props.fileSize > SIZE_WARNING_THRESHOLD) {
    showSizeWarning.value = true
    return
  }
  await nextTick()
  initEditor()
  loadFile()
})

watch(() => props.remotePath, async () => {
  if (showSizeWarning.value) return
  if (!editor) {
    await nextTick()
    initEditor()
  }
  if (editor) {
    const lang = getLanguage(props.fileName)
    const model = editor.getModel()
    if (model) {
      monaco.editor.setModelLanguage(model, lang)
    }
  }
  loadFile()
})

onBeforeUnmount(() => {
  editor?.dispose()
  editor = null
})
</script>

<template>
  <div class="file-editor">
    <div class="editor-header">
      <span class="editor-title" :title="remotePath">
        <span class="editor-icon">📝</span>
        {{ fileName }}
        <span v-if="modified" class="modified-dot">●</span>
      </span>
      <div class="editor-actions">
        <span v-if="saving" class="saving-label">Saving...</span>
        <button
          class="editor-btn save"
          :disabled="!modified || saving || loading"
          @click="saveFile"
          title="Save (Ctrl+S)"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M12.5 14h-9a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1h6.5L13.5 5.5V13a1 1 0 0 1-1 1z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M5.5 14V9h5v5M5.5 2v3h4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Save
        </button>
        <button class="editor-btn" @click="emit('close')" title="Close">✕</button>
      </div>
    </div>

    <div v-if="error" class="editor-error">{{ error }}</div>

    <!-- Size warning -->
    <div v-if="showSizeWarning" class="size-warning">
      <div class="warning-content">
        <span class="warning-icon">⚠️</span>
        <div>
          <p class="warning-title">Large file warning</p>
          <p class="warning-text">
            This file is {{ (fileSize / 1024 / 1024).toFixed(1) }} MB.
            Opening large files may slow down the editor.
          </p>
        </div>
      </div>
      <div class="warning-actions">
        <button class="warning-btn cancel" @click="emit('close')">Cancel</button>
        <button class="warning-btn proceed" @click="proceedOpen">Open Anyway</button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-else-if="loading" class="editor-loading">Loading file...</div>

    <!-- Editor -->
    <div v-show="!showSizeWarning && !loading" ref="editorContainer" class="editor-container" />
  </div>
</template>

<style scoped>
.file-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.editor-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.editor-title {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.editor-icon { font-size: 14px; }

.modified-dot {
  color: var(--accent);
  font-size: 10px;
}

.editor-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.saving-label {
  font-size: 11px;
  color: var(--text-muted);
}

.editor-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  min-height: 28px;
  transition: all var(--transition-fast);
}

.editor-btn:hover { color: var(--text-primary); background: var(--bg-hover); }
.editor-btn:disabled { opacity: 0.4; cursor: default; }
.editor-btn.save:hover:not(:disabled) { color: var(--success); }

.editor-error {
  padding: 6px 8px;
  background: rgba(255, 80, 80, 0.1);
  color: var(--error);
  font-size: 12px;
  border-bottom: 1px solid var(--border-color);
}

.editor-container {
  flex: 1;
  min-height: 0;
}

.editor-loading {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
}

.size-warning {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 20px;
  padding: 24px;
}

.warning-content {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.warning-icon { font-size: 24px; }

.warning-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--warning);
  margin-bottom: 6px;
}

.warning-text {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.warning-actions {
  display: flex;
  gap: 8px;
}

.warning-btn {
  padding: 6px 16px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.warning-btn.cancel {
  background: var(--bg-surface);
  color: var(--text-secondary);
}
.warning-btn.cancel:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.warning-btn.proceed {
  background: var(--warning);
  color: var(--bg-primary);
  border-color: var(--warning);
}
.warning-btn.proceed:hover {
  opacity: 0.9;
}
</style>
