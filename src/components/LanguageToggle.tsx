// src/components/LanguageToggle.tsx

import { LANGUAGE_NAMES, type Language } from '../i18n/translations';

interface LanguageToggleProps {
  language: Language;
  onChange: (lang: Language) => void;
  /** true = hiện nổi góc màn hình (Setup/Unlock). false = dùng inline trong header (Vault). */
  floating?: boolean;
}

const ORDER: Language[] = ['en', 'vi'];

export function LanguageToggle({ language, onChange, floating = true }: LanguageToggleProps) {
  const nextLang = ORDER[(ORDER.indexOf(language) + 1) % ORDER.length];

  return (
    <button
      className="language-toggle"
      style={floating ? undefined : { position: 'static' }}
      onClick={() => onChange(nextLang)}
      title={`Switch to ${LANGUAGE_NAMES[nextLang]}`}
      aria-label="Change language"
    >
      {language.toUpperCase()}
    </button>
  );
}
