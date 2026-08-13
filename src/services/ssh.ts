import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type {
  ConnectionInfo,
  CreateConnectionRequest,
  UpdateConnectionRequest,
  GroupInfo,
  CreateGroupRequest,
} from '@/types'

export async function listConnections(): Promise<ConnectionInfo[]> {
  return invoke('list_connections')
}

export async function getConnection(id: string): Promise<ConnectionInfo> {
  return invoke('get_connection', { id })
}

export async function createConnection(request: CreateConnectionRequest): Promise<ConnectionInfo> {
  return invoke('create_connection', { request })
}

export async function updateConnection(request: UpdateConnectionRequest): Promise<ConnectionInfo> {
  return invoke('update_connection', { request })
}

export async function deleteConnection(id: string): Promise<void> {
  return invoke('delete_connection', { id })
}

export async function listGroups(): Promise<GroupInfo[]> {
  return invoke('list_groups')
}

export async function createGroup(request: CreateGroupRequest): Promise<GroupInfo> {
  return invoke('create_group', { request })
}

export async function updateGroup(
  id: string,
  name?: string,
  color?: string,
  icon?: string,
  sortOrder?: number,
): Promise<void> {
  return invoke('update_group', { id, name, color, icon, sortOrder })
}

export async function deleteGroup(id: string): Promise<void> {
  return invoke('delete_group', { id })
}

export async function sshConnect(connectionId: string): Promise<string> {
  return invoke('ssh_connect', { connectionId })
}

export async function sshDisconnect(sessionId: string): Promise<void> {
  return invoke('ssh_disconnect', { sessionId })
}

export async function sshWrite(sessionId: string, data: number[]): Promise<void> {
  return invoke('ssh_write', { sessionId, data })
}

export async function sshResize(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke('ssh_resize', { sessionId, cols, rows })
}

export function onTerminalOutput(
  sessionId: string,
  callback: (data: number[]) => void,
): Promise<UnlistenFn> {
  return listen<number[]>(`terminal-output-${sessionId}`, (event) => {
    callback(event.payload)
  })
}

export async function sessionLogToggle(
  sessionId: string,
  enable: boolean,
  logPath?: string,
): Promise<void> {
  return invoke('session_log_toggle', { sessionId, enable, logPath })
}

export function onTerminalStatus(
  sessionId: string,
  callback: (status: string) => void,
): Promise<UnlistenFn> {
  return listen<string>(`terminal-status-${sessionId}`, (event) => {
    callback(event.payload)
  })
}
