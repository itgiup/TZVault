// src/components/AddKeyModal.tsx

import { useState } from 'react';
import { addKey } from '../api/vault';
import { translateError, getKeyTypeLabels } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import type { KeyType } from '../types';

interface AddKeyModalProps {
  onClose: () => void;
  onAdded: () => void;
}

export function AddKeyModal({ onClose, onAdded }: AddKeyModalProps) {
  const { t } = useI18n();
  const keyTypeLabels = getKeyTypeLabels(t);

  const [name, setName] = useState('');
  const [keyType, setKeyType] = useState<KeyType>('ssh');
  const [secretValue, setSecretValue] = useState('');
  const [tagsInput, setTagsInput] = useState('');
  const [notes, setNotes] = useState('');
  const [wantsExtraPassword, setWantsExtraPassword] = useState(false);
  const [extraPassword, setExtraPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [errorCode, setErrorCode] = useState<unknown>(null);

  const extraPasswordTooShort = wantsExtraPassword && extraPassword.length > 0 && extraPassword.length < 8;
  const canSubmit =
    name.trim().length > 0 &&
    secretValue.length > 0 &&
    (!wantsExtraPassword || extraPassword.length >= 8) &&
    !submitting;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!canSubmit) return;

    setSubmitting(true);
    setErrorCode(null);

    const tags = tagsInput
      .split(',')
      .map((tg) => tg.trim())
      .filter(Boolean);

    try {
      await addKey({
        name: name.trim(),
        key_type: keyType,
        secret_value: secretValue,
        tags,
        notes: notes.trim() || null,
        extra_password: wantsExtraPassword ? extraPassword : null,
      });
      onAdded();
    } catch (err) {
      setErrorCode(err);
      setSubmitting(false);
    }
  }

  const error = errorCode ? translateError(errorCode, t) : null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-card" onClick={(e) => e.stopPropagation()}>
        <h2 className="modal-title">{t.addKeyTitle}</h2>

        <form className="auth-form" onSubmit={handleSubmit}>
          <div className="field">
            <label htmlFor="key-name">{t.nameLabel}</label>
            <input
              id="key-name"
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t.namePlaceholder}
            />
          </div>

          <div className="field">
            <label htmlFor="key-type">{t.keyTypeLabel}</label>
            <select id="key-type" value={keyType} onChange={(e) => setKeyType(e.target.value as KeyType)}>
              {(Object.entries(keyTypeLabels) as [KeyType, string][]).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </div>

          <div className="field">
            <label htmlFor="key-secret">{t.keyContentLabel}</label>
            <textarea
              id="key-secret"
              value={secretValue}
              onChange={(e) => setSecretValue(e.target.value)}
              placeholder={t.secretPlaceholder}
              spellCheck={false}
            />
          </div>

          <div className="field">
            <label htmlFor="key-tags">{t.tagsLabel}</label>
            <input
              id="key-tags"
              value={tagsInput}
              onChange={(e) => setTagsInput(e.target.value)}
              placeholder={t.tagsPlaceholder}
            />
          </div>

          <div className="field">
            <label htmlFor="key-notes">{t.notesOptionalLabel}</label>
            <input
              id="key-notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder={t.notesOptionalPlaceholder}
            />
          </div>

          <div style={{ height: 1, background: 'var(--border)', margin: '4px 0' }} />

          <label
            style={{
              display: 'flex',
              gap: 8,
              alignItems: 'center',
              fontSize: 13,
              color: 'var(--text)',
              cursor: 'pointer',
            }}
          >
            <input
              type="checkbox"
              checked={wantsExtraPassword}
              onChange={(e) => {
                setWantsExtraPassword(e.target.checked);
                if (!e.target.checked) setExtraPassword('');
              }}
              style={{ width: 'auto' }}
            />
            {t.extraPasswordCheckbox}
          </label>

          {wantsExtraPassword && (
            <div className="field">
              <label htmlFor="extra-password">{t.extraPasswordFieldLabel}</label>
              <input
                id="extra-password"
                type="password"
                value={extraPassword}
                onChange={(e) => setExtraPassword(e.target.value)}
                placeholder={t.extraPasswordFieldPlaceholder}
                autoComplete="new-password"
              />
              {extraPasswordTooShort && (
                <p className="hint-text">{t.needMoreChars(8 - extraPassword.length)}</p>
              )}
            </div>
          )}

          {error && <p className="error-text">{error}</p>}

          <div style={{ display: 'flex', gap: 10, justifyContent: 'flex-end', marginTop: 4 }}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              {t.cancel}
            </button>
            <button type="submit" className="btn btn-primary" disabled={!canSubmit}>
              {submitting ? t.savingKeyBtn : t.saveKeyBtn}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
