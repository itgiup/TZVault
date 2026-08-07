// src/components/Modal.tsx
//
// Wrapper dùng chung cho mọi modal trong app (AddKeyModal, SettingsModal,
// KeyPasswordModal...) — xử lý animation mở/đóng mượt ở 1 chỗ duy nhất
// thay vì lặp lại logic này trong từng modal.
//
// Cách hoạt động animation "đóng":
//   React thường unmount component ngay lập tức khi điều kiện render tắt,
//   nên không kịp chạy transition "biến mất". Modal này tự quản lý 1 state
//   nội bộ `closing` — khi cần đóng, bật `closing` trước (CSS transition
//   chạy), đợi đúng thời gian animation rồi mới thực sự gọi onClose (lúc
//   đó component cha mới unmount nó).

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';

const ANIMATION_MS = 180;

interface ModalCloseApi {
  /** Đóng modal (theo nghĩa "hủy") - chạy animation rồi gọi onClose gốc. */
  requestClose: () => void;
  /** Đóng modal rồi chạy 1 callback khác thay vì onClose gốc - dùng khi
   * thao tác thành công (VD thêm key xong) cần làm thêm việc như refresh
   * danh sách, không chỉ đơn thuần đóng modal. */
  closeThen: (afterClose: () => void) => void;
}

const ModalCloseContext = createContext<ModalCloseApi | null>(null);

/** Gọi bên trong children của <Modal> để lấy hàm đóng có animation. */
export function useModalClose(): ModalCloseApi {
  const ctx = useContext(ModalCloseContext);
  if (!ctx) {
    throw new Error('useModalClose phải được gọi bên trong <Modal>');
  }
  return ctx;
}

interface ModalProps {
  onClose: () => void;
  children: ReactNode;
}

export function Modal({ onClose, children }: ModalProps) {
  // visible: bắt đầu false -> true ngay sau khi mount (trigger transition
  // "xuất hiện"). closing: true khi đang trong quá trình đóng.
  const [visible, setVisible] = useState(false);
  const [closing, setClosing] = useState(false);

  useEffect(() => {
    const raf = requestAnimationFrame(() => setVisible(true));
    return () => cancelAnimationFrame(raf);
  }, []);

  function closeThen(afterClose: () => void) {
    if (closing) return; // tránh double-trigger nếu bấm nhiều lần
    setClosing(true);
    setVisible(false);
    setTimeout(afterClose, ANIMATION_MS);
  }

  function requestClose() {
    closeThen(onClose);
  }

  // Đóng bằng phím Esc
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') requestClose();
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [closing]);

  const stateClass = visible ? ' visible' : '';

  return (
    <ModalCloseContext.Provider value={{ requestClose, closeThen }}>
      <div className={`modal-overlay${stateClass}`} onClick={requestClose}>
        <div className={`modal-card${stateClass}`} onClick={(e) => e.stopPropagation()}>
          {children}
        </div>
      </div>
    </ModalCloseContext.Provider>
  );
}
