// src/App.tsx

import { useEffect, useState } from 'react';
import { vaultExists } from './api/vault';
import { translateError } from './i18n/translations';
import { LanguageProvider, useI18n } from './i18n/LanguageContext';
import { APP_STATES, type AppState } from './types';
import { SetupScreen } from './components/SetupScreen';
import { UnlockScreen } from './components/UnlockScreen';
import { VaultScreen } from './components/VaultScreen';
import { Dial } from './components/Dial';
import { ThemeToggle } from './components/ThemeToggle';
import { LanguageToggle } from './components/LanguageToggle';
import { useTheme } from './hooks/useTheme';
import './styles/vault.css';

function AppInner() {
  const [state, setState] = useState<AppState>(APP_STATES.loading);
  const [initErrorCode, setInitErrorCode] = useState<unknown>(null);
  const { theme, toggleTheme } = useTheme();
  const { t, language, setLanguage } = useI18n();

  useEffect(() => {
    checkVaultStatus();
  }, []);

  async function checkVaultStatus() {
    try {
      const exists = await vaultExists();
      setState(exists ? APP_STATES.locked : APP_STATES.needs_setup);
    } catch (err) {
      setInitErrorCode(err);
      setState(APP_STATES.init_error);
    }
  }

  const initError = initErrorCode ? translateError(initErrorCode, t) : null;

  // Nút nổi dùng chung cho các màn hình Setup/Unlock/loading/init_error
  // (VaultScreen tự đặt 2 nút này inline trong header riêng).
  const floatingControls = (
    <>
      <LanguageToggle language={language} onChange={setLanguage} />
      <ThemeToggle theme={theme} onToggle={toggleTheme} />
    </>
  );

  if (state === APP_STATES.loading) {
    return (
      <div className="vault-app">
        {floatingControls}
        <div className="auth-screen">
          <Dial spinning />
        </div>
      </div>
    );
  }

  if (state === APP_STATES.init_error) {
    return (
      <div className="vault-app">
        {floatingControls}
        <div className="auth-screen">
          <div className="auth-card">
            <Dial variant="error" />
            <p className="error-text">
              {t.connectErrorPrefix}
              {initError}
            </p>
            <button className="btn btn-secondary" onClick={checkVaultStatus}>
              {t.tryAgain}
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (state === APP_STATES.needs_setup) {
    return (
      <div className="vault-app">
        {floatingControls}
        <SetupScreen
          onSetupComplete={() => setState(APP_STATES.unlocked)}
          onImportComplete={() => setState(APP_STATES.locked)}
        />
      </div>
    );
  }

  if (state === APP_STATES.locked) {
    return (
      <div className="vault-app">
        {floatingControls}
        <UnlockScreen onUnlocked={() => setState(APP_STATES.unlocked)} />
      </div>
    );
  }

  return (
    <VaultScreen
      onLocked={() => setState(APP_STATES.locked)}
      theme={theme}
      onToggleTheme={toggleTheme}
    />
  );
}

function App() {
  return (
    <LanguageProvider>
      <AppInner />
    </LanguageProvider>
  );
}

export default App;