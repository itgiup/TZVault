// src/components/KeyPasswordModal.tsx
//
// Modal dùng chung cho 4 thao tác liên quan tới mật khẩu riêng của 1 key:
//   - 'unlock': nhập mật khẩu riêng để xem nội dung key đang bị khóa
//   - 'add':    thêm mật khẩu riêng cho key hiện chưa được bảo vệ
//   - 'remove': gỡ mật khẩu riêng (cần đúng mật khẩu hiện tại)
//   - 'change': đổi mật khẩu riêng (cần đúng mật khẩu hiện tại + mật khẩu mới)

import { useState } from 'react';
import {
  unlockKeyWithPassword,
  addKeyPassword,
  removeKeyPassword,
  changeKeyPassword,
} from '../api/vault';
import { translateError } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import type { KeyWithSecret } from '../types';
import { Modal, useModalClose } from './Modal';

export type KeyPasswordMode = 'unlock' | 'add' | 'remove' | 'change';

interface KeyPasswordModalProps {
  mode: KeyPasswordMode;
  keyId: string;
  onClose: () => void;
  /** 'unlock' trả về secret đã giải mã; các mode khác chỉ cần biết đã xong. */
  onSuccess: (secret?: KeyWithSecret) => void;
}

export function KeyPasswordModal({ mode, keyId, onClose, onSuccess }: KeyPasswordModalProps) {
  return (
    <Modal onClose={onClose}>
      <KeyPasswordModalContent mode={mode} keyId={keyId} onSuccess={onSuccess} />
    </Modal>
  );
}

const MIN_KEY_PASSWORD_LENGTH = 8;

function KeyPasswordModalContent({
  mode,
  keyId,
  onSuccess,
}: {
  mode: KeyPasswordMode;
  keyId: string;
  onSuccess: (secret?: KeyWithSecret) => void;
}) {
  const { t } = useI18n();
  const { requestClose, closeThen } = useModalClose();

  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [errorCode, setErrorCode] = useState<unknown>(null);

  const error = errorCode ? translateError(errorCode, t) : null;

  const title =
    mode === 'unlock'
      ? t.keyPasswordRequiredTitle
      : mode === 'add'
        ? t.addKeyPasswordTitle
        : mode === 'remove'
          ? t.removeKeyPasswordTitle
          : t.changeKeyPasswordTitle;

  const needsCurrentPassword = mode === 'unlock' || mode === 'remove' || mode === 'change';
  const needsNewPassword = mode === 'add' || mode === 'change';

  const mismatch = needsNewPassword && confirmPassword.length > 0 && newPassword !== confirmPassword;
  const newPasswordTooShort = needsNewPassword && newPassword.length > 0 && newPassword.length < MIN_KEY_PASSWORD_LENGTH;

  const canSubmit =
    !submitting &&
    (!needsCurrentPassword || currentPassword.length > 0) &&
    (!needsNewPassword || (newPassword.length >= MIN_KEY_PASSWORD_LENGTH && newPassword === confirmPassword));

  const submitLabel =
    mode === 'unlock'
      ? (submitting ? t.unlockingKeyBtn : t.unlockKeyBtn)
      : mode === 'add'
        ? (submitting ? t.savingBtn : t.addKeyPasswordBtn)
        : mode === 'remove'
          ? (submitting ? t.removingBtn : t.removeKeyPasswordBtn)
          : (submitting ? t.changingBtn : t.changeKeyPasswordBtn);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!canSubmit) return;

    setSubmitting(true);
    setErrorCode(null);

    try {
      switch (mode) {
        case 'unlock': {
          const secret = await unlockKeyWithPassword(keyId, currentPassword);
          closeThen(() => onSuccess(secret));
          return;
        }
        case 'add': {
          await addKeyPassword(keyId, newPassword);
          closeThen(() => onSuccess());
          return;
        }
        case 'remove': {
          await removeKeyPassword(keyId, currentPassword);
          closeThen(() => onSuccess());
          return;
        }
        case 'change': {
          await changeKeyPassword(keyId, currentPassword, newPassword);
          closeThen(() => onSuccess());
          return;
        }
      }
    } catch (err) {
      setErrorCode(err);
      setSubmitting(false);
    }
  }

  return (
    <>
      <h2 className="modal-title">{title}</h2>

      {mode === 'unlock' && <p className="hint-text">{t.keyPasswordRequiredSubtitle}</p>}

      <form className="auth-form" onSubmit={handleSubmit}>
        {needsCurrentPassword && (
          <div className="field">
            <label htmlFor="current-key-password">
              {mode === 'unlock' ? t.keyPasswordLabel : t.currentKeyPasswordLabel}
            </label>
            <input
              id="current-key-password"
              type="password"
              autoFocus
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              autoComplete="current-password"
            />
          </div>
        )}

        {needsNewPassword && (
          <>
            <div className="field">
              <label htmlFor="new-key-password">{t.newKeyPasswordLabel}</label>
              <input
                id="new-key-password"
                type="password"
                autoFocus={!needsCurrentPassword}
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder={t.extraPasswordFieldPlaceholder}
                autoComplete="new-password"
              />
              {newPasswordTooShort && (
                <p className="hint-text">{t.needMoreChars(MIN_KEY_PASSWORD_LENGTH - newPassword.length)}</p>
              )}
            </div>

            <div className="field">
              <label htmlFor="confirm-key-password">{t.confirmKeyPasswordLabel}</label>
              <input
                id="confirm-key-password"
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                autoComplete="new-password"
              />
              {mismatch && <p className="error-text">{t.passwordMismatch}</p>}
            </div>
          </>
        )}

        {error && <p className="error-text">{error}</p>}

        <div style={{ display: 'flex', gap: 10, justifyContent: 'flex-end', marginTop: 4 }}>
          <button type="button" className="btn btn-secondary" onClick={requestClose}>
            {t.cancel}
          </button>
          <button type="submit" className="btn btn-primary" disabled={!canSubmit}>
            {submitLabel}
          </button>
        </div>
      </form>
    </>
  );
}
