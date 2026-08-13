import { invoke } from '@tauri-apps/api/core'

export interface KeyGenResult {
  private_key_path: string
  public_key_path: string
  public_key_text: string
  fingerprint: string
}

export interface LocalKeyInfo {
  name: string
  path: string
  key_type: string
  fingerprint: string
  bits: number | null
  comment: string
  has_public_key: boolean
  public_key_path: string
  created: string
}

export interface SecurityEvent {
  id: string
  event_type: string
  details: string | null
  timestamp: string
}

export async function generateKeyPair(
  keyType: string,
  bits?: number,
  passphrase?: string,
  comment?: string,
  savePath?: string,
): Promise<KeyGenResult> {
  return invoke('generate_key_pair', {
    keyType,
    bits: bits ?? null,
    passphrase: passphrase ?? null,
    comment: comment ?? null,
    savePath: savePath ?? null,
  })
}

export async function listLocalKeys(): Promise<LocalKeyInfo[]> {
  return invoke('list_local_keys')
}

export async function deleteKey(path: string): Promise<void> {
  return invoke('delete_key', { path })
}

export async function deployPublicKey(connectionId: string, publicKeyPath: string): Promise<void> {
  return invoke('deploy_public_key', { connectionId, publicKeyPath })
}

export async function logSecurityEvent(eventType: string, details?: string): Promise<void> {
  return invoke('log_security_event', { eventType, details: details ?? null })
}

export async function getSecurityEvents(limit?: number): Promise<SecurityEvent[]> {
  return invoke('get_security_events', { limit: limit ?? null })
}

export async function clearClipboard(): Promise<void> {
  return invoke('clear_clipboard')
}

let clipboardTimer: ReturnType<typeof setTimeout> | null = null

export function scheduleClipboardClear(timeoutSecs: number): void {
  if (clipboardTimer) {
    clearTimeout(clipboardTimer)
    clipboardTimer = null
  }
  if (timeoutSecs <= 0) return
  clipboardTimer = setTimeout(async () => {
    try {
      await clearClipboard()
    } catch { /* non-critical */ }
    clipboardTimer = null
  }, timeoutSecs * 1000)
}
