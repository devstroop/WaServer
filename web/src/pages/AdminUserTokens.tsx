import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Alert, Card } from '@devstroop/react-uikit';
import { users } from '../api/endpoints';
import type { UserInfo } from '../api/types';
import { TokensPanel } from '../components/TokensPanel';
import { useAuth } from '../hooks/useAuth';

export default function AdminUserTokens() {
  const { id } = useParams<{ id: string }>();
  const { user: authUser, loading: authLoading } = useAuth();
  const isAdmin = (authUser?.role ?? '').toLowerCase() === 'admin';

  const [data, setData] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) {
      setError('Missing user id');
      setLoading(false);
      return;
    }
    let cancelled = false;
    (async () => {
      try {
        setLoading(true);
        setError(null);
        const u = await users.get(id);
        if (!cancelled) setData(u);
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id]);

  if (!authLoading && !isAdmin) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">User tokens</h1>
        <Card>
          <div className="p-6">
            <Alert tone="danger" title="Forbidden">
              Admin access required.
            </Alert>
          </div>
        </Card>
      </div>
    );
  }

  if (loading || authLoading) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">User tokens</h1>
        <Card>
          <div className="p-6 text-sm text-zinc-400">Loading user…</div>
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <Link to="/admin/users" className="text-sm text-primary hover:underline">
            ← Users
          </Link>
          <h1 className="text-xl font-semibold tracking-tight">User tokens</h1>
        </div>
        <Alert tone="danger" title="Failed to load user">
          {error}
        </Alert>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">User tokens</h1>
        <Alert tone="warning" title="Not found">
          User not found.
        </Alert>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <Link to="/admin/users" className="text-sm text-primary hover:underline">
          ← Users
        </Link>
        <h1 className="text-xl font-semibold tracking-tight">User tokens</h1>
        <span className="rounded-full bg-zinc-800 px-2 py-0.5 text-xs text-zinc-200">{data.id.slice(0, 8)}</span>
      </div>

      <Card header="User">
        <div className="grid grid-cols-1 gap-2 p-4 text-sm md:grid-cols-3">
          <div>
            <div className="text-xs text-zinc-400">Username</div>
            <div className="font-medium">{data.username}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-400">Email</div>
            <div>{data.email ?? '—'}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-400">Role</div>
            <div className="capitalize">{data.role}</div>
          </div>
        </div>
      </Card>

      <TokensPanel userId={data.id} />
    </div>
  );
}
