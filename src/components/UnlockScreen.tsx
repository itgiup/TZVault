// src/components/UnlockScreen.tsx

import { useState } from 'react';
import { unlockVault } from '../api/vault';
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

  const error = errorCode ? translateError(errorCode, t) : null;

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
      </div>
    </div>
  );
}
