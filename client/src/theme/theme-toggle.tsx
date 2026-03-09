import { useContext } from 'react';
import { Moon, Sun, Monitor } from 'lucide-react';
import { ThemeContext } from './theme-context';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/DropdownMenu';
import { Button } from '@/components/ui/Button';

export function ThemeToggle() {
  const context = useContext(ThemeContext);
  if (!context) throw new Error('ThemeToggle must be used within ThemeProvider');
  const { setTheme, resolvedTheme } = context;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon">
          {resolvedTheme === 'dark' ? <Moon className="h-5 w-5" /> : <Sun className="h-5 w-5" />}
          <span className="sr-only">Toggle theme</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => setTheme('light')}><Sun className="mr-2 h-4 w-4" />Light</DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('dark')}><Moon className="mr-2 h-4 w-4" />Dark</DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('system')}><Monitor className="mr-2 h-4 w-4" />System</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
