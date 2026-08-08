// src/components/UnlockScreen.tsx

import { useEffect, useState } from 'react';
import { unlockVault, getDbPath, setDbPath, pickLinkSource } from '../api/vault';
import { translateError } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import { Dial } from './Dial';

interface UnlockScreenProps {
  onUnlocked: () => void;
}

// Tăng dần độ trễ sau mỗi lần sai — giảm tốc độ brute-force cục bộ.
const DELAY_STEPS_MS = [0, 500, 1000, 2000, 4000];

export function UnlockScreen({ onUnlocked }: UnlockScreenProps) {
  const { t } = useI18n();
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [errorCode, setErrorCode] = useState<unknown>(null);
  const [failCount, setFailCount] = useState(0);

  const [currentDbPath, setCurrentDbPath] = useState<string | null>(null);
  const [switchingDb, setSwitchingDb] = useState(false);
  const [switchErrorCode, setSwitchErrorCode] = useState<unknown>(null);

  useEffect(() => {
    getDbPath()
      .then(setCurrentDbPath)
      .catch(() => setCurrentDbPath(null));
  }, []);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!password || submitting) return;

    setSubmitting(true);
    setErrorCode(null);

    const delay = DELAY_STEPS_MS[Math.min(failCount, DELAY_STEPS_MS.length - 1)];
    if (delay > 0) {
      await new Promise((resolve) => setTimeout(resolve, delay));
    }

    try {
      await unlockVault(password);
      onUnlocked();
    } catch (err) {
      setErrorCode(err);
      setFailCount((c) => c + 1);
      setPassword('');
      setSubmitting(false);
    }
  }

  async function handleSwitchDb() {
    setSwitchErrorCode(null);
    const srcPath = await pickLinkSource(t.linkDialogTitle, t.vaultFileFilterName);
    if (!srcPath) return; // bấm Cancel trên dialog

    setSwitchingDb(true);
    try {
      await setDbPath(srcPath, 'link');
      setCurrentDbPath(srcPath);
      setPassword('');
      setErrorCode(null);
      setFailCount(0);
    } catch (err) {
      setSwitchErrorCode(err);
    } finally {
      setSwitchingDb(false);
    }
  }

  const error = errorCode ? translateError(errorCode, t) : null;
  const switchError = switchErrorCode ? translateError(switchErrorCode, t) : null;

  return (
    <div className="auth-screen">
      <div className="auth-card">
        <Dial spinning={submitting} variant={error ? 'error' : 'neutral'} />

        <div>
          <h1 className="auth-title">{t.unlockTitle}</h1>
          <p className="auth-subtitle">{t.unlockSubtitle}</p>
        </div>

        <form className="auth-form" onSubmit={handleSubmit}>
          <div className="field">
            <label htmlFor="unlock-password">{t.masterPasswordLabel}</label>
            <input
              id="unlock-password"
              type="password"
              autoFocus
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t.masterPasswordPlaceholderUnlock}
              autoComplete="current-password"
            />
          </div>

          {error && <p className="error-text">{error}</p>}

          <button type="submit" className="btn btn-primary" disabled={!password || submitting}>
            {submitting ? t.unlockingBtn : t.unlockBtn}
          </button>
        </form>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12, width: '100%' }}>
          <div style={{ flex: 1, height: 1, background: 'var(--border)' }} />
          <span className="hint-text">{t.orDivider}</span>
          <div style={{ flex: 1, height: 1, background: 'var(--border)' }} />
        </div>

        <button
          className="btn btn-secondary"
          onClick={handleSwitchDb}
          disabled={switchingDb}
          style={{ width: '100%' }}
        >
          {switchingDb ? t.linkingVaultBtn : t.linkVaultBtn}
        </button>

        {switchError && <p className="error-text">{switchError}</p>}

        {currentDbPath && (
          <p className="hint-text" style={{ wordBreak: 'break-all', textAlign: 'center' }}>
            {currentDbPath}
          </p>
        )}
      </div>
    </div>
  );
}
