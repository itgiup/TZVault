// src/types.ts
// Khớp chính xác với các struct trong src-tauri/src/models.rs

export type KeyType = 'ssh' | 'crypto_wallet' | 'pgp' | 'api_key' | 'other';

export interface KeySummary {
  id: string;
  name: string;
  key_type: KeyType;
  tags: string[];
  notes: string | null;
  created_at: number;
  updated_at: number;
  has_extra_password: boolean;
}

export interface KeyWithSecret {
  id: string;
  name: string;
  key_type: KeyType;
  secret_value: string;
  tags: string[];
  notes: string | null;
}

export interface NewKeyInput {
  name: string;
  key_type: KeyType;
  secret_value: string;
  tags: string[];
  notes: string | null;
  extra_password: string | null;
}

// Nhãn hiển thị cho từng KeyType giờ lấy qua getKeyTypeLabels(t) trong
// src/i18n/translations.ts (đổi theo ngôn ngữ đang chọn), không hardcode ở đây nữa.

// ---------- App state (điều phối luồng Setup -> Unlock -> Vault) ----------

export const APP_STATES = {
  loading: 'loading',
  needs_setup: 'needs_setup',
  locked: 'locked',
  unlocked: 'unlocked',
  init_error: 'init_error',
} as const;

export type AppState = (typeof APP_STATES)[keyof typeof APP_STATES];
