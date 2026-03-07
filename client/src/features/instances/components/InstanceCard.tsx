import { useState } from 'react';
import { Link } from 'react-router-dom';
import { MoreVertical, Play, RotateCw, Trash2, ExternalLink } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/DropdownMenu';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { StatusBadge } from '@/components/shared/StatusBadge';
import type { Instance } from '@/services/instance.service';
import { useDeleteInstance, useWarmupInstance, useResetInstance } from '../hooks/useInstances';
import { formatRelativeTime, formatPhoneNumber } from '@/lib/utils';
import { INSTANCE_STATUS } from '@/lib/constants';

interface InstanceCardProps {
  instance: Instance;
}

export function InstanceCard({ instance }: InstanceCardProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const deleteInstance = useDeleteInstance();
  const warmupInstance = useWarmupInstance();
  const resetInstance = useResetInstance();

  const isActive = instance.status === INSTANCE_STATUS.ACTIVE;
  const isInactive = instance.status === INSTANCE_STATUS.INACTIVE;
  const isLoading = deleteInstance.isPending || warmupInstance.isPending || resetInstance.isPending;

  return (
    <>
      <Card className="hover:shadow-md transition-shadow">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-lg font-medium">{instance.name}</CardTitle>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" disabled={isLoading}><MoreVertical className="h-4 w-4" /></Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem asChild>
                <Link to={`/instances/${instance.id}`} className="flex items-center">
                  <ExternalLink className="mr-2 h-4 w-4" />View Details
                </Link>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              {isInactive && (
                <DropdownMenuItem onClick={() => warmupInstance.mutate(instance.id)}>
                  <Play className="mr-2 h-4 w-4" />Warmup
                </DropdownMenuItem>
              )}
              {isActive && (
                <DropdownMenuItem onClick={() => resetInstance.mutate(instance.id)}>
                  <RotateCw className="mr-2 h-4 w-4" />Reset
                </DropdownMenuItem>
              )}
              <DropdownMenuSeparator />
              <DropdownMenuItem className="text-destructive" onClick={() => setDeleteDialogOpen(true)}>
                <Trash2 className="mr-2 h-4 w-4" />Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <p className="text-sm text-muted-foreground">
                {instance.phone_number ? formatPhoneNumber(instance.phone_number) : 'Not connected'}
              </p>
              <p className="text-xs text-muted-foreground">Updated {formatRelativeTime(instance.updated_at)}</p>
            </div>
            <StatusBadge status={instance.status} authorized={instance.authorized} />
          </div>
        </CardContent>
      </Card>
      <ConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="Delete Instance"
        description={`Are you sure you want to delete "${instance.name}"? This action cannot be undone.`}
        confirmText="Delete"
        variant="destructive"
        onConfirm={() => deleteInstance.mutateAsync(instance.id)}
        loading={deleteInstance.isPending}
      />
    </>
  );
}
