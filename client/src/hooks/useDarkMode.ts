import { useEffect } from 'react';
import { useTheme } from '@/theme';

export function useDarkMode() {
  const { theme, setTheme, resolvedTheme } = useTheme();

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(resolvedTheme);
  }, [resolvedTheme]);

  const toggleDarkMode = () => {
    setTheme(resolvedTheme === 'dark' ? 'light' : 'dark');
  };

  return {
    isDark: resolvedTheme === 'dark',
    theme,
    setTheme,
    toggleDarkMode,
  };
}
