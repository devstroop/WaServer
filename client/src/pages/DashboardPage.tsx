import { Link } from 'react-router-dom';
import { Server, MessageSquare, Activity, ArrowRight } from 'lucide-react';
import { MainLayout } from '@/layouts';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Skeleton } from '@/components/ui/Skeleton';
import { InstanceStats, CreateInstanceModal } from '@/features/instances';
import { useInstances } from '@/features/instances/hooks/useInstances';
import { ROUTES, INSTANCE_STATUS } from '@/lib/constants';
import { formatRelativeTime } from '@/lib/utils';
import type { Instance } from '@/services/instance.service';

export function DashboardPage() {
  const { data: instances, isLoading } = useInstances();

  const recentInstances = instances?.instances?.slice(0, 3) ?? [];
  const activeCount = instances?.instances?.filter((i: Instance) => i.status === INSTANCE_STATUS.ACTIVE).length ?? 0;

  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold">Welcome back!</h1>
            <p className="text-muted-foreground">Here's what's happening with your WhatsApp instances.</p>
          </div>
          <CreateInstanceModal />
        </div>

        <InstanceStats />

        <div className="grid gap-6 lg:grid-cols-2">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>Recent Instances</CardTitle>
                <CardDescription>Your recently updated instances</CardDescription>
              </div>
              <Button variant="outline" size="sm" asChild>
                <Link to={ROUTES.INSTANCES}>View All<ArrowRight className="ml-2 h-4 w-4" /></Link>
              </Button>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <div className="space-y-3">
                  {Array.from({ length: 3 }).map((_, i: number) => (
                    <div key={i} className="flex items-center gap-3">
                      <Skeleton className="h-10 w-10 rounded-full" />
                      <div className="flex-1 space-y-2">
                        <Skeleton className="h-4 w-24" />
                        <Skeleton className="h-3 w-32" />
                      </div>
                    </div>
                  ))}
                </div>
              ) : recentInstances.length > 0 ? (
                <div className="space-y-3">
                  {recentInstances.map((instance: Instance) => (
                    <Link key={instance.id} to={`/instances/${instance.id}`}
                      className="flex items-center gap-3 p-2 rounded-lg hover:bg-accent transition-colors">
                      <div className="h-10 w-10 rounded-full bg-primary/10 flex items-center justify-center">
                        <Server className="h-5 w-5 text-primary" />
                      </div>
                      <div className="flex-1">
                        <p className="font-medium">{instance.name}</p>
                        <p className="text-sm text-muted-foreground">Updated {formatRelativeTime(instance.updated_at)}</p>
                      </div>
                      <div className={`h-2 w-2 rounded-full ${instance.status === INSTANCE_STATUS.ACTIVE ? 'bg-green-500' : 'bg-gray-400'}`} />
                    </Link>
                  ))}
                </div>
              ) : (
                <div className="text-center py-6">
                  <Server className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                  <p className="text-muted-foreground mb-3">No instances yet</p>
                  <CreateInstanceModal />
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
              <CardDescription>Common tasks and shortcuts</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-3">
              <Button variant="outline" className="justify-start h-auto py-3" asChild>
                <Link to={ROUTES.INSTANCES}>
                  <Server className="mr-3 h-5 w-5" />
                  <div className="text-left">
                    <p className="font-medium">Manage Instances</p>
                    <p className="text-sm text-muted-foreground">{activeCount} active instance{activeCount !== 1 ? 's' : ''}</p>
                  </div>
                </Link>
              </Button>
              <Button variant="outline" className="justify-start h-auto py-3" asChild>
                <Link to={ROUTES.MESSAGES}>
                  <MessageSquare className="mr-3 h-5 w-5" />
                  <div className="text-left">
                    <p className="font-medium">Send Messages</p>
                    <p className="text-sm text-muted-foreground">Compose and send WhatsApp messages</p>
                  </div>
                </Link>
              </Button>
              <Button variant="outline" className="justify-start h-auto py-3" asChild>
                <Link to={ROUTES.SETTINGS}>
                  <Activity className="mr-3 h-5 w-5" />
                  <div className="text-left">
                    <p className="font-medium">System Health</p>
                    <p className="text-sm text-muted-foreground">Check API and service status</p>
                  </div>
                </Link>
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </MainLayout>
  );
}
