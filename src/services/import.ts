import { invoke } from '@tauri-apps/api/core'

import type { ImportedConnection, ImportedConnectionSave } from '@/types'

export async function importSshConfig(filePath?: string): Promise<ImportedConnection[]> {
  return invoke('import_ssh_config', { filePath })
}

export async function importPuttySessions(): Promise<ImportedConnection[]> {
  return invoke('import_putty_sessions')
}

export async function importXshellSessions(filePath: string): Promise<ImportedConnection[]> {
  return invoke('import_xshell_sessions', { filePath })
}

export async function saveImportedConnections(connections: ImportedConnectionSave[]): Promise<number> {
  return invoke('save_imported_connections', { connections })
}
