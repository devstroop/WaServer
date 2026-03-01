import { Sun, Moon, Monitor } from 'lucide-react';
import { useTheme } from '@/store/ThemeContext';
import { Button } from '@/components/ui';

interface HeaderProps {
  title: string;
  description?: string;
  actions?: React.ReactNode;
}

export function Header({ title, description, actions }: HeaderProps) {
  const { theme, setTheme } = useTheme();

  const cycleTheme = () => {
    const themes: ('light' | 'dark' | 'system')[] = ['light', 'dark', 'system'];
    const currentIndex = themes.indexOf(theme);
    const nextIndex = (currentIndex + 1) % themes.length;
    setTheme(themes[nextIndex]);
  };

  const ThemeIcon = theme === 'light' ? Sun : theme === 'dark' ? Moon : Monitor;

  return (
    <header className="sticky top-0 z-30 bg-bg-light/80 dark:bg-bg-dark/80 backdrop-blur-sm border-b border-border-light dark:border-border-dark">
      <div className="px-6 py-4 flex items-center justify-between">
        <div className="lg:ml-0 ml-12">
          <h1 className="text-2xl font-bold text-text-light dark:text-text-dark">
            {title}
          </h1>
          {description && (
            <p className="text-text-muted-light dark:text-text-muted-dark mt-1">
              {description}
            </p>
          )}
        </div>
        <div className="flex items-center gap-3">
          {actions}
          <Button
            variant="ghost"
            size="icon"
            onClick={cycleTheme}
            title={`Theme: ${theme}`}
          >
            <ThemeIcon className="h-5 w-5" />
          </Button>
        </div>
      </div>
    </header>
  );
}
