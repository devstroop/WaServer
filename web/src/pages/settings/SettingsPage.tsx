import { Header } from '@/components/layout';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';
import { useTheme } from '@/store/ThemeContext';
import { useAuthStore } from '@/store/authStore';
import { Sun, Moon, Monitor } from 'lucide-react';

export function SettingsPage() {
  const { theme, setTheme } = useTheme();
  const { user } = useAuthStore();

  return (
    <>
      <Header 
        title="Settings" 
        description="Manage your preferences"
      />

      <div className="p-6 space-y-6 max-w-2xl">
        {/* Theme Settings */}
        <Card>
          <CardHeader>
            <CardTitle>Appearance</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                Choose your preferred theme
              </p>
              <div className="grid grid-cols-3 gap-3">
                <Button
                  variant={theme === 'light' ? 'primary' : 'outline'}
                  onClick={() => setTheme('light')}
                  className="flex flex-col items-center gap-2 h-auto py-4"
                >
                  <Sun className="h-5 w-5" />
                  Light
                </Button>
                <Button
                  variant={theme === 'dark' ? 'primary' : 'outline'}
                  onClick={() => setTheme('dark')}
                  className="flex flex-col items-center gap-2 h-auto py-4"
                >
                  <Moon className="h-5 w-5" />
                  Dark
                </Button>
                <Button
                  variant={theme === 'system' ? 'primary' : 'outline'}
                  onClick={() => setTheme('system')}
                  className="flex flex-col items-center gap-2 h-auto py-4"
                >
                  <Monitor className="h-5 w-5" />
                  System
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Account Info */}
        <Card>
          <CardHeader>
            <CardTitle>Account</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div>
                <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                  Username
                </p>
                <p className="font-medium text-text-light dark:text-text-dark">
                  {user?.username || '—'}
                </p>
              </div>
              <div>
                <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                  Role
                </p>
                <p className="font-medium text-text-light dark:text-text-dark capitalize">
                  {user?.role || '—'}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* About */}
        <Card>
          <CardHeader>
            <CardTitle>About</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2 text-sm text-text-muted-light dark:text-text-muted-dark">
              <p><strong>WAS</strong> - WhatsApp Automation Server</p>
              <p>A powerful WhatsApp automation solution with multi-instance support.</p>
            </div>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
