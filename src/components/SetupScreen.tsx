// src/components/SetupScreen.tsx

import { useState } from 'react';
import { setupVault, pickImportSource, importVault } from '../api/vault';
import { translateError } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import { Dial } from './Dial';

interface SetupScreenProps {
  onSetupComplete: () => void;
  onImportComplete: () => void;
}

const MIN_LENGTH = 12;

export function SetupScreen({ onSetupComplete, onImportComplete }: SetupScreenProps) {
  const { t } = useI18n();
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [errorCode, setErrorCode] = useState<unknown>(null);
  const [understood, setUnderstood] = useState(false);

  const [importing, setImporting] = useState(false);
  const [importErrorCode, setImportErrorCode] = useState<unknown>(null);

  const tooShort = password.length > 0 && password.length < MIN_LENGTH;
  const mismatch = confirm.length > 0 && password !== confirm;
  const canSubmit =
    password.length >= MIN_LENGTH && password === confirm && understood && !submitting;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!canSubmit) return;

    setSubmitting(true);
    setErrorCode(null);
    try {
      await setupVault(password);
      onSetupComplete();
    } catch (err) {
      setErrorCode(err);
      setSubmitting(false);
    }
  }

  async function handleImport() {
    setImportErrorCode(null);

    const srcPath = await pickImportSource(t.importDialogTitle, t.vaultFileFilterName);
    if (!srcPath) return; // người dùng bấm Cancel trên dialog

    setImporting(true);
    try {
      await importVault(srcPath);
      onImportComplete();
    } catch (err) {
      setImportErrorCode(err);
      setImporting(false);
    }
  }

  const error = errorCode ? translateError(errorCode, t) : null;
  const importError = importErrorCode ? translateError(importErrorCode, t) : null;

  return (
    <div className="auth-screen">
      <div className="auth-card">
        <Dial spinning={submitting || importing} />

        <div>
          <h1 className="auth-title">{t.setupTitle}</h1>
          <p className="auth-subtitle">{t.setupSubtitle}</p>
        </div>

        <form className="auth-form" onSubmit={handleSubmit}>
          <div className="field">
            <label htmlFor="password">{t.masterPasswordLabel}</label>
            <input
              id="password"
              type="password"
              autoFocus
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t.masterPasswordPlaceholderSetup}
              autoComplete="new-password"
            />
            {tooShort && <p className="hint-text">{t.needMoreChars(MIN_LENGTH - password.length)}</p>}
          </div>

          <div className="field">
            <label htmlFor="confirm">{t.confirmPasswordLabel}</label>
            <input
              id="confirm"
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              placeholder={t.confirmPasswordPlaceholder}
              autoComplete="new-password"
            />
            {mismatch && <p className="error-text">{t.passwordMismatch}</p>}
          </div>

          <label
            style={{
              display: 'flex',
              gap: 8,
              alignItems: 'flex-start',
              fontSize: 12,
              color: 'var(--text-dim)',
              cursor: 'pointer',
            }}
          >
            <input
              type="checkbox"
              checked={understood}
              onChange={(e) => setUnderstood(e.target.checked)}
              style={{ width: 'auto', marginTop: 2 }}
            />
            {t.understandCheckbox}
          </label>

          {error && <p className="error-text">{error}</p>}

          <button type="submit" className="btn btn-primary" disabled={!canSubmit}>
            {submitting ? t.creatingVaultBtn : t.createVaultBtn}
          </button>
        </form>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12, width: '100%' }}>
          <div style={{ flex: 1, height: 1, background: 'var(--border)' }} />
          <span className="hint-text">{t.orDivider}</span>
          <div style={{ flex: 1, height: 1, background: 'var(--border)' }} />
        </div>

        <button className="btn btn-secondary" onClick={handleImport} disabled={importing} style={{ width: '100%' }}>
          {importing ? t.importingVaultBtn : t.importVaultBtn}
        </button>
        {importError && <p className="error-text">{importError}</p>}
      </div>
    </div>
  );
}