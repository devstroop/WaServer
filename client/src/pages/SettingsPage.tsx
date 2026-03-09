import { useQuery } from '@tanstack/react-query';
import { Settings as SettingsIcon, Key, Activity, CheckCircle, XCircle, AlertTriangle } from 'lucide-react';
import { MainLayout } from '@/layouts';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Skeleton } from '@/components/ui/Skeleton';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/Alert';
import { ThemeToggle } from '@/theme';
import { useIsAuthenticated } from '@/features/auth/hooks/useAuth';
import { useAuthStore } from '@/stores/auth.store';
import { healthService } from '@/services/health.service';
import { queryKeys } from '@/lib/query-client';

function HealthStatusCard() {
  const { data: health, isLoading, error } = useQuery({
    queryKey: queryKeys.health,
    queryFn: healthService.check,
    refetchInterval: 30000,
  });

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>System Health</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {Array.from({ length: 3 }).map((_, i: number) => (
            <div key={i} className="flex items-center justify-between">
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-4 w-16" />
            </div>
          ))}
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <XCircle className="h-4 w-4" />
        <AlertTitle>Health Check Failed</AlertTitle>
        <AlertDescription>Unable to reach the API server.</AlertDescription>
      </Alert>
    );
  }

  const StatusIcon = health?.status === 'healthy' ? CheckCircle : health?.status === 'degraded' ? AlertTriangle : XCircle;
  const statusColor = health?.status === 'healthy' ? 'text-green-500' : health?.status === 'degraded' ? 'text-yellow-500' : 'text-red-500';

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Activity className="h-5 w-5" />
          System Health
        </CardTitle>
        <CardDescription>Current status of the WAS backend services</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <span>Overall Status</span>
          <div className={`flex items-center gap-1 ${statusColor}`}>
            <StatusIcon className="h-4 w-4" />
            <span className="capitalize font-medium">{health?.status}</span>
          </div>
        </div>
        <div className="flex items-center justify-between">
          <span>Version</span>
          <code className="text-sm bg-muted px-2 py-1 rounded">{health?.version}</code>
        </div>
        <div className="flex items-center justify-between">
          <span>Uptime</span>
          <span>{Math.floor((health?.uptime_seconds ?? 0) / 3600)}h {Math.floor(((health?.uptime_seconds ?? 0) % 3600) / 60)}m</span>
        </div>
        {health?.services && health.services.length > 0 && (
          <div className="pt-2 border-t space-y-2">
            <span className="text-sm font-medium">Services</span>
            {health.services.map((service) => (
              <div key={service.name} className="flex items-center justify-between text-sm">
                <span className="capitalize">{service.name}</span>
                <span className={service.status === 'healthy' ? 'text-green-500' : 'text-red-500'}>
                  {service.status === 'healthy' ? 'Healthy' : 'Unhealthy'}
                </span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function SettingsPage() {
  const isAuthenticated = useIsAuthenticated();
  const apiKey = useAuthStore((state) => state.apiKey);

  // Mask API key for display
  const maskedApiKey = apiKey ? `${apiKey.slice(0, 8)}${'*'.repeat(Math.max(0, apiKey.length - 12))}${apiKey.slice(-4)}` : 'Not set';

  return (
    <MainLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold">Settings</h1>
          <p className="text-muted-foreground">Manage your application preferences</p>
        </div>

        <Tabs defaultValue="api" className="space-y-4">
          <TabsList>
            <TabsTrigger value="api"><Key className="mr-2 h-4 w-4" />API</TabsTrigger>
            <TabsTrigger value="appearance"><SettingsIcon className="mr-2 h-4 w-4" />Appearance</TabsTrigger>
            <TabsTrigger value="system"><Activity className="mr-2 h-4 w-4" />System</TabsTrigger>
          </TabsList>

          <TabsContent value="api">
            <Card>
              <CardHeader>
                <CardTitle>API Configuration</CardTitle>
                <CardDescription>Your API key is used to authenticate requests</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <span>Status</span>
                  <span className={isAuthenticated ? 'text-green-500' : 'text-yellow-500'}>
                    {isAuthenticated ? 'Authenticated' : 'Not authenticated'}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span>API Key</span>
                  <code className="text-sm bg-muted px-2 py-1 rounded">{maskedApiKey}</code>
                </div>
                <p className="text-sm text-muted-foreground">
                  Your API key is stored securely in your browser's local storage.
                </p>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="appearance">
            <Card>
              <CardHeader>
                <CardTitle>Appearance</CardTitle>
                <CardDescription>Customize how WAS looks</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Theme</p>
                    <p className="text-sm text-muted-foreground">Select your preferred color scheme</p>
                  </div>
                  <ThemeToggle />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="system">
            <HealthStatusCard />
          </TabsContent>
        </Tabs>
      </div>
    </MainLayout>
  );
}
