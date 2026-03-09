import { createContext } from 'react';

export type Theme = 'dark' | 'light' | 'system';

export interface ThemeContextValue {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  resolvedTheme: 'dark' | 'light';
}

export const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);
