import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Header } from '@/components/layout';
import { Card, CardContent, CardHeader, CardTitle, Input, Button } from '@/components/ui';
import { instancesApi } from '@/api/instances';

export function NewInstancePage() {
  const navigate = useNavigate();
  const [instanceId, setInstanceId] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      await instancesApi.createInstance(instanceId);
      navigate(`/instances/${instanceId}/link`);
    } catch (err: any) {
      setError(err.message || 'Failed to create instance');
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Header 
        title="New Instance" 
        description="Create a new WhatsApp instance"
      />

      <div className="p-6 max-w-lg">
        <Card>
          <CardHeader>
            <CardTitle>Instance Details</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              {error && (
                <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/20">
                  <p className="text-sm text-red-500">{error}</p>
                </div>
              )}

              <Input
                label="Instance ID"
                placeholder="my-instance"
                value={instanceId}
                onChange={(e) => setInstanceId(e.target.value.toLowerCase().replace(/[^a-z0-9-_]/g, ''))}
                error={instanceId.length > 0 && instanceId.length < 3 ? 'Minimum 3 characters' : undefined}
              />

              <p className="text-sm text-text-muted-light dark:text-text-muted-dark">
                Use lowercase letters, numbers, hyphens, and underscores only.
              </p>

              <div className="flex gap-3 pt-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => navigate('/instances')}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={instanceId.length < 3 || loading}
                  isLoading={loading}
                >
                  Create & Link
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
