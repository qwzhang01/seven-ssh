import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
}

export interface AiConfig {
  provider: string
  api_key: string | null
  base_url: string | null
  model: string
  temperature: number
  max_tokens: number
}

export interface ConnectionContext {
  hostname: string | null
  username: string | null
  os_hint: string | null
  cwd: string | null
}

export interface DangerWarning {
  level: 'critical' | 'warning'
  message: string
}

export async function aiChat(
  messages: ChatMessage[],
  config: AiConfig,
  context?: ConnectionContext,
): Promise<string> {
  return invoke('ai_chat', { messages, config, context: context ?? null })
}

export async function onAiRequestId(
  callback: (requestId: number) => void,
): Promise<UnlistenFn> {
  return listen<number>('ai-request-id', (event) => {
    callback(event.payload)
  })
}

export async function onAiChunk(
  requestId: number,
  callback: (chunk: string) => void,
): Promise<UnlistenFn> {
  return listen<string>(`ai-chunk-${requestId}`, (event) => {
    callback(event.payload)
  })
}

export async function onAiComplete(
  requestId: number,
  callback: (full: string) => void,
): Promise<UnlistenFn> {
  return listen<string>(`ai-complete-${requestId}`, (event) => {
    callback(event.payload)
  })
}

export async function aiCheckDanger(
  command: string,
): Promise<DangerWarning | null> {
  return invoke('ai_check_danger', { command })
}

export async function aiRedact(text: string): Promise<string> {
  return invoke('ai_redact', { text })
}

export async function aiGetConfig(): Promise<AiConfig> {
  return invoke('ai_get_config')
}

export async function aiSaveConfig(config: AiConfig): Promise<void> {
  return invoke('ai_save_config', { config })
}
