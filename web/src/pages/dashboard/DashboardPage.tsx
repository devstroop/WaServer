import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { 
  MessageSquare, 
  CheckCircle, 
  AlertCircle,
  Plus,
  ArrowRight
} from 'lucide-react';
import { Header } from '@/components/layout';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Skeleton } from '@/components/ui';
import { instancesApi, type Instance } from '@/api/instances';

interface StatCard {
  title: string;
  value: string | number;
  icon: React.ComponentType<{ className?: string }>;
  color: string;
}

export function DashboardPage() {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadInstances();
  }, []);

  const loadInstances = async () => {
    try {
      const data = await instancesApi.listInstances();
      setInstances(data);
    } catch (error) {
      console.error('Failed to load instances:', error);
    } finally {
      setLoading(false);
    }
  };

  const connectedCount = instances.filter(i => i.status === 'connected').length;
  const disconnectedCount = instances.length - connectedCount;

  const stats: StatCard[] = [
    {
      title: 'Total Instances',
      value: instances.length,
      icon: MessageSquare,
      color: 'bg-primary-500',
    },
    {
      title: 'Connected',
      value: connectedCount,
      icon: CheckCircle,
      color: 'bg-green-500',
    },
    {
      title: 'Disconnected',
      value: disconnectedCount,
      icon: AlertCircle,
      color: 'bg-red-500',
    },
  ];

  return (
    <>
      <Header 
        title="Dashboard" 
        description="Overview of your WhatsApp instances"
        actions={
          <Button asChild>
            <Link to="/instances/new">
              <Plus className="h-4 w-4 mr-2" />
              New Instance
            </Link>
          </Button>
        }
      />

      <div className="p-6 space-y-6">
        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {loading ? (
            Array.from({ length: 3 }).map((_, i) => (
              <Card key={i}>
                <CardContent className="p-6">
                  <Skeleton className="h-12 w-12 rounded-lg mb-3" />
                  <Skeleton className="h-4 w-24 mb-2" />
                  <Skeleton className="h-8 w-16" />
                </CardContent>
              </Card>
            ))
          ) : (
            stats.map((stat) => (
              <Card key={stat.title}>
                <CardContent className="p-6">
                  <div className={`h-12 w-12 rounded-lg ${stat.color} flex items-center justify-center mb-3`}>
                    <stat.icon className="h-6 w-6 text-white" />
                  </div>
                  <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                    {stat.title}
                  </p>
                  <p className="text-3xl font-bold text-text-light dark:text-text-dark">
                    {stat.value}
                  </p>
                </CardContent>
              </Card>
            ))
          )}
        </div>

        {/* Recent Instances */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle>Recent Instances</CardTitle>
              <Button variant="ghost" size="sm" asChild>
                <Link to="/instances">
                  View All
                  <ArrowRight className="h-4 w-4 ml-1" />
                </Link>
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-3">
                {Array.from({ length: 3 }).map((_, i) => (
                  <div key={i} className="flex items-center justify-between p-3 rounded-lg bg-bg-surface-light dark:bg-bg-surface-dark">
                    <div className="flex items-center gap-3">
                      <Skeleton className="h-10 w-10 rounded-lg" />
                      <div>
                        <Skeleton className="h-4 w-32 mb-1" />
                        <Skeleton className="h-3 w-24" />
                      </div>
                    </div>
                    <Skeleton className="h-6 w-20" />
                  </div>
                ))}
              </div>
            ) : instances.length === 0 ? (
              <div className="text-center py-8">
                <MessageSquare className="h-12 w-12 mx-auto text-text-muted-light dark:text-text-muted-dark mb-3" />
                <p className="text-text-muted-light dark:text-text-muted-dark mb-3">
                  No instances yet
                </p>
                <Button asChild>
                  <Link to="/instances/new">
                    <Plus className="h-4 w-4 mr-2" />
                    Create First Instance
                  </Link>
                </Button>
              </div>
            ) : (
              <div className="space-y-2">
                {instances.slice(0, 5).map((instance) => (
                  <Link
                    key={instance.id}
                    to={`/instances/${instance.id}`}
                    className="flex items-center justify-between p-3 rounded-lg bg-bg-subtle-light dark:bg-bg-elevated-dark hover:bg-bg-hover-light dark:hover:bg-bg-hover-dark transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <div className="h-10 w-10 rounded-lg bg-primary-500/10 flex items-center justify-center">
                        <MessageSquare className="h-5 w-5 text-primary-500" />
                      </div>
                      <div>
                        <p className="font-medium text-text-light dark:text-text-dark">
                          {instance.instance_name || instance.id}
                        </p>
                        <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                          {instance.phone_number || 'Not linked'}
                        </p>
                      </div>
                    </div>
                    <Badge variant={instance.status === 'connected' ? 'success' : 'error'}>
                      {instance.status}
                    </Badge>
                  </Link>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
