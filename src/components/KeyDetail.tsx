// src/components/KeyDetail.tsx

import { useEffect, useState } from 'react';
import { deleteKey, getKeySecret } from '../api/vault';
import { translateError, getKeyTypeLabels } from '../i18n/translations';
import { useI18n } from '../i18n/LanguageContext';
import type { KeySummary, KeyWithSecret } from '../types';
import { KeyPasswordModal, type KeyPasswordMode } from './KeyPasswordModal';

interface KeyDetailProps {
  summary: KeySummary;
  onDeleted: () => void;
  /** Gọi sau khi thêm/gỡ/đổi mật khẩu riêng thành công -> parent refresh() để has_extra_password cập nhật */
  onUpdated: () => void;
}

// Xóa clipboard sau khoảng thời gian này để tránh lộ qua clipboard manager khác.
const CLIPBOARD_CLEAR_MS = 20_000;

export function KeyDetail({ summary, onDeleted, onUpdated }: KeyDetailProps) {
  const { t } = useI18n();
  const keyTypeLabels = getKeyTypeLabels(t);

  const [revealed, setRevealed] = useState<KeyWithSecret | null>(null);
  const [loading, setLoading] = useState(false);
  const [errorCode, setErrorCode] = useState<unknown>(null);
  const [copyLabel, setCopyLabel] = useState(t.copyBtn);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  // Modal mật khẩu riêng: 'unlock' khi xem key bị khóa, hoặc add/remove/change
  // khi bấm nút quản lý bên dưới. pendingCopy = true nếu mở modal từ nút Copy
  // (để tự động copy ngay sau khi mở khóa thành công thay vì bắt bấm 2 lần).
  const [passwordModal, setPasswordModal] = useState<KeyPasswordMode | null>(null);
  const [pendingCopy, setPendingCopy] = useState(false);

  // Reset toàn bộ trạng thái khi chuyển sang xem key khác
  useEffect(() => {
    setRevealed(null);
    setErrorCode(null);
    setConfirmingDelete(false);
    setPasswordModal(null);
    setPendingCopy(false);
  }, [summary.id]);

  // Đồng bộ label nút Copy khi đổi ngôn ngữ giữa chừng
  useEffect(() => {
    setCopyLabel(t.copyBtn);
  }, [t.copyBtn]);

  async function copyToClipboard(value: string) {
    await navigator.clipboard.writeText(value);
    setCopyLabel(t.copiedBtn);

    setTimeout(async () => {
      // KHÔNG dùng navigator.clipboard.readText() để kiểm tra trước khi
      // xóa — API này thường bị webview chặn âm thầm (đặc biệt khi cửa
      // sổ mất focus), khiến việc xóa không bao giờ chạy dù trông như
      // đã xong. Luôn xóa vô điều kiện, giống cách Bitwarden/1Password
      // làm — đánh đổi nhỏ (có thể xóa nhầm thứ khác người dùng vừa copy
      // trong 20s đó) để đổi lấy đảm bảo secret luôn được dọn khỏi
      // clipboard, không phụ thuộc vào 1 API không ổn định.
      try {
        await navigator.clipboard.writeText('');
      } catch {
        // Trường hợp hiếm: writeText cũng bị chặn (VD cửa sổ mất focus
        // hoàn toàn) — không có gì thêm để làm, bỏ qua an toàn.
      }
      setCopyLabel(t.copyBtn);
    }, CLIPBOARD_CLEAR_MS);
  }

  async function handleReveal() {
    if (revealed) {
      setRevealed(null); // "Hide" = quên luôn, lần "Show" sau phải giải mã lại
      return;
    }

    if (summary.has_extra_password) {
      setPendingCopy(false);
      setPasswordModal('unlock');
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
    if (revealed) {
      await copyToClipboard(revealed.secret_value);
      return;
    }

    if (summary.has_extra_password) {
      setPendingCopy(true);
      setPasswordModal('unlock');
      return;
    }

    try {
      const data = await getKeySecret(summary.id);
      await copyToClipboard(data.secret_value);
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

  function handlePasswordModalSuccess(secret?: KeyWithSecret) {
    const mode = passwordModal;
    setPasswordModal(null);

    if (mode === 'unlock' && secret) {
      setRevealed(secret);
      if (pendingCopy) {
        copyToClipboard(secret.secret_value);
      }
      setPendingCopy(false);
      return;
    }

    // add / remove / change: nội dung không đổi nhưng has_extra_password
    // và lớp mã hóa đã đổi -> quên bản đã revealed (nếu có) cho an toàn,
    // và báo parent refresh lại summary.
    setRevealed(null);
    onUpdated();
  }

  const error = errorCode ? translateError(errorCode, t) : null;

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 20 }}>
        <div>
          <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
            <span className="type-badge">{keyTypeLabels[summary.key_type]}</span>
            {summary.has_extra_password && (
              <span className="type-badge protected-badge" title={t.protectedBadge}>
                🔒 {t.protectedBadge}
              </span>
            )}
          </div>
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
        <div className="field" style={{ marginBottom: 20 }}>
          <label>{t.notesLabel}</label>
          <p style={{ fontSize: 13, color: 'var(--text-dim)', margin: 0 }}>{summary.notes}</p>
        </div>
      )}

      {/* ---------- Quản lý mật khẩu riêng ---------- */}
      <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
        {!summary.has_extra_password ? (
          <button className="btn btn-secondary" onClick={() => setPasswordModal('add')}>
            {t.addKeyPasswordBtn}
          </button>
        ) : (
          <>
            <button className="btn btn-secondary" onClick={() => setPasswordModal('change')}>
              {t.changeKeyPasswordBtn}
            </button>
            <button className="btn btn-secondary" onClick={() => setPasswordModal('remove')}>
              {t.removeKeyPasswordBtn}
            </button>
          </>
        )}
      </div>

      {error && <p className="error-text" style={{ marginTop: 12 }}>{error}</p>}

      <p className="hint-text" style={{ marginTop: 24 }}>
        {t.clipboardHint(CLIPBOARD_CLEAR_MS / 1000)}
      </p>

      {passwordModal && (
        <KeyPasswordModal
          mode={passwordModal}
          keyId={summary.id}
          onClose={() => {
            setPasswordModal(null);
            setPendingCopy(false);
          }}
          onSuccess={handlePasswordModalSuccess}
        />
      )}
    </div>
  );
}