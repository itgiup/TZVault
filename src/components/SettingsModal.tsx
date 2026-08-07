// src/components/SettingsModal.tsx

import { useState } from 'react';
import { changePassword, setAutoLockTimeout } from '../api/vault';
import { translateError } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import { Modal, useModalClose } from './Modal';

interface SettingsModalProps {
  onClose: () => void;
  onPasswordChanged: () => void; // vault bị lock lại sau khi đổi password -> quay về Unlock
}

const MIN_LENGTH = 12;

export function SettingsModal({ onClose, onPasswordChanged }: SettingsModalProps) {
  return (
    <Modal onClose={onClose}>
      <SettingsModalContent onPasswordChanged={onPasswordChanged} />
    </Modal>
  );
}

function SettingsModalContent({ onPasswordChanged }: { onPasswordChanged: () => void }) {
  const { t } = useI18n();
  const { requestClose, closeThen } = useModalClose();

  const TIMEOUT_OPTIONS = [
    { label: t.timeout1min, seconds: 60 },
    { label: t.timeout5min, seconds: 300 },
    { label: t.timeout15min, seconds: 900 },
    { label: t.timeout30min, seconds: 1800 },
  ];

  const [oldPassword, setOldPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [submittingPassword, setSubmittingPassword] = useState(false);
  const [passwordErrorCode, setPasswordErrorCode] = useState<unknown>(null);

  const [timeoutSeconds, setTimeoutSeconds] = useState(300);
  const [timeoutSaved, setTimeoutSaved] = useState(false);
  const [timeoutErrorCode, setTimeoutErrorCode] = useState<unknown>(null);

  const mismatch = confirmPassword.length > 0 && newPassword !== confirmPassword;
  const canSubmitPassword =
    oldPassword.length > 0 &&
    newPassword.length >= MIN_LENGTH &&
    newPassword === confirmPassword &&
    !submittingPassword;

  async function handleChangePassword(e: React.FormEvent) {
    e.preventDefault();
    if (!canSubmitPassword) return;

    setSubmittingPassword(true);
    setPasswordErrorCode(null);
    try {
      await changePassword(oldPassword, newPassword);
      closeThen(onPasswordChanged);
    } catch (err) {
      setPasswordErrorCode(err);
      setSubmittingPassword(false);
    }
  }

  async function handleSaveTimeout(seconds: number) {
    setTimeoutSeconds(seconds);
    setTimeoutErrorCode(null);
    setTimeoutSaved(false);
    try {
      await setAutoLockTimeout(seconds);
      setTimeoutSaved(true);
      setTimeout(() => setTimeoutSaved(false), 2000);
    } catch (err) {
      setTimeoutErrorCode(err);
    }
  }

  const passwordError = passwordErrorCode ? translateError(passwordErrorCode, t) : null;
  const timeoutError = timeoutErrorCode ? translateError(timeoutErrorCode, t) : null;

  return (
    <>
      <h2 className="modal-title">{t.settingsTitle}</h2>

      {/* ---------- Auto-lock timeout ---------- */}
      <div className="field">
        <label>{t.autoLockLabel}</label>
        <select value={timeoutSeconds} onChange={(e) => handleSaveTimeout(Number(e.target.value))}>
          {TIMEOUT_OPTIONS.map((opt) => (
            <option key={opt.seconds} value={opt.seconds}>
              {opt.label}
            </option>
          ))}
        </select>
        {timeoutSaved && <p className="hint-text" style={{ color: 'var(--success)' }}>{t.savedLabel}</p>}
        {timeoutError && <p className="error-text">{timeoutError}</p>}
        <p className="hint-text">{t.autoLockHint}</p>
      </div>

      <div style={{ height: 1, background: 'var(--border)', margin: '4px 0' }} />

      {/* ---------- Đổi master password ---------- */}
      <form className="auth-form" onSubmit={handleChangePassword}>
        <label style={{ fontSize: 13, fontWeight: 600 }}>{t.changePasswordTitle}</label>

        <div className="field">
          <label htmlFor="old-password">{t.oldPasswordLabel}</label>
          <input
            id="old-password"
            type="password"
            value={oldPassword}
            onChange={(e) => setOldPassword(e.target.value)}
            autoComplete="current-password"
          />
        </div>

        <div className="field">
          <label htmlFor="new-password">{t.newPasswordLabel}</label>
          <input
            id="new-password"
            type="password"
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            placeholder={t.newPasswordPlaceholder}
            autoComplete="new-password"
          />
        </div>

        <div className="field">
          <label htmlFor="confirm-new-password">{t.confirmNewPasswordLabel}</label>
          <input
            id="confirm-new-password"
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            autoComplete="new-password"
          />
          {mismatch && <p className="error-text">{t.passwordMismatch}</p>}
        </div>

        <p className="hint-text">{t.changePasswordHint}</p>

        {passwordError && <p className="error-text">{passwordError}</p>}

        <div style={{ display: 'flex', gap: 10, justifyContent: 'flex-end', marginTop: 4 }}>
          <button type="button" className="btn btn-secondary" onClick={requestClose}>
            {t.close}
          </button>
          <button type="submit" className="btn btn-primary" disabled={!canSubmitPassword}>
            {submittingPassword ? t.changingPasswordBtn : t.changePasswordBtn}
          </button>
        </div>
      </form>
    </>
  );
}
