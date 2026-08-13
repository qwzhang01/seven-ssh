import { invoke } from '@tauri-apps/api/core'

export type ExportFormat = 'json' | 'ssh_config'

export async function exportConnections(
  format: ExportFormat,
  connectionIds?: string[]
): Promise<string> {
  return invoke('export_connections', { format, connectionIds })
}

export async function exportConnectionsToFile(
  format: ExportFormat,
  path: string,
  connectionIds?: string[]
): Promise<void> {
  return invoke('export_connections_to_file', { format, path, connectionIds })
}
