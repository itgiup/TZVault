// src/components/KeyDetail.tsx

import { useEffect, useState } from 'react';
import { deleteKey, getKeySecret } from '../api/vault';
import { translateError, getKeyTypeLabels } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import type { KeySummary, KeyWithSecret } from '../types';

interface KeyDetailProps {
  summary: KeySummary;
  onDeleted: () => void;
}

// Xóa clipboard sau khoảng thời gian này để tránh lộ qua clipboard manager khác.
const CLIPBOARD_CLEAR_MS = 20_000;

export function KeyDetail({ summary, onDeleted }: KeyDetailProps) {
  const { t } = useI18n();
  const keyTypeLabels = getKeyTypeLabels(t);

  const [revealed, setRevealed] = useState<KeyWithSecret | null>(null);
  const [loading, setLoading] = useState(false);
  const [errorCode, setErrorCode] = useState<unknown>(null);
  const [copyLabel, setCopyLabel] = useState(t.copyBtn);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  // Reset trạng thái hiện/ẩn khi chuyển sang xem key khác
  useEffect(() => {
    setRevealed(null);
    setErrorCode(null);
    setConfirmingDelete(false);
  }, [summary.id]);

  // Đồng bộ label nút Copy khi đổi ngôn ngữ giữa chừng
  useEffect(() => {
    setCopyLabel(t.copyBtn);
  }, [t.copyBtn]);

  async function handleReveal() {
    if (revealed) {
      setRevealed(null);
      return;
    }
    setLoading(true);
    setErrorCode(null);
    try {
      const data = await getKeySecret(summary.id);
      setRevealed(data);
    } catch (err) {
      setErrorCode(err);
    } finally {
      setLoading(false);
    }
  }

  async function handleCopy() {
    try {
      const data = revealed ?? (await getKeySecret(summary.id));
      await navigator.clipboard.writeText(data.secret_value);
      setCopyLabel(t.copiedBtn);

      setTimeout(async () => {
        try {
          // Chỉ xóa clipboard nếu nó vẫn đang chứa đúng giá trị này
          const current = await navigator.clipboard.readText();
          if (current === data.secret_value) {
            await navigator.clipboard.writeText('');
          }
        } catch {
          // Trình duyệt/OS có thể chặn đọc clipboard — bỏ qua an toàn
        }
        setCopyLabel(t.copyBtn);
      }, CLIPBOARD_CLEAR_MS);
    } catch (err) {
      setErrorCode(err);
    }
  }

  async function handleDelete() {
    if (!confirmingDelete) {
      setConfirmingDelete(true);
      return;
    }
    try {
      await deleteKey(summary.id);
      onDeleted();
    } catch (err) {
      setErrorCode(err);
    }
  }

  const error = errorCode ? translateError(errorCode, t) : null;

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 20 }}>
        <div>
          <span className="type-badge">{keyTypeLabels[summary.key_type]}</span>
          <h2 style={{ fontFamily: 'var(--font-display)', fontSize: 20, margin: '10px 0 0 0' }}>
            {summary.name}
          </h2>
        </div>

        <div style={{ display: 'flex', gap: 8 }}>
          {!confirmingDelete ? (
            <button className="btn btn-danger" onClick={handleDelete}>
              {t.deleteBtn}
            </button>
          ) : (
            <>
              <button className="btn btn-secondary" onClick={() => setConfirmingDelete(false)}>
                {t.cancelDeleteBtn}
              </button>
              <button className="btn btn-danger" onClick={handleDelete} style={{ borderColor: 'var(--danger)' }}>
                {t.confirmDeleteBtn}
              </button>
            </>
          )}
        </div>
      </div>

      {summary.tags.length > 0 && (
        <div style={{ display: 'flex', gap: 6, marginBottom: 20, flexWrap: 'wrap' }}>
          {summary.tags.map((tag) => (
            <span key={tag} className="tag-pill">
              {tag}
            </span>
          ))}
        </div>
      )}

      <div className="field" style={{ marginBottom: 16 }}>
        <label>{t.keyContentLabel}</label>
        <div className={`secret-box${revealed ? '' : ' masked'}`}>
          {revealed ? revealed.secret_value : '••••••••••••••••••••••••••••••••'}
        </div>
      </div>

      <div style={{ display: 'flex', gap: 10, marginBottom: 20 }}>
        <button className="btn btn-secondary" onClick={handleReveal} disabled={loading}>
          {loading ? t.decryptingBtn : revealed ? t.hideBtn : t.showBtn}
        </button>
        <button className="btn btn-secondary" onClick={handleCopy}>
          {copyLabel}
        </button>
      </div>

      {summary.notes && (
        <div className="field">
          <label>{t.notesLabel}</label>
          <p style={{ fontSize: 13, color: 'var(--text-dim)', margin: 0 }}>{summary.notes}</p>
        </div>
      )}

      {error && <p className="error-text" style={{ marginTop: 12 }}>{error}</p>}

      <p className="hint-text" style={{ marginTop: 24 }}>
        {t.clipboardHint(CLIPBOARD_CLEAR_MS / 1000)}
      </p>
    </div>
  );
}
