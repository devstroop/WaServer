import { Server } from 'lucide-react';
import { MainLayout } from '@/layouts';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/shared/EmptyState';
import { CreateInstanceModal, InstanceCard } from '@/features/instances';
import { useInstances } from '@/features/instances/hooks/useInstances';
import type { Instance } from '@/services/instance.service';

export function InstancesPage() {
  const { data: instances, isLoading, error } = useInstances();

  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold">Instances</h1>
            <p className="text-muted-foreground">Manage your WhatsApp instances</p>
          </div>
          <CreateInstanceModal />
        </div>

        {isLoading ? (
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 6 }).map((_, i: number) => (
              <div key={i} className="border rounded-lg p-4 space-y-3">
                <div className="flex justify-between">
                  <Skeleton className="h-5 w-32" />
                  <Skeleton className="h-5 w-5" />
                </div>
                <Skeleton className="h-4 w-24" />
                <div className="flex justify-between items-center">
                  <Skeleton className="h-3 w-20" />
                  <Skeleton className="h-6 w-16 rounded-full" />
                </div>
              </div>
            ))}
          </div>
        ) : error ? (
          <EmptyState icon={Server} title="Failed to load instances" description="An error occurred while loading your instances. Please try again." action={<CreateInstanceModal />} />
        ) : instances?.instances?.length === 0 ? (
          <EmptyState
            icon={Server}
            title="No instances yet"
            description="Create your first WhatsApp instance to start automating messages."
            action={<CreateInstanceModal />}
          />
        ) : (
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {instances?.instances?.map((instance: Instance) => (
              <InstanceCard key={instance.id} instance={instance} />
            ))}
          </div>
        )}
      </div>
    </MainLayout>
  );
}
