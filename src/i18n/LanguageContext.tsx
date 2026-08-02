// src/i18n/LanguageContext.tsx

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { translations, type Language, type Translations } from './translations';

const STORAGE_KEY = 'vault-language';
const DEFAULT_LANGUAGE: Language = 'en';

interface LanguageContextValue {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: Translations;
}

const LanguageContext = createContext<LanguageContextValue | null>(null);

function getInitialLanguage(): Language {
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored === 'en' || stored === 'vi') return stored;
  // Mặc định tiếng Anh theo yêu cầu, kể cả khi OS đang ở locale khác —
  // người dùng có thể tự đổi qua LanguageToggle.
  return DEFAULT_LANGUAGE;
}

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<Language>(getInitialLanguage);

  useEffect(() => {
    window.localStorage.setItem(STORAGE_KEY, language);
    document.documentElement.setAttribute('lang', language);
  }, [language]);

  function setLanguage(lang: Language) {
    setLanguageState(lang);
  }

  const value: LanguageContextValue = {
    language,
    setLanguage,
    t: translations[language],
  };

  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

export function useI18n(): LanguageContextValue {
  const ctx = useContext(LanguageContext);
  if (!ctx) {
    throw new Error('useI18n phải được gọi bên trong <LanguageProvider>');
  }
  return ctx;
}
