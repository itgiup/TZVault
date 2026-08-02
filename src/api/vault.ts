// src/api/vault.ts
//
// Lớp duy nhất được phép gọi invoke() trực tiếp. Component chỉ import
// từ file này — giúp dễ đổi backend sau này và dễ mock khi test UI.

import { invoke } from '@tauri-apps/api/core';
import type { KeySummary, KeyWithSecret, NewKeyInput } from '../types';

export async function vaultExists(): Promise<boolean> {
  return invoke('cmd_vault_exists');
}

export async function setupVault(password: string): Promise<void> {
  return invoke('cmd_setup_vault', { password });
}

export async function unlockVault(password: string): Promise<void> {
  return invoke('cmd_unlock_vault', { password });
}

export async function lockVault(): Promise<void> {
  return invoke('cmd_lock_vault');
}

export async function isUnlocked(): Promise<boolean> {
  return invoke('cmd_is_unlocked');
}

export async function changePassword(oldPassword: string, newPassword: string): Promise<void> {
  return invoke('cmd_change_password', { oldPassword, newPassword });
}

export async function setAutoLockTimeout(seconds: number): Promise<void> {
  return invoke('cmd_set_auto_lock_timeout', { seconds });
}

export async function listKeys(): Promise<KeySummary[]> {
  return invoke('cmd_list_keys');
}

export async function getKeySecret(id: string): Promise<KeyWithSecret> {
  return invoke('cmd_get_key_secret', { id });
}

export async function addKey(input: NewKeyInput): Promise<string> {
  return invoke('cmd_add_key', { input });
}

export async function deleteKey(id: string): Promise<void> {
  return invoke('cmd_delete_key', { id });
}

/// Rút gọn lỗi trả về từ Rust (thường đã là string dễ đọc) thành message
/// hiển thị được cho người dùng, phòng trường hợp lỗi là object khác.
export function toErrorMessage(err: unknown): string {
  if (typeof err === 'string') return err;
  if (err instanceof Error) return err.message;
  return 'Đã xảy ra lỗi không xác định';
}
