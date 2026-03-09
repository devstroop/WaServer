import { useParams, useNavigate, Link } from 'react-router-dom';
import { ArrowLeft, Power, RotateCw, Trash2, LogOut } from 'lucide-react';
import { MainLayout } from '@/layouts';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Skeleton } from '@/components/ui/Skeleton';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { StatusBadge } from '@/components/shared/StatusBadge';
import { QrCodeDisplay } from '@/features/instances';
import { useInstance, useDeleteInstance, useWarmupInstance, useResetInstance } from '@/features/instances/hooks/useInstances';
import { useWhatsAppStatus, useWhatsAppLogout } from '@/features/instances/hooks/useWhatsApp';
import { ROUTES, INSTANCE_STATUS } from '@/lib/constants';
import { formatDate, formatPhoneNumber } from '@/lib/utils';
import { useState } from 'react';

export function InstanceDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [logoutDialogOpen, setLogoutDialogOpen] = useState(false);

  const { data: instance, isLoading } = useInstance(id ?? '');
  const { data: status } = useWhatsAppStatus(id ?? '', !!instance?.authorized);
  const deleteInstance = useDeleteInstance();
  const warmupInstance = useWarmupInstance();
  const resetInstance = useResetInstance();
  const logout = useWhatsAppLogout();

  const isActive = instance?.status === INSTANCE_STATUS.ACTIVE;
  const isActionLoading = deleteInstance.isPending || warmupInstance.isPending || resetInstance.isPending || logout.isPending;

  const handleDelete = async () => {
    if (!id) return;
    await deleteInstance.mutateAsync(id);
    navigate(ROUTES.INSTANCES);
  };

  if (isLoading) {
    return (
      <MainLayout>
        <div className="space-y-6">
          <Skeleton className="h-8 w-48" />
          <div className="grid gap-6 lg:grid-cols-2">
            <Skeleton className="h-64" />
            <Skeleton className="h-64" />
          </div>
        </div>
      </MainLayout>
    );
  }

  if (!instance) {
    return (
      <MainLayout>
        <div className="text-center py-12">
          <h1 className="text-2xl font-bold mb-2">Instance not found</h1>
          <p className="text-muted-foreground mb-4">The instance you're looking for doesn't exist.</p>
          <Button asChild><Link to={ROUTES.INSTANCES}>Back to Instances</Link></Button>
        </div>
      </MainLayout>
    );
  }

  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" asChild><Link to={ROUTES.INSTANCES}><ArrowLeft className="h-5 w-5" /></Link></Button>
          <div className="flex-1">
            <div className="flex items-center gap-3">
              <h1 className="text-3xl font-bold">{instance.name}</h1>
              <StatusBadge status={instance.status} authorized={instance.authorized} />
            </div>
            <p className="text-muted-foreground">
              {instance.phone_number ? formatPhoneNumber(instance.phone_number) : 'Not connected'}
            </p>
          </div>
        </div>

        <Tabs defaultValue="connection" className="space-y-4">
          <TabsList>
            <TabsTrigger value="connection">Connection</TabsTrigger>
            <TabsTrigger value="details">Details</TabsTrigger>
            <TabsTrigger value="actions">Actions</TabsTrigger>
          </TabsList>

          <TabsContent value="connection" className="space-y-4">
            <div className="grid gap-6 lg:grid-cols-2">
              <QrCodeDisplay instanceId={instance.id} />
              {status && (
                <Card>
                  <CardHeader>
                    <CardTitle>Device Info</CardTitle>
                    <CardDescription>Connected device information</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Phone Number</span>
                      <span className="font-medium">{status.phone_number ?? 'N/A'}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Battery</span>
                      <span className="font-medium">{status.battery_level !== null ? `${status.battery_level}%` : 'N/A'}{status.is_plugged && ' ⚡'}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Status</span>
                      <span className="font-medium">{status.connected ? 'Connected' : 'Disconnected'}</span>
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>
          </TabsContent>

          <TabsContent value="details">
            <Card>
              <CardHeader>
                <CardTitle>Instance Details</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">ID</span>
                  <code className="text-sm bg-muted px-2 py-1 rounded">{instance.id}</code>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Created</span>
                  <span>{formatDate(instance.created_at, 'full')}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Last Updated</span>
                  <span>{formatDate(instance.updated_at, 'full')}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Status</span>
                  <StatusBadge status={instance.status} authorized={instance.authorized} />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="actions">
            <Card>
              <CardHeader>
                <CardTitle>Instance Actions</CardTitle>
                <CardDescription>Manage your instance state</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex flex-wrap gap-3">
                  {!isActive && (
                    <Button onClick={() => warmupInstance.mutate(instance.id)} loading={warmupInstance.isPending} disabled={isActionLoading}>
                      <Power className="mr-2 h-4 w-4" />Warmup
                    </Button>
                  )}
                  {isActive && (
                    <Button variant="secondary" onClick={() => resetInstance.mutate(instance.id)} loading={resetInstance.isPending} disabled={isActionLoading}>
                      <RotateCw className="mr-2 h-4 w-4" />Reset
                    </Button>
                  )}
                  {instance.authorized && (
                    <Button variant="outline" onClick={() => setLogoutDialogOpen(true)} disabled={isActionLoading}>
                      <LogOut className="mr-2 h-4 w-4" />Disconnect WhatsApp
                    </Button>
                  )}
                </div>
                <div className="border-t pt-4">
                  <h4 className="font-medium text-destructive mb-2">Danger Zone</h4>
                  <Button variant="destructive" onClick={() => setDeleteDialogOpen(true)} disabled={isActionLoading}>
                    <Trash2 className="mr-2 h-4 w-4" />Delete Instance
                  </Button>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>

      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete Instance"
        description={`Are you sure you want to delete "${instance.name}"? This action cannot be undone.`}
        confirmText="Delete"
        variant="destructive"
        onConfirm={handleDelete}
        loading={deleteInstance.isPending}
      />
      <ConfirmDialog
        open={logoutDialogOpen}
        onOpenChange={setLogoutDialogOpen}
        title="Disconnect WhatsApp"
        description="This will log out the connected WhatsApp account. You'll need to scan the QR code again to reconnect."
        confirmText="Disconnect"
        variant="destructive"
        onConfirm={() => logout.mutateAsync(instance.id)}
        loading={logout.isPending}
      />
    </MainLayout>
  );
}
