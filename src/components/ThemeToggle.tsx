// src/components/ThemeToggle.tsx

import type { Theme } from '../hooks/useTheme';

interface ThemeToggleProps {
  theme: Theme;
  onToggle: () => void;
  /** true = hiện nổi góc màn hình (Setup/Unlock). false = dùng inline trong header (Vault). */
  floating?: boolean;
}

export function ThemeToggle({ theme, onToggle, floating = true }: ThemeToggleProps) {
  return (
    <button
      className="theme-toggle"
      style={floating ? undefined : { position: 'static' }}
      onClick={onToggle}
      title={theme === 'dark' ? 'Chuyển sang giao diện sáng' : 'Chuyển sang giao diện tối'}
      aria-label="Đổi giao diện sáng/tối"
    >
      {theme === 'dark' ? (
        // icon mặt trời - bấm để chuyển sang light
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
        </svg>
      ) : (
        // icon mặt trăng - bấm để chuyển sang dark
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      )}
    </button>
  );
}
