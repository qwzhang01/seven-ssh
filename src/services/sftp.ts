import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type { RemoteFileEntry } from '@/types'

export async function sftpOpen(connectionId: string): Promise<string> {
  return invoke('sftp_open', { connectionId })
}

export async function sftpClose(sessionId: string): Promise<void> {
  return invoke('sftp_close', { sessionId })
}

export async function sftpListDir(sessionId: string, path: string): Promise<RemoteFileEntry[]> {
  return invoke('sftp_list_dir', { sessionId, path })
}

export async function sftpMkdir(sessionId: string, path: string): Promise<void> {
  return invoke('sftp_mkdir', { sessionId, path })
}

export async function sftpRemove(sessionId: string, path: string, isDir: boolean): Promise<void> {
  return invoke('sftp_remove', { sessionId, path, isDir })
}

export async function sftpRename(sessionId: string, oldPath: string, newPath: string): Promise<void> {
  return invoke('sftp_rename', { sessionId, oldPath, newPath })
}

export async function sftpRealpath(sessionId: string, path: string): Promise<string> {
  return invoke('sftp_realpath', { sessionId, path })
}

export async function sftpUpload(
  sessionId: string,
  localPath: string,
  remotePath: string,
  transferId: string,
): Promise<void> {
  return invoke('sftp_upload', { sessionId, localPath, remotePath, transferId })
}

export async function sftpDownload(
  sessionId: string,
  remotePath: string,
  localPath: string,
  transferId: string,
): Promise<void> {
  return invoke('sftp_download', { sessionId, remotePath, localPath, transferId })
}

export async function sftpStat(
  sessionId: string,
  path: string,
): Promise<{ size: number; is_dir: boolean; modified?: number }> {
  return invoke('sftp_stat', { sessionId, path })
}

export async function sftpReadFile(
  sessionId: string,
  remotePath: string,
): Promise<string> {
  return invoke('sftp_read_file', { sessionId, remotePath })
}

export async function sftpWriteFile(
  sessionId: string,
  remotePath: string,
  content: string,
): Promise<void> {
  return invoke('sftp_write_file', { sessionId, remotePath, content })
}

export async function sftpUploadWithResume(
  sessionId: string,
  localPath: string,
  remotePath: string,
  transferId: string,
  offset: number,
): Promise<void> {
  return invoke('sftp_upload_resume', { sessionId, localPath, remotePath, transferId, offset })
}

export async function sftpDownloadWithResume(
  sessionId: string,
  remotePath: string,
  localPath: string,
  transferId: string,
  offset: number,
): Promise<void> {
  return invoke('sftp_download_resume', { sessionId, remotePath, localPath, transferId, offset })
}

export interface TransferProgress {
  transfer_id: string
  bytes_transferred: number
  total_bytes: number
  progress: number
}

export function onTransferProgress(
  transferId: string,
  callback: (data: TransferProgress) => void,
): Promise<UnlistenFn> {
  return listen<TransferProgress>(`transfer-progress-${transferId}`, (event) => {
    callback(event.payload)
  })
}

export function onTransferComplete(
  transferId: string,
  callback: (data: { transfer_id: string; status: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ transfer_id: string; status: string }>(
    `transfer-complete-${transferId}`,
    (event) => {
      callback(event.payload)
    },
  )
}
