import { useState } from 'react';
import { User, Bell, Shield, Palette, Save } from 'lucide-react';
import { PageHeader } from '@/components/layout/PageHeader';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { InputField } from '@/components/forms/InputField';
import { SelectField } from '@/components/forms/SelectField';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Separator } from '@/components/ui/Separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { useTheme } from '@/theme';

export function SettingsPage() {
  const { theme, setTheme } = useTheme();
  const [profileData, setProfileData] = useState({
    name: 'Admin User',
    email: 'admin@example.com',
    organization: 'My Company',
  });

  const [notifications, setNotifications] = useState({
    emailAlerts: true,
    slackAlerts: false,
    dailyDigest: true,
    errorAlerts: true,
  });

  return (
    <ContentContainer>
      <PageHeader
        title="Settings"
        description="Manage your account and system preferences"
      />

      <Tabs defaultValue="profile" className="space-y-6">
        <TabsList>
          <TabsTrigger value="profile" className="gap-2">
            <User className="h-4 w-4" />
            Profile
          </TabsTrigger>
          <TabsTrigger value="notifications" className="gap-2">
            <Bell className="h-4 w-4" />
            Notifications
          </TabsTrigger>
          <TabsTrigger value="appearance" className="gap-2">
            <Palette className="h-4 w-4" />
            Appearance
          </TabsTrigger>
          <TabsTrigger value="security" className="gap-2">
            <Shield className="h-4 w-4" />
            Security
          </TabsTrigger>
        </TabsList>

        <TabsContent value="profile">
          <Card>
            <CardHeader>
              <CardTitle>Profile Information</CardTitle>
              <CardDescription>
                Update your personal information and account details
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-6 md:grid-cols-2">
                <InputField
                  label="Name"
                  value={profileData.name}
                  onChange={(e) =>
                    setProfileData({ ...profileData, name: e.target.value })
                  }
                />
                <InputField
                  label="Email"
                  type="email"
                  value={profileData.email}
                  onChange={(e) =>
                    setProfileData({ ...profileData, email: e.target.value })
                  }
                />
              </div>
              <InputField
                label="Organization"
                value={profileData.organization}
                onChange={(e) =>
                  setProfileData({ ...profileData, organization: e.target.value })
                }
              />
              <Button>
                <Save className="h-4 w-4 mr-2" />
                Save Changes
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="notifications">
          <Card>
            <CardHeader>
              <CardTitle>Notification Preferences</CardTitle>
              <CardDescription>
                Configure how you want to receive notifications
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Email Alerts</p>
                    <p className="text-sm text-muted-foreground">
                      Receive important alerts via email
                    </p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      className="sr-only peer"
                      checked={notifications.emailAlerts}
                      onChange={(e) =>
                        setNotifications({ ...notifications, emailAlerts: e.target.checked })
                      }
                    />
                    <div className="w-11 h-6 bg-secondary rounded-full peer peer-checked:bg-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                  </label>
                </div>

                <Separator />

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Slack Notifications</p>
                    <p className="text-sm text-muted-foreground">
                      Send notifications to Slack channel
                    </p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      className="sr-only peer"
                      checked={notifications.slackAlerts}
                      onChange={(e) =>
                        setNotifications({ ...notifications, slackAlerts: e.target.checked })
                      }
                    />
                    <div className="w-11 h-6 bg-secondary rounded-full peer peer-checked:bg-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                  </label>
                </div>

                <Separator />

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Daily Digest</p>
                    <p className="text-sm text-muted-foreground">
                      Receive a daily summary of activity
                    </p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      className="sr-only peer"
                      checked={notifications.dailyDigest}
                      onChange={(e) =>
                        setNotifications({ ...notifications, dailyDigest: e.target.checked })
                      }
                    />
                    <div className="w-11 h-6 bg-secondary rounded-full peer peer-checked:bg-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                  </label>
                </div>

                <Separator />

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Error Alerts</p>
                    <p className="text-sm text-muted-foreground">
                      Immediate alerts for system errors
                    </p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      className="sr-only peer"
                      checked={notifications.errorAlerts}
                      onChange={(e) =>
                        setNotifications({ ...notifications, errorAlerts: e.target.checked })
                      }
                    />
                    <div className="w-11 h-6 bg-secondary rounded-full peer peer-checked:bg-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                  </label>
                </div>
              </div>

              <Button>
                <Save className="h-4 w-4 mr-2" />
                Save Preferences
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="appearance">
          <Card>
            <CardHeader>
              <CardTitle>Appearance</CardTitle>
              <CardDescription>
                Customize the look and feel of the dashboard
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <SelectField
                label="Theme"
                options={[
                  { value: 'light', label: 'Light' },
                  { value: 'dark', label: 'Dark' },
                  { value: 'system', label: 'System' },
                ]}
                value={theme}
                onValueChange={(value) => setTheme(value as 'light' | 'dark' | 'system')}
              />

              <div className="grid gap-4 md:grid-cols-3">
                <button
                  onClick={() => setTheme('light')}
                  className={`p-4 rounded-lg border-2 transition-colors ${
                    theme === 'light' ? 'border-primary' : 'border-border'
                  }`}
                >
                  <div className="h-20 bg-white rounded border mb-2"></div>
                  <p className="text-sm font-medium">Light</p>
                </button>
                <button
                  onClick={() => setTheme('dark')}
                  className={`p-4 rounded-lg border-2 transition-colors ${
                    theme === 'dark' ? 'border-primary' : 'border-border'
                  }`}
                >
                  <div className="h-20 bg-zinc-900 rounded border mb-2"></div>
                  <p className="text-sm font-medium">Dark</p>
                </button>
                <button
                  onClick={() => setTheme('system')}
                  className={`p-4 rounded-lg border-2 transition-colors ${
                    theme === 'system' ? 'border-primary' : 'border-border'
                  }`}
                >
                  <div className="h-20 bg-gradient-to-r from-white to-zinc-900 rounded border mb-2"></div>
                  <p className="text-sm font-medium">System</p>
                </button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="security">
          <Card>
            <CardHeader>
              <CardTitle>Security Settings</CardTitle>
              <CardDescription>
                Manage your account security and authentication
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-4">
                <div>
                  <h3 className="font-medium mb-2">Change Password</h3>
                  <div className="grid gap-4 max-w-sm">
                    <InputField
                      label="Current Password"
                      type="password"
                      placeholder="Enter current password"
                    />
                    <InputField
                      label="New Password"
                      type="password"
                      placeholder="Enter new password"
                    />
                    <InputField
                      label="Confirm New Password"
                      type="password"
                      placeholder="Confirm new password"
                    />
                    <Button className="w-fit">Update Password</Button>
                  </div>
                </div>

                <Separator />

                <div>
                  <h3 className="font-medium mb-2">Two-Factor Authentication</h3>
                  <p className="text-sm text-muted-foreground mb-4">
                    Add an extra layer of security to your account
                  </p>
                  <Button variant="outline">Enable 2FA</Button>
                </div>

                <Separator />

                <div>
                  <h3 className="font-medium mb-2">Active Sessions</h3>
                  <p className="text-sm text-muted-foreground mb-4">
                    Manage your active sessions across devices
                  </p>
                  <div className="p-4 bg-muted rounded-lg">
                    <div className="flex justify-between items-center">
                      <div>
                        <p className="font-medium">Current Session</p>
                        <p className="text-sm text-muted-foreground">
                          Linux • Chrome • Last active now
                        </p>
                      </div>
                      <span className="text-xs px-2 py-1 bg-success/10 text-success rounded">
                        Active
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </ContentContainer>
  );
}
