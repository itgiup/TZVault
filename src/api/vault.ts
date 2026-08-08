// src/api/vault.ts
//
// Lớp duy nhất được phép gọi invoke() trực tiếp. Component chỉ import
// từ file này — giúp dễ đổi backend sau này và dễ mock khi test UI.

import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
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

export async function unlockKeyWithPassword(id: string, keyPassword: string): Promise<KeyWithSecret> {
  return invoke('cmd_unlock_key_with_password', { id, keyPassword });
}

export async function addKeyPassword(id: string, newKeyPassword: string): Promise<void> {
  return invoke('cmd_add_key_password', { id, newKeyPassword });
}

export async function removeKeyPassword(id: string, currentKeyPassword: string): Promise<void> {
  return invoke('cmd_remove_key_password', { id, currentKeyPassword });
}

export async function changeKeyPassword(
  id: string,
  currentKeyPassword: string,
  newKeyPassword: string
): Promise<void> {
  return invoke('cmd_change_key_password', { id, currentKeyPassword, newKeyPassword });
}

// ---------- Export / Import vault ----------
//
// Chọn đường dẫn (dialog native) tách riêng khỏi lệnh gọi Rust thực sự,
// để dễ test/tái dùng và để component tự quyết định khi nào hỏi đường dẫn.

/// Mở dialog "Save" để chọn nơi lưu file export. Trả về null nếu người
/// dùng bấm Cancel.
export async function pickExportDestination(dialogTitle: string, filterName: string): Promise<string | null> {
  const path = await save({
    title: dialogTitle,
    defaultPath: 'vault-backup.db',
    filters: [{ name: filterName, extensions: ['db'] }],
  });
  return path ?? null;
}

/// Mở dialog "Open" để chọn file vault cần import. Trả về null nếu
/// người dùng bấm Cancel.
export async function pickImportSource(dialogTitle: string, filterName: string): Promise<string | null> {
  const selected = await open({
    title: dialogTitle,
    multiple: false,
    filters: [{ name: filterName, extensions: ['db'] }],
  });
  if (!selected || Array.isArray(selected)) return null;
  return selected;
}

export async function exportVault(destPath: string): Promise<void> {
  return invoke('cmd_export_vault', { destPath });
}

export async function importVault(srcPath: string): Promise<void> {
  return invoke('cmd_import_vault', { srcPath });
}

// ---------- Đổi vị trí vault đang dùng (move/link) ----------

export async function getDbPath(): Promise<string> {
  return invoke('cmd_get_db_path');
}

export async function setDbPath(newPath: string, mode: 'move' | 'link'): Promise<void> {
  return invoke('cmd_set_db_path', { newPath, mode });
}

/// Dialog "Save" để chọn nơi DI CHUYỂN vault hiện tại tới.
export async function pickMoveDestination(dialogTitle: string, filterName: string): Promise<string | null> {
  const path = await save({
    title: dialogTitle,
    defaultPath: 'vault.db',
    filters: [{ name: filterName, extensions: ['db'] }],
  });
  return path ?? null;
}

/// Dialog "Open" để chọn 1 file vault ĐÃ CÓ SẴN để liên kết trực tiếp
/// (không copy).
export async function pickLinkSource(dialogTitle: string, filterName: string): Promise<string | null> {
  const selected = await open({
    title: dialogTitle,
    multiple: false,
    filters: [{ name: filterName, extensions: ['db'] }],
  });
  if (!selected || Array.isArray(selected)) return null;
  return selected;
}

/// Rút gọn lỗi trả về từ Rust (thường đã là string dễ đọc) thành message
/// hiển thị được cho người dùng, phòng trường hợp lỗi là object khác.
export function toErrorMessage(err: unknown): string {
  if (typeof err === 'string') return err;
  if (err instanceof Error) return err.message;
  return 'Đã xảy ra lỗi không xác định';
}
