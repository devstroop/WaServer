import { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { 
  MessageSquare, 
  QrCode, 
  Trash2, 
  RefreshCw,
  Phone,
  Calendar,
  Activity
} from 'lucide-react';
import { Header } from '@/components/layout';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Skeleton } from '@/components/ui';
import { instancesApi, type Instance } from '@/api/instances';

export function InstanceDetailPage() {
  const { instanceId } = useParams<{ instanceId: string }>();
  const navigate = useNavigate();
  const [instance, setInstance] = useState<Instance | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadInstance();
  }, [instanceId]);

  const loadInstance = async () => {
    if (!instanceId) return;
    try {
      const data = await instancesApi.getInstance(instanceId);
      setInstance(data);
    } catch (error) {
      console.error('Failed to load instance:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!instanceId) return;
    if (!confirm(`Are you sure you want to delete instance "${instanceId}"?`)) {
      return;
    }

    try {
      await instancesApi.deleteInstance(instanceId);
      navigate('/instances');
    } catch (error) {
      console.error('Failed to delete instance:', error);
    }
  };

  const handleDisconnect = async () => {
    if (!instanceId) return;
    if (!confirm('Are you sure you want to disconnect this instance?')) {
      return;
    }

    try {
      await instancesApi.disconnectInstance(instanceId);
      loadInstance();
    } catch (error) {
      console.error('Failed to disconnect instance:', error);
    }
  };

  if (loading) {
    return (
      <>
        <Header title="Instance Details" />
        <div className="p-6">
          <Card>
            <CardContent className="p-6">
              <div className="space-y-4">
                <Skeleton className="h-8 w-48" />
                <Skeleton className="h-4 w-64" />
                <Skeleton className="h-4 w-32" />
              </div>
            </CardContent>
          </Card>
        </div>
      </>
    );
  }

  if (!instance) {
    return (
      <>
        <Header title="Instance Not Found" />
        <div className="p-6">
          <Card>
            <CardContent className="text-center py-12">
              <MessageSquare className="h-12 w-12 mx-auto text-text-muted-light dark:text-text-muted-dark mb-3" />
              <p className="text-text-muted-light dark:text-text-muted-dark mb-3">
                Instance not found
              </p>
              <Button asChild>
                <Link to="/instances">Back to Instances</Link>
              </Button>
            </CardContent>
          </Card>
        </div>
      </>
    );
  }

  return (
    <>
      <Header 
        title={instance.instance_name || instance.id}
        description="Instance details and management"
        actions={
          <div className="flex gap-2">
            {instance.status !== 'connected' && (
              <Button asChild>
                <Link to={`/instances/${instanceId}/link`}>
                  <QrCode className="h-4 w-4 mr-2" />
                  Link
                </Link>
              </Button>
            )}
            <Button variant="outline" onClick={handleDelete}>
              <Trash2 className="h-4 w-4 mr-2" />
              Delete
            </Button>
          </div>
        }
      />

      <div className="p-6 space-y-6">
        {/* Status Card */}
        <Card>
          <CardHeader>
            <CardTitle>Status</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="flex items-center gap-3">
                <div className="h-10 w-10 rounded-lg bg-bg-surface-light dark:bg-bg-surface-dark flex items-center justify-center">
                  <Activity className="h-5 w-5 text-text-muted-light dark:text-text-muted-dark" />
                </div>
                <div>
                  <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                    Status
                  </p>
                  <Badge variant={instance.status === 'connected' ? 'success' : 'error'}>
                    {instance.status}
                  </Badge>
                </div>
              </div>

              <div className="flex items-center gap-3">
                <div className="h-10 w-10 rounded-lg bg-bg-surface-light dark:bg-bg-surface-dark flex items-center justify-center">
                  <Phone className="h-5 w-5 text-text-muted-light dark:text-text-muted-dark" />
                </div>
                <div>
                  <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                    Phone Number
                  </p>
                  <p className="font-medium text-text-light dark:text-text-dark">
                    {instance.phone_number || 'Not linked'}
                  </p>
                </div>
              </div>

              <div className="flex items-center gap-3">
                <div className="h-10 w-10 rounded-lg bg-bg-surface-light dark:bg-bg-surface-dark flex items-center justify-center">
                  <Calendar className="h-5 w-5 text-text-muted-light dark:text-text-muted-dark" />
                </div>
                <div>
                  <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                    Created
                  </p>
                  <p className="font-medium text-text-light dark:text-text-dark">
                    {instance.created_at ? new Date(instance.created_at).toLocaleDateString() : '—'}
                  </p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Actions Card */}
        <Card>
          <CardHeader>
            <CardTitle>Actions</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-3">
              <Button variant="outline" onClick={loadInstance}>
                <RefreshCw className="h-4 w-4 mr-2" />
                Refresh Status
              </Button>
              
              {instance.status === 'connected' && (
                <Button variant="outline" onClick={handleDisconnect}>
                  Disconnect
                </Button>
              )}
              
              {instance.status !== 'connected' && (
                <Button asChild>
                  <Link to={`/instances/${instanceId}/link`}>
                    <QrCode className="h-4 w-4 mr-2" />
                    Link WhatsApp
                  </Link>
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
