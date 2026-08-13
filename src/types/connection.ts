export interface ConnectionInfo {
  id: string
  name: string
  host: string
  port: number
  username: string
  auth_method: AuthMethod
  private_key_path?: string
  group_id?: string
  tags: string
  color?: string
  charset: string
  keepalive_interval: number
  startup_command?: string
  proxy_jump_id?: string
  sort_order: number
  is_favorite: boolean
  note?: string
  created_at: string
  updated_at: string
}

export type AuthMethod = 'password' | 'publickey' | 'agent' | 'keyboard-interactive'

export interface CreateConnectionRequest {
  name: string
  host: string
  port?: number
  username: string
  auth_method: AuthMethod
  password?: string
  private_key_path?: string
  passphrase?: string
  group_id?: string
  tags?: string[]
  color?: string
  charset?: string
  keepalive_interval?: number
  startup_command?: string
  proxy_jump_id?: string
  note?: string
}

export interface UpdateConnectionRequest {
  id: string
  name?: string
  host?: string
  port?: number
  username?: string
  auth_method?: AuthMethod
  password?: string
  private_key_path?: string
  passphrase?: string
  group_id?: string
  tags?: string[]
  color?: string
  charset?: string
  keepalive_interval?: number
  startup_command?: string
  proxy_jump_id?: string
  is_favorite?: boolean
  note?: string
  sort_order?: number
}

export interface GroupInfo {
  id: string
  name: string
  parent_id?: string
  sort_order: number
  color?: string
  icon?: string
  created_at: string
  updated_at: string
}

export interface CreateGroupRequest {
  name: string
  parent_id?: string
  color?: string
  icon?: string
}

export type ConnectionStatus = 'idle' | 'connecting' | 'connected' | 'disconnected' | 'error'

export type SplitDirection = 'none' | 'horizontal' | 'vertical' | 'grid'

export interface TerminalPane {
  id: string
  connectionId: string
  connectionName: string
  sessionId?: string
  status: ConnectionStatus
  host: string
}

export interface SessionTab {
  id: string
  panes: TerminalPane[]
  splitDirection: SplitDirection
  syncInput: boolean
}

export interface Snippet {
  id: string
  name: string
  command: string
  category?: string
  description?: string
}

export interface RemoteFileEntry {
  name: string
  path: string
  is_dir: boolean
  size: number
  permissions: number
  modified?: number
  owner?: number
  group?: number
}

export interface ImportedConnection {
  host_alias: string
  hostname: string
  port: number
  username: string
  identity_file?: string
  proxy_jump?: string
}

export interface ImportedConnectionSave {
  name: string
  host: string
  port: number
  username: string
  auth_method: string
  private_key_path?: string
  group_id?: string
}

export interface LocalFileEntry {
  name: string
  path: string
  is_dir: boolean
  size: number
  modified?: number
}

export type TransferDirection = 'upload' | 'download'
export type TransferStatus = 'pending' | 'active' | 'completed' | 'failed' | 'cancelled' | 'paused'

export interface TransferItem {
  id: string
  direction: TransferDirection
  localPath: string
  remotePath: string
  fileName: string
  totalBytes: number
  bytesTransferred: number
  progress: number
  status: TransferStatus
  sftpSessionId: string
  speed?: number
  startedAt?: number
  error?: string
}

export type SortField = 'name' | 'size' | 'date'
export type SortOrder = 'asc' | 'desc'

export interface DragTransferPayload {
  source: 'local' | 'remote'
  entries: Array<{ name: string; path: string; is_dir: boolean; size: number }>
}
