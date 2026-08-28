import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Alert, Card } from '@devstroop/react-uikit';
import { users } from '../api/endpoints';
import type { UserInfo } from '../api/types';
import { AssignmentsPanel } from '../components/AssignmentsPanel';
import { useAuth } from '../hooks/useAuth';

function formatDate(value: string | null | undefined): string {
  if (!value) return '—';
  try {
    const d = new Date(value);
    if (Number.isNaN(d.getTime())) return value;
    return d.toLocaleString();
  } catch {
    return value;
  }
}

export default function AdminUserAssignments() {
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
        <h1 className="text-xl font-semibold tracking-tight">Instance assignments</h1>
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
        <h1 className="text-xl font-semibold tracking-tight">Instance assignments</h1>
        <Card>
          <div className="p-6 text-sm text-zinc-500">Loading user…</div>
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <Link to="/admin/users" className="text-sm text-violet-600 hover:underline">
            ← Users
          </Link>
          <h1 className="text-xl font-semibold tracking-tight">Instance assignments</h1>
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
        <h1 className="text-xl font-semibold tracking-tight">Instance assignments</h1>
        <Alert tone="warning" title="Not found">
          User not found.
        </Alert>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <Link to="/admin/users" className="text-sm text-violet-600 hover:underline">
          ← Users
        </Link>
        <h1 className="text-xl font-semibold tracking-tight">Instance assignments</h1>
        <span className="rounded-full bg-zinc-100 px-2 py-0.5 text-xs text-zinc-700">{data.id.slice(0, 8)}</span>
      </div>

      <Card header="User">
        <div className="grid grid-cols-1 gap-2 p-4 text-sm md:grid-cols-3">
          <div>
            <div className="text-xs text-zinc-500">Username</div>
            <div className="font-medium">{data.username}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-500">Email</div>
            <div>{data.email ?? '—'}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-500">Role</div>
            <div className="capitalize">{data.role}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-500">Created</div>
            <div>{formatDate(data.created_at)}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-500">Active</div>
            <div>{data.is_active ? 'Yes' : 'No'}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-500">ID</div>
            <div className="font-mono text-xs break-all">{data.id}</div>
          </div>
        </div>
      </Card>

      <AssignmentsPanel userId={data.id} />
    </div>
  );
}
