import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Badge, Button, Card, DataFilter, DataGrid, EmptyState } from '@devstroop/react-uikit';
import type { GridColumn, DataFilterProperty } from '@devstroop/react-uikit';
import { instances, users } from '../api/endpoints';
import type { InstanceInfo } from '../api/types';
import { useAuth } from '../hooks/useAuth';

const E164_RE = /^\+[1-9]\d{6,14}$/;

function statusTone(s: string): 'success' | 'warning' | 'neutral' | 'danger' | 'primary' {
  if (s === 'active') return 'success';
  if (s === 'warming_up') return 'warning';
  if (s === 'sleeping') return 'neutral';
  if (s === 'error') return 'danger';
  return 'primary';
}

type InstanceListProps = {
  admin?: boolean;
  isAdmin?: boolean;
  variant?: 'admin' | 'user';
  mode?: 'admin' | 'user' | 'all' | 'scoped';
  scope?: 'admin' | 'user' | 'all' | 'scoped';
  rbac?: boolean;
};

function resolveIsAdmin(props: InstanceListProps): boolean {
  if (props.admin !== undefined) return !!props.admin;
  if (props.isAdmin !== undefined) return !!props.isAdmin;
  if (props.variant === 'admin') return true;
  if (props.variant === 'user') return false;
  if (props.mode === 'admin' || props.mode === 'all') return true;
  if (props.mode === 'user' || props.mode === 'scoped') return false;
  if (props.scope === 'admin' || props.scope === 'all') return true;
  if (props.scope === 'user' || props.scope === 'scoped') return false;
  if (props.rbac) return true;
  return false;
}

export function InstanceList(props: InstanceListProps) {
  const isAdminRoute = resolveIsAdmin(props);
  const navigate = useNavigate();
  const { user, loading: authLoading } = useAuth();
  const isUserAdmin = (user?.role ?? '').toLowerCase() === 'admin';

  const [list, setList] = useState<InstanceInfo[]>([]);
  const [filtered, setFiltered] = useState<InstanceInfo[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState('');
  const [phone, setPhone] = useState('');
  const [formError, setFormError] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const rows = filtered ?? list;

  const filterProperties = useMemo<readonly DataFilterProperty[]>(
    () => [
      { name: 'name', title: 'Name', type: 'string' },
      { name: 'phone_number', title: 'Phone', type: 'string' },
      {
        name: 'status',
        title: 'Status',
        type: 'enum',
        values: [
          { value: 'active', label: 'active' },
          { value: 'warming_up', label: 'warming_up' },
          { value: 'sleeping', label: 'sleeping' },
          { value: 'error', label: 'error' },
        ],
      },
      { name: 'authorized', title: 'Authorized', type: 'boolean' },
    ],
    [],
  );

  const columns = useMemo<GridColumn<InstanceInfo>[]>(
    () => [
      {
        property: 'name',
        title: 'Name',
        sortable: true,
        render: (row) => <span className="font-medium">{row.name}</span>,
      },
      {
        property: 'phone_number',
        title: 'Phone',
        render: (row) => <span className="text-zinc-600">{row.phone_number ?? '—'}</span>,
      },
      {
        property: 'status',
        title: 'Status',
        render: (row) => <Badge tone={statusTone(row.status)}>{row.status}</Badge>,
      },
      {
        property: 'authorized',
        title: 'Authorized',
        type: 'boolean',
        render: (row) => (
          <Badge tone={row.authorized ? 'success' : 'neutral'} variant="soft">
            {row.authorized ? 'yes' : 'no'}
          </Badge>
        ),
      },
      {
        property: 'id',
        title: '',
        sortable: false,
        render: (row) => (
          <Button
            size="sm"
            variant="secondary"
            onClick={() => navigate(isAdminRoute ? `/admin/instances/${row.id}` : `/dashboard/instances/${row.id}`)}
          >
            Open
          </Button>
        ),
      },
    ],
    [navigate, isAdminRoute],
  );

  const load = useCallback(async () => {
    try {
      setError(null);
      if (isAdminRoute) {
        const r = await instances.list();
        setList(r.instances ?? []);
      } else {
        const me = await users.me();
        const resp = (await users.instances(me.id)) as unknown as { instances: unknown[] };
        const raw = resp.instances ?? [];
        if (raw.length === 0) {
          setList([]);
        } else if (raw.length > 0 && typeof raw[0] === 'object' && raw[0] !== null && 'name' in (raw[0] as Record<string, unknown>)) {
          setList(raw as unknown as InstanceInfo[]);
        } else if (raw.length > 0 && typeof raw[0] === 'object' && raw[0] !== null && 'instance_id' in (raw[0] as Record<string, unknown>)) {
          const ids = (raw as unknown as { instance_id: string }[]).map((r) => r.instance_id);
          const infos: InstanceInfo[] = [];
          for (const id of ids) {
            try {
              const info = await instances.get(id);
              infos.push(info);
            } catch {
              // skip failed fetch, still show id as fallback
              infos.push({
                id,
                name: id.slice(0, 8),
                phone_number: null,
                status: 'unknown',
                authorized: false,
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
              } as InstanceInfo);
            }
          }
          setList(infos);
        } else {
          setList(raw as unknown as InstanceInfo[]);
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [isAdminRoute]);

  useEffect(() => {
    setLoading(true);
    void load();
    const id = window.setInterval(() => void load(), 5000);
    return () => window.clearInterval(id);
  }, [load]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    setMsg(null);
    const trimmedName = name.trim();
    const trimmedPhone = phone.trim();
    if (!trimmedName) {
      setFormError('Name is required');
      return;
    }
    if (trimmedPhone && !E164_RE.test(trimmedPhone)) {
      setFormError('phone_number must be E.164 (+15551234567)');
      return;
    }
    setCreating(true);
    try {
      await instances.create({ name: trimmedName, phone_number: trimmedPhone || undefined });
      setName('');
      setPhone('');
      setMsg('Created');
      await load();
    } catch (err) {
      setFormError(err instanceof Error ? err.message : 'Create failed');
    } finally {
      setCreating(false);
    }
  };

  if (!authLoading && isAdminRoute && !isUserAdmin) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold">{isAdminRoute ? 'All Instances' : 'My Instances'}</h1>
        <Card>
          <EmptyState
            title="Forbidden"
            description="Admin access required. Your account does not have administrator privileges."
            action={
              <Button variant="secondary" onClick={() => navigate('/dashboard/instances')}>
                Go to My Instances
              </Button>
            }
          />
        </Card>
      </div>
    );
  }

  const emptyState = (
    <EmptyState
      title={loading ? 'Loading instances…' : 'No instances yet'}
      description={
        isAdminRoute
          ? 'No instances found. Create one to get started.'
          : 'You don’t have any WhatsApp instances assigned. Create one or contact your admin.'
      }
      action={
        <Button variant="primary" onClick={() => document.getElementById('instance-name-input')?.focus()}>
          Create instance
        </Button>
      }
    />
  );

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">{isAdminRoute ? 'All Instances' : 'My Instances'}</h1>
        <p className="text-sm text-zinc-500">
          {isAdminRoute ? 'Global view — all instances (admin). Polls every 5s.' : 'Scoped view — your instances via /users/me. Polls every 5s.'}
        </p>
      </div>

      {error && <div className="rounded bg-red-50 p-3 text-sm text-red-700">{error}</div>}

      <Card header={<span className="font-medium">Create Instance</span>}>
        <form onSubmit={handleCreate} className="flex flex-wrap gap-2">
          <input
            id="instance-name-input"
            className="rounded border px-3 py-2 text-sm"
            placeholder="Name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
          />
          <input
            className="rounded border px-3 py-2 text-sm"
            placeholder="+15551234567 (E.164)"
            value={phone}
            onChange={(e) => setPhone(e.target.value)}
            aria-label="Phone E.164"
          />
          <Button type="submit" variant="primary" disabled={creating}>
            {creating ? 'Creating…' : 'Create'}
          </Button>
          {msg && <span className="self-center text-sm text-green-600">{msg}</span>}
          {formError && <span className="self-center text-sm text-red-600">{formError}</span>}
        </form>
      </Card>

      <Card>
        <div className="space-y-3">
          <DataFilter
            properties={filterProperties}
            items={list}
            viewChanged={(v) => setFiltered(v as InstanceInfo[])}
          />
          <DataGrid
            columns={columns}
            rows={rows}
            rowKey={(row) => row.id}
            allowSorting
            allowPaging
            pageSize={10}
            isLoading={loading}
            empty={emptyState}
            ariaLabel={isAdminRoute ? 'All instances' : 'My instances'}
          />
          {!loading && list.length > 0 && rows.length === 0 && (
            <div className="text-center text-sm text-zinc-500">No results match current filters.</div>
          )}
          <div className="flex justify-between text-xs text-zinc-500">
            <span>
              Showing {rows.length} of {list.length} instances
            </span>
            <Link to={isAdminRoute ? '/dashboard/instances' : '/admin/instances'} className="text-violet-600 hover:underline">
              {isAdminRoute ? 'Go to My Instances' : 'Go to All Instances (admin)'}
            </Link>
          </div>
        </div>
      </Card>
    </div>
  );
}

export default InstanceList;
export const Instances = InstanceList;
