import { invoke } from '@tauri-apps/api/core'

export async function checkHasMasterPassword(): Promise<boolean> {
  return invoke('check_has_master_password')
}

export async function setMasterPassword(password: string): Promise<void> {
  return invoke('set_master_password', { password })
}

export async function verifyMasterPassword(password: string): Promise<boolean> {
  return invoke('verify_master_password', { password })
}

export async function lockApp(): Promise<void> {
  return invoke('lock_app')
}

export async function checkLocked(): Promise<boolean> {
  return invoke('check_locked')
}

export async function touchActivity(): Promise<void> {
  return invoke('touch_activity')
}

export async function checkAutoLock(timeoutSecs: number): Promise<boolean> {
  return invoke('check_auto_lock', { timeoutSecs })
}
