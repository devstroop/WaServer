import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { 
  MessageSquare, 
  Plus, 
  Search,
  Trash2,
  Settings,
  QrCode
} from 'lucide-react';
import { Header } from '@/components/layout';
import { Card, CardContent, Input, Button, Badge, SkeletonTable } from '@/components/ui';
import { instancesApi, type Instance } from '@/api/instances';

export function InstancesListPage() {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');

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

  const handleDelete = async (instanceId: string) => {
    if (!confirm(`Are you sure you want to delete instance "${instanceId}"?`)) {
      return;
    }

    try {
      await instancesApi.deleteInstance(instanceId);
      setInstances(instances.filter(i => i.id !== instanceId));
    } catch (error) {
      console.error('Failed to delete instance:', error);
    }
  };

  const filteredInstances = instances.filter(instance =>
    instance.id.toLowerCase().includes(search.toLowerCase()) ||
    instance.instance_name?.toLowerCase().includes(search.toLowerCase()) ||
    instance.phone_number?.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <>
      <Header 
        title="Instances" 
        description="Manage your WhatsApp instances"
        actions={
          <Button asChild>
            <Link to="/instances/new">
              <Plus className="h-4 w-4 mr-2" />
              New Instance
            </Link>
          </Button>
        }
      />

      <div className="p-6">
        <Card>
          <CardContent className="p-4">
            {/* Search */}
            <div className="mb-4">
              <Input
                placeholder="Search instances..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                leftIcon={<Search className="h-4 w-4" />}
              />
            </div>

            {/* Table */}
            {loading ? (
              <SkeletonTable rows={5} />
            ) : filteredInstances.length === 0 ? (
              <div className="text-center py-12">
                <MessageSquare className="h-12 w-12 mx-auto text-text-muted-light dark:text-text-muted-dark mb-3" />
                <p className="text-text-muted-light dark:text-text-muted-dark mb-3">
                  {search ? 'No instances found matching your search' : 'No instances yet'}
                </p>
                {!search && (
                  <Button asChild>
                    <Link to="/instances/new">
                      <Plus className="h-4 w-4 mr-2" />
                      Create Instance
                    </Link>
                  </Button>
                )}
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-border-light dark:border-border-dark">
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Instance
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Phone
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Status
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Created
                      </th>
                      <th className="text-right py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredInstances.map((instance) => (
                      <tr 
                        key={instance.id}
                        className="border-b border-border-light dark:border-border-dark last:border-0 hover:bg-bg-surface-light dark:hover:bg-bg-surface-dark"
                      >
                        <td className="py-3 px-4">
                          <Link 
                            to={`/instances/${instance.id}`}
                            className="flex items-center gap-3"
                          >
                            <div className="h-10 w-10 rounded-lg bg-primary-500/10 flex items-center justify-center">
                              <MessageSquare className="h-5 w-5 text-primary-500" />
                            </div>
                            <span className="font-medium text-text-light dark:text-text-dark hover:text-primary-500">
                              {instance.instance_name || instance.id}
                            </span>
                          </Link>
                        </td>
                        <td className="py-3 px-4 text-text-muted-light dark:text-text-muted-dark">
                          {instance.phone_number || '—'}
                        </td>
                        <td className="py-3 px-4">
                          <Badge variant={instance.status === 'connected' ? 'success' : 'error'}>
                            {instance.status}
                          </Badge>
                        </td>
                        <td className="py-3 px-4 text-text-muted-light dark:text-text-muted-dark">
                          {instance.created_at ? new Date(instance.created_at).toLocaleDateString() : '—'}
                        </td>
                        <td className="py-3 px-4">
                          <div className="flex items-center justify-end gap-1">
                            {instance.status !== 'connected' && (
                              <Button variant="ghost" size="icon" asChild>
                                <Link to={`/instances/${instance.id}/link`}>
                                  <QrCode className="h-4 w-4" />
                                </Link>
                              </Button>
                            )}
                            <Button variant="ghost" size="icon" asChild>
                              <Link to={`/instances/${instance.id}`}>
                                <Settings className="h-4 w-4" />
                              </Link>
                            </Button>
                            <Button 
                              variant="ghost" 
                              size="icon"
                              onClick={() => handleDelete(instance.id)}
                            >
                              <Trash2 className="h-4 w-4 text-red-500" />
                            </Button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
