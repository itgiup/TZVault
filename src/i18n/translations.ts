// src/i18n/translations.ts
//
// Toàn bộ chuỗi hiển thị trong UI. Mọi component lấy chữ qua hook
// useI18n() thay vì hardcode trực tiếp, để đổi ngôn ngữ đồng bộ toàn app.
//
// QUY TẮC: en và vi phải implement đúng interface Translations —
// TypeScript sẽ báo lỗi ngay nếu thiếu key khi thêm ngôn ngữ mới.

export interface Translations {
  // Chung
  loading: string;
  cancel: string;
  close: string;
  tryAgain: string;

  // Setup screen
  setupTitle: string;
  setupSubtitle: string;
  masterPasswordLabel: string;
  masterPasswordPlaceholderSetup: string;
  confirmPasswordLabel: string;
  confirmPasswordPlaceholder: string;
  passwordMismatch: string;
  needMoreChars: (n: number) => string;
  understandCheckbox: string;
  createVaultBtn: string;
  creatingVaultBtn: string;

  // Unlock screen
  unlockTitle: string;
  unlockSubtitle: string;
  masterPasswordPlaceholderUnlock: string;
  unlockBtn: string;
  unlockingBtn: string;

  // Vault screen
  vaultBrand: string;
  settingsBtn: string;
  lockBtn: string;
  searchPlaceholder: string;
  addKeyBtn: string;
  loadingKeys: string;
  emptyNoKeys: string;
  emptyNoResults: string;
  selectKeyPrompt: string;
  backToListBtn: string;

  // Key detail
  keyContentLabel: string;
  showBtn: string;
  hideBtn: string;
  decryptingBtn: string;
  copyBtn: string;
  copiedBtn: string;
  deleteBtn: string;
  confirmDeleteBtn: string;
  cancelDeleteBtn: string;
  notesLabel: string;
  clipboardHint: (seconds: number) => string;

  // Add key modal
  addKeyTitle: string;
  nameLabel: string;
  namePlaceholder: string;
  keyTypeLabel: string;
  secretPlaceholder: string;
  tagsLabel: string;
  tagsPlaceholder: string;
  notesOptionalLabel: string;
  notesOptionalPlaceholder: string;
  saveKeyBtn: string;
  savingKeyBtn: string;
  extraPasswordCheckbox: string;
  extraPasswordFieldLabel: string;
  extraPasswordFieldPlaceholder: string;

  // Per-key password: protected badge + unlock flow
  protectedBadge: string;
  keyPasswordRequiredTitle: string;
  keyPasswordRequiredSubtitle: string;
  keyPasswordLabel: string;
  unlockKeyBtn: string;
  unlockingKeyBtn: string;

  // Per-key password: management (add/remove/change)
  addKeyPasswordBtn: string;
  removeKeyPasswordBtn: string;
  changeKeyPasswordBtn: string;
  addKeyPasswordTitle: string;
  removeKeyPasswordTitle: string;
  changeKeyPasswordTitle: string;
  newKeyPasswordLabel: string;
  confirmKeyPasswordLabel: string;
  currentKeyPasswordLabel: string;
  savingBtn: string;
  removingBtn: string;
  changingBtn: string;

  // Settings modal
  settingsTitle: string;
  autoLockLabel: string;
  autoLockHint: string;
  savedLabel: string;
  timeout1min: string;
  timeout5min: string;
  timeout15min: string;
  timeout30min: string;
  changePasswordTitle: string;
  oldPasswordLabel: string;
  newPasswordLabel: string;
  newPasswordPlaceholder: string;
  confirmNewPasswordLabel: string;
  changePasswordHint: string;
  changePasswordBtn: string;
  changingPasswordBtn: string;
  exportVaultTitle: string;
  exportVaultHint: string;
  exportVaultBtn: string;
  exportingVaultBtn: string;
  exportVaultSuccess: string;
  exportDialogTitle: string;
  vaultFileFilterName: string;
  importVaultBtn: string;
  importingVaultBtn: string;
  importVaultSuccess: string;
  importDialogTitle: string;
  orDivider: string;

  vaultLocationTitle: string;
  vaultLocationHint: string;
  currentLocationLabel: string;
  moveVaultBtn: string;
  movingVaultBtn: string;
  linkVaultBtn: string;
  linkingVaultBtn: string;
  moveDialogTitle: string;
  linkDialogTitle: string;
  vaultRelocateWarning: string;

  // Key types
  keyTypeSsh: string;
  keyTypeCryptoWallet: string;
  keyTypePgp: string;
  keyTypeApiKey: string;
  keyTypeOther: string;

  // App init
  connectErrorPrefix: string;

  // Error codes trả về từ backend (Rust) - xem src-tauri/src/commands/
  errorVaultExists: string;
  errorPasswordTooShort: string;
  errorInvalidPassword: string;
  errorVaultLocked: string;
  errorNameEmpty: string;
  errorSecretEmpty: string;
  errorKeyNotFound: string;
  errorTimeoutTooShort: string;
  errorGeneric: string;
  errorInternal: string;
  errorKeyPasswordTooShort: string;
  errorInvalidKeyPassword: string;
  errorKeyAlreadyProtected: string;
  errorInvalidVaultFile: string;
}

const en: Translations = {
  loading: 'Loading...',
  cancel: 'Cancel',
  close: 'Close',
  tryAgain: 'Try again',

  setupTitle: 'Create Storage Vault',
  setupSubtitle:
    'Set a master password to encrypt all your private keys. No one — not even us — can recover your data if you forget this password.',
  masterPasswordLabel: 'Master password',
  masterPasswordPlaceholderSetup: 'At least 12 characters',
  confirmPasswordLabel: 'Confirm master password',
  confirmPasswordPlaceholder: 'Re-enter the password above',
  passwordMismatch: "Passwords don't match",
  needMoreChars: (n) => `${n} more character${n > 1 ? 's' : ''} needed`,
  understandCheckbox:
    'I understand that if I forget my master password, all data in the vault cannot be recovered.',
  createVaultBtn: 'Create Vault',
  creatingVaultBtn: 'Creating vault...',

  unlockTitle: 'Unlock Vault',
  unlockSubtitle: 'Enter your master password to access your private keys.',
  masterPasswordPlaceholderUnlock: 'Enter master password',
  unlockBtn: 'Unlock',
  unlockingBtn: 'Unlocking...',

  vaultBrand: 'Vault',
  settingsBtn: 'Settings',
  lockBtn: 'Lock',
  searchPlaceholder: 'Search by name, tag, type...',
  addKeyBtn: '+ Add new key',
  loadingKeys: 'Loading...',
  emptyNoKeys: 'No keys yet. Click "Add new key" to get started.',
  emptyNoResults: 'No matching results found.',
  selectKeyPrompt: 'Select a key from the list on the left to view details',
  backToListBtn: '← List',

  keyContentLabel: 'Key content',
  showBtn: 'Show',
  hideBtn: 'Hide',
  decryptingBtn: 'Decrypting...',
  copyBtn: 'Copy',
  copiedBtn: 'Copied',
  deleteBtn: 'Delete',
  confirmDeleteBtn: 'Confirm delete',
  cancelDeleteBtn: 'Cancel',
  notesLabel: 'Notes',
  clipboardHint: (s) => `Clipboard will be cleared automatically ${s}s after copying.`,

  addKeyTitle: 'Add Private Key',
  nameLabel: 'Name',
  namePlaceholder: 'e.g. SSH - Production server',
  keyTypeLabel: 'Key type',
  secretPlaceholder: 'Paste your private key or secret here',
  tagsLabel: 'Tags (comma separated)',
  tagsPlaceholder: 'production, aws, backend',
  notesOptionalLabel: 'Notes (non-sensitive)',
  notesOptionalPlaceholder: 'Optional',
  saveKeyBtn: 'Save to vault',
  savingKeyBtn: 'Saving...',
  extraPasswordCheckbox: 'Protect with an extra password',
  extraPasswordFieldLabel: 'Password for this key',
  extraPasswordFieldPlaceholder: 'At least 8 characters',

  protectedBadge: 'Extra protected',
  keyPasswordRequiredTitle: 'This key is extra protected',
  keyPasswordRequiredSubtitle: 'Enter this key\'s password to view its content.',
  keyPasswordLabel: 'Key password',
  unlockKeyBtn: 'Unlock key',
  unlockingKeyBtn: 'Unlocking...',

  addKeyPasswordBtn: '🔒 Add extra password',
  removeKeyPasswordBtn: 'Remove extra password',
  changeKeyPasswordBtn: 'Change extra password',
  addKeyPasswordTitle: 'Add extra password',
  removeKeyPasswordTitle: 'Remove extra password',
  changeKeyPasswordTitle: 'Change extra password',
  newKeyPasswordLabel: 'New key password',
  confirmKeyPasswordLabel: 'Confirm key password',
  currentKeyPasswordLabel: 'Current key password',
  savingBtn: 'Saving...',
  removingBtn: 'Removing...',
  changingBtn: 'Changing...',

  settingsTitle: 'Settings',
  autoLockLabel: 'Auto-lock after inactivity',
  autoLockHint: 'Only applies to the current session, resets to 5 minutes each time you reopen the app.',
  savedLabel: 'Saved',
  timeout1min: '1 minute',
  timeout5min: '5 minutes (default)',
  timeout15min: '15 minutes',
  timeout30min: '30 minutes',
  changePasswordTitle: 'Change master password',
  oldPasswordLabel: 'Current master password',
  newPasswordLabel: 'New master password',
  newPasswordPlaceholder: 'At least 12 characters',
  confirmNewPasswordLabel: 'Confirm new master password',
  changePasswordHint: "After a successful change, you'll need to unlock the vault again with the new password.",
  changePasswordBtn: 'Change password',
  changingPasswordBtn: 'Changing...',
  exportVaultTitle: 'Export Vault',
  exportVaultHint: 'Save your entire vault as one file — copy it to a USB drive, cloud storage, or another computer.',
  exportVaultBtn: 'Export Vault',
  exportingVaultBtn: 'Exporting...',
  exportVaultSuccess: 'Vault exported successfully',
  exportDialogTitle: 'Save vault backup',
  vaultFileFilterName: 'Vault Database',
  importVaultBtn: 'Import existing vault',
  importingVaultBtn: 'Importing...',
  importVaultSuccess: 'Vault imported successfully',
  importDialogTitle: 'Select a vault file to import',
  orDivider: 'or',

  vaultLocationTitle: 'Vault Location',
  vaultLocationHint: 'Move your vault to a synced folder (Dropbox, Google Drive...) or link to a vault file already stored elsewhere.',
  currentLocationLabel: 'Current location',
  moveVaultBtn: 'Move vault to new location...',
  movingVaultBtn: 'Moving...',
  linkVaultBtn: 'Use a different vault file...',
  linkingVaultBtn: 'Linking...',
  moveDialogTitle: 'Choose new location for vault',
  linkDialogTitle: 'Select an existing vault file',
  vaultRelocateWarning: "You'll need to unlock again after this.",

  keyTypeSsh: 'SSH Key',
  keyTypeCryptoWallet: 'Crypto Wallet',
  keyTypePgp: 'PGP Key',
  keyTypeApiKey: 'API Key',
  keyTypeOther: 'Other',

  connectErrorPrefix: 'Could not connect to the vault: ',

  errorVaultExists: 'A vault already exists',
  errorPasswordTooShort: 'Master password must be at least 12 characters',
  errorInvalidPassword: 'Incorrect master password',
  errorVaultLocked: 'Vault is locked',
  errorNameEmpty: 'Name cannot be empty',
  errorSecretEmpty: 'Key content cannot be empty',
  errorKeyNotFound: 'Key not found',
  errorTimeoutTooShort: 'Minimum auto-lock time is 30 seconds',
  errorGeneric: 'Something went wrong',
  errorInternal: 'An unexpected error occurred. Please try again or restart the app.',
  errorKeyPasswordTooShort: 'Key password must be at least 8 characters',
  errorInvalidKeyPassword: 'Incorrect key password',
  errorKeyAlreadyProtected: 'This key already has an extra password',
  errorInvalidVaultFile: "This doesn't look like a valid vault file",
};

const vi: Translations = {
  loading: 'Đang tải...',
  cancel: 'Hủy',
  close: 'Đóng',
  tryAgain: 'Thử lại',

  setupTitle: 'Tạo két lưu trữ',
  setupSubtitle:
    'Đặt master password để mã hóa toàn bộ private key của bạn. Không ai — kể cả chúng tôi — có thể khôi phục nếu bạn quên mật khẩu này.',
  masterPasswordLabel: 'Master password',
  masterPasswordPlaceholderSetup: 'Ít nhất 12 ký tự',
  confirmPasswordLabel: 'Xác nhận master password',
  confirmPasswordPlaceholder: 'Nhập lại mật khẩu ở trên',
  passwordMismatch: 'Hai mật khẩu chưa khớp',
  needMoreChars: (n) => `Cần thêm ${n} ký tự nữa`,
  understandCheckbox:
    'Tôi hiểu rằng nếu quên master password, toàn bộ dữ liệu trong két sẽ không thể khôi phục.',
  createVaultBtn: 'Tạo két lưu trữ',
  creatingVaultBtn: 'Đang tạo két...',

  unlockTitle: 'Mở khóa két',
  unlockSubtitle: 'Nhập master password để truy cập private key của bạn.',
  masterPasswordPlaceholderUnlock: 'Nhập master password',
  unlockBtn: 'Mở khóa',
  unlockingBtn: 'Đang mở khóa...',

  vaultBrand: 'Két lưu trữ',
  settingsBtn: 'Cài đặt',
  lockBtn: 'Khóa lại',
  searchPlaceholder: 'Tìm theo tên, tag, loại...',
  addKeyBtn: '+ Thêm key mới',
  loadingKeys: 'Đang tải...',
  emptyNoKeys: 'Chưa có key nào. Bấm "Thêm key mới" để bắt đầu.',
  emptyNoResults: 'Không tìm thấy kết quả phù hợp.',
  selectKeyPrompt: 'Chọn một key từ danh sách bên trái để xem chi tiết',
  backToListBtn: '← Danh sách',

  keyContentLabel: 'Nội dung key',
  showBtn: 'Hiện',
  hideBtn: 'Ẩn',
  decryptingBtn: 'Đang giải mã...',
  copyBtn: 'Sao chép',
  copiedBtn: 'Đã sao chép',
  deleteBtn: 'Xóa',
  confirmDeleteBtn: 'Xác nhận xóa',
  cancelDeleteBtn: 'Thôi',
  notesLabel: 'Ghi chú',
  clipboardHint: (s) => `Clipboard sẽ tự xóa ${s}s sau khi sao chép.`,

  addKeyTitle: 'Thêm private key',
  nameLabel: 'Tên',
  namePlaceholder: 'VD: SSH - Server production',
  keyTypeLabel: 'Loại key',
  secretPlaceholder: 'Dán private key hoặc secret vào đây',
  tagsLabel: 'Tags (phân cách bằng dấu phẩy)',
  tagsPlaceholder: 'production, aws, backend',
  notesOptionalLabel: 'Ghi chú (không nhạy cảm)',
  notesOptionalPlaceholder: 'Tùy chọn',
  saveKeyBtn: 'Lưu vào két',
  savingKeyBtn: 'Đang lưu...',
  extraPasswordCheckbox: 'Bảo vệ thêm bằng mật khẩu riêng',
  extraPasswordFieldLabel: 'Mật khẩu riêng cho key này',
  extraPasswordFieldPlaceholder: 'Ít nhất 8 ký tự',

  protectedBadge: 'Bảo vệ thêm',
  keyPasswordRequiredTitle: 'Key này được bảo vệ thêm',
  keyPasswordRequiredSubtitle: 'Nhập mật khẩu riêng của key này để xem nội dung.',
  keyPasswordLabel: 'Mật khẩu riêng',
  unlockKeyBtn: 'Mở khóa key',
  unlockingKeyBtn: 'Đang mở khóa...',

  addKeyPasswordBtn: '🔒 Thêm mật khẩu riêng',
  removeKeyPasswordBtn: 'Gỡ mật khẩu riêng',
  changeKeyPasswordBtn: 'Đổi mật khẩu riêng',
  addKeyPasswordTitle: 'Thêm mật khẩu riêng',
  removeKeyPasswordTitle: 'Gỡ mật khẩu riêng',
  changeKeyPasswordTitle: 'Đổi mật khẩu riêng',
  newKeyPasswordLabel: 'Mật khẩu riêng mới',
  confirmKeyPasswordLabel: 'Xác nhận mật khẩu riêng',
  currentKeyPasswordLabel: 'Mật khẩu riêng hiện tại',
  savingBtn: 'Đang lưu...',
  removingBtn: 'Đang gỡ...',
  changingBtn: 'Đang đổi...',

  settingsTitle: 'Cài đặt',
  autoLockLabel: 'Tự động khóa sau khi không hoạt động',
  autoLockHint: 'Chỉ áp dụng cho phiên làm việc hiện tại, reset về 5 phút mỗi lần mở lại app.',
  savedLabel: 'Đã lưu',
  timeout1min: '1 phút',
  timeout5min: '5 phút (mặc định)',
  timeout15min: '15 phút',
  timeout30min: '30 phút',
  changePasswordTitle: 'Đổi master password',
  oldPasswordLabel: 'Master password hiện tại',
  newPasswordLabel: 'Master password mới',
  newPasswordPlaceholder: 'Ít nhất 12 ký tự',
  confirmNewPasswordLabel: 'Xác nhận master password mới',
  changePasswordHint: 'Sau khi đổi thành công, bạn sẽ cần mở khóa lại két bằng mật khẩu mới.',
  changePasswordBtn: 'Đổi mật khẩu',
  changingPasswordBtn: 'Đang đổi...',
  exportVaultTitle: 'Xuất Vault',
  exportVaultHint: 'Lưu toàn bộ vault thành 1 file — copy vào USB, cloud, hoặc mang sang máy khác.',
  exportVaultBtn: 'Xuất Vault',
  exportingVaultBtn: 'Đang xuất...',
  exportVaultSuccess: 'Đã xuất vault thành công',
  exportDialogTitle: 'Lưu bản sao vault',
  vaultFileFilterName: 'Vault Database',
  importVaultBtn: 'Nhập vault đã có',
  importingVaultBtn: 'Đang nhập...',
  importVaultSuccess: 'Đã nhập vault thành công',
  importDialogTitle: 'Chọn file vault cần nhập',
  orDivider: 'hoặc',

  vaultLocationTitle: 'Vị trí Vault',
  vaultLocationHint: 'Di chuyển vault vào thư mục đồng bộ (Dropbox, Google Drive...) hoặc liên kết tới 1 file vault đã có sẵn ở nơi khác.',
  currentLocationLabel: 'Vị trí hiện tại',
  moveVaultBtn: 'Di chuyển vault đến vị trí mới...',
  movingVaultBtn: 'Đang di chuyển...',
  linkVaultBtn: 'Dùng 1 file vault khác...',
  linkingVaultBtn: 'Đang liên kết...',
  moveDialogTitle: 'Chọn vị trí mới cho vault',
  linkDialogTitle: 'Chọn file vault đã có sẵn',
  vaultRelocateWarning: 'Bạn sẽ cần mở khóa lại sau khi thực hiện.',

  keyTypeSsh: 'SSH Key',
  keyTypeCryptoWallet: 'Ví Crypto',
  keyTypePgp: 'PGP Key',
  keyTypeApiKey: 'API Key',
  keyTypeOther: 'Khác',

  connectErrorPrefix: 'Không thể kết nối tới két lưu trữ: ',

  errorVaultExists: 'Vault đã tồn tại',
  errorPasswordTooShort: 'Master password cần ít nhất 12 ký tự',
  errorInvalidPassword: 'Master password không đúng',
  errorVaultLocked: 'Két đang bị khóa',
  errorNameEmpty: 'Tên không được để trống',
  errorSecretEmpty: 'Nội dung key không được để trống',
  errorKeyNotFound: 'Không tìm thấy key',
  errorTimeoutTooShort: 'Thời gian auto-lock tối thiểu là 30 giây',
  errorGeneric: 'Đã xảy ra lỗi không xác định',
  errorInternal: 'Đã xảy ra lỗi ngoài dự kiến. Vui lòng thử lại hoặc khởi động lại app.',
  errorKeyPasswordTooShort: 'Mật khẩu riêng cần ít nhất 8 ký tự',
  errorInvalidKeyPassword: 'Mật khẩu riêng không đúng',
  errorKeyAlreadyProtected: 'Key này đã có mật khẩu riêng rồi',
  errorInvalidVaultFile: 'File này không phải file vault hợp lệ',
};

export type Language = 'en' | 'vi';

export const LANGUAGE_NAMES: Record<Language, string> = {
  en: 'English',
  vi: 'Tiếng Việt',
};

export const translations: Record<Language, Translations> = { en, vi };

export function getKeyTypeLabels(t: Translations) {
  return {
    ssh: t.keyTypeSsh,
    crypto_wallet: t.keyTypeCryptoWallet,
    pgp: t.keyTypePgp,
    api_key: t.keyTypeApiKey,
    other: t.keyTypeOther,
  } as const;
}

/// Dịch mã lỗi trả về từ backend Rust (VD "ERR_INVALID_PASSWORD") sang
/// câu hiển thị đúng ngôn ngữ hiện tại.
///
/// Fallback: nếu backend lỡ trả về 1 mã dạng "ERR_*" mà frontend chưa
/// map (VD thêm mã mới ở Rust nhưng quên thêm ở đây) -> hiện errorGeneric
/// thay vì hiện thẳng mã lỗi thô ra UI. Chỉ những giá trị KHÔNG theo
/// định dạng "ERR_*" (VD lỗi mạng/IPC không đến từ backend của ta) mới
/// hiện nguyên văn, vì đó có thể là thông tin hữu ích để debug.
export function translateError(err: unknown, t: Translations): string {
  const raw = typeof err === 'string' ? err : err instanceof Error ? err.message : '';

  const knownErrors: Record<string, string> = {
    ERR_VAULT_EXISTS: t.errorVaultExists,
    ERR_PASSWORD_TOO_SHORT: t.errorPasswordTooShort,
    ERR_INVALID_PASSWORD: t.errorInvalidPassword,
    ERR_VAULT_LOCKED: t.errorVaultLocked,
    ERR_NAME_EMPTY: t.errorNameEmpty,
    ERR_SECRET_EMPTY: t.errorSecretEmpty,
    ERR_KEY_NOT_FOUND: t.errorKeyNotFound,
    ERR_TIMEOUT_TOO_SHORT: t.errorTimeoutTooShort,
    ERR_INTERNAL: t.errorInternal,
    ERR_KEY_PASSWORD_TOO_SHORT: t.errorKeyPasswordTooShort,
    ERR_INVALID_KEY_PASSWORD: t.errorInvalidKeyPassword,
    ERR_KEY_ALREADY_PROTECTED: t.errorKeyAlreadyProtected,
    ERR_INVALID_VAULT_FILE: t.errorInvalidVaultFile,
  };

  if (raw in knownErrors) return knownErrors[raw];
  if (raw.startsWith('ERR_')) {
    console.warn(`[i18n] Unmapped error code from backend: ${raw}`);
    return t.errorGeneric;
  }
  return raw || t.errorGeneric;
}
