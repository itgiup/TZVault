// src/components/VaultScreen.tsx

import { useEffect, useMemo, useState } from 'react';
import { listKeys, lockVault } from '../api/vault';
import { getKeyTypeLabels } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import type { KeySummary } from '../types';
import type { Language } from '../i18n/translations';
import { KeyDetail } from './KeyDetail';
import { AddKeyModal } from './AddKeyModal';
import { SettingsModal } from './SettingsModal';
import { ThemeToggle } from './ThemeToggle';
import { LanguageToggle } from './LanguageToggle';
import { Dial } from './Dial';
import type { Theme } from '../hooks/useTheme';

interface VaultScreenProps {
  onLocked: () => void;
  theme: Theme;
  onToggleTheme: () => void;
}

// Phải khớp với timeout mặc định ở backend (vault/state.rs -> 5 phút),
// dùng để reset UI về màn hình Unlock khi backend tự động lock.
const IDLE_CHECK_INTERVAL_MS = 15_000;

export function VaultScreen({ onLocked, theme, onToggleTheme }: VaultScreenProps) {
  const { t, language, setLanguage } = useI18n();
  const keyTypeLabels = getKeyTypeLabels(t);

  const [keys, setKeys] = useState<KeySummary[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setError(null);
    try {
      const data = await listKeys();
      setKeys(data);
    } catch (err) {
      // Nếu backend đã auto-lock (idle timeout), listKeys sẽ trả lỗi
      // ERR_VAULT_LOCKED -> quay lại màn hình Unlock.
      onLocked();
      return;
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, IDLE_CHECK_INTERVAL_MS);
    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleLock() {
    await lockVault();
    onLocked();
  }

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return keys;
    return keys.filter(
      (k) =>
        k.name.toLowerCase().includes(q) ||
        k.tags.some((tag) => tag.toLowerCase().includes(q)) ||
        keyTypeLabels[k.key_type].toLowerCase().includes(q)
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [keys, search, language]);

  const selected = keys.find((k) => k.id === selectedId) ?? null;

  return (
    <div className="vault-app">
      <header className="vault-header">
        <div className="vault-brand">
          <Dial />
          {t.vaultBrand}
        </div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <LanguageToggle
            language={language}
            onChange={(lang: Language) => setLanguage(lang)}
            floating={false}
          />
          <ThemeToggle theme={theme} onToggle={onToggleTheme} floating={false} />
          <button className="btn btn-secondary" onClick={() => setShowSettings(true)}>
            {t.settingsBtn}
          </button>
          <button className="btn btn-secondary" onClick={handleLock}>
            {t.lockBtn}
          </button>
        </div>
      </header>

      <div className={`vault-layout${selected ? ' has-selection' : ''}`}>
        <aside className="vault-list-pane">
          <div className="vault-list-header">
            <input
              className="vault-search"
              placeholder={t.searchPlaceholder}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <button className="btn btn-primary" onClick={() => setShowAddModal(true)}>
              {t.addKeyBtn}
            </button>
          </div>

          <div className="vault-list">
            {loading && <p className="hint-text" style={{ padding: 12 }}>{t.loadingKeys}</p>}

            {!loading && filtered.length === 0 && keys.length === 0 && (
              <p className="hint-text" style={{ padding: 12 }}>{t.emptyNoKeys}</p>
            )}

            {!loading && filtered.length === 0 && keys.length > 0 && (
              <p className="hint-text" style={{ padding: 12 }}>{t.emptyNoResults}</p>
            )}

            {filtered.map((key) => (
              <div
                key={key.id}
                className={`vault-list-item${key.id === selectedId ? ' active' : ''}`}
                onClick={() => setSelectedId(key.id)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && setSelectedId(key.id)}
              >
                <span className="vault-list-item-name">
                  {key.name}
                  {key.has_extra_password && (
                    <span title={t.protectedBadge} style={{ marginLeft: 6, fontSize: 12 }}>
                      🔒
                    </span>
                  )}
                </span>
                <div className="vault-list-item-meta">
                  <span className="type-badge">{keyTypeLabels[key.key_type]}</span>
                  {key.tags.length > 0 && <span>{key.tags.join(', ')}</span>}
                </div>
              </div>
            ))}
          </div>
        </aside>

        <main className="vault-detail-pane">
          {error && <p className="error-text">{error}</p>}

          {selected ? (
            <>
              <button className="mobile-back-btn" onClick={() => setSelectedId(null)}>
                {t.backToListBtn}
              </button>
              <KeyDetail
                summary={selected}
                onDeleted={() => {
                  setSelectedId(null);
                  refresh();
                }}
                onUpdated={refresh}
              />
            </>
          ) : (
            <div className="vault-empty-state">
              <Dial />
              <p>{t.selectKeyPrompt}</p>
            </div>
          )}
        </main>
      </div>

      {showAddModal && (
        <AddKeyModal
          onClose={() => setShowAddModal(false)}
          onAdded={() => {
            setShowAddModal(false);
            refresh();
          }}
        />
      )}

      {showSettings && (
        <SettingsModal
          onClose={() => setShowSettings(false)}
          onPasswordChanged={() => {
            setShowSettings(false);
            onLocked();
          }}
        />
      )}
    </div>
  );
}
