import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Badge, Button, Card, DataGrid, EmptyState, Stat, useToast } from '@devstroop/react-uikit';
import type { GridColumn } from '@devstroop/react-uikit';
import { health, users } from '../api/endpoints';
import type { HealthResponse, InstanceInfo } from '../api/types';

function statusTone(s: string): 'success' | 'warning' | 'neutral' | 'danger' | 'primary' {
  if (s === 'active') return 'success';
  if (s === 'warming_up') return 'warning';
  if (s === 'sleeping') return 'neutral';
  if (s === 'error') return 'danger';
  return 'primary';
}

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const m = Math.floor(seconds / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  const rem = m % 60;
  return `${h}h ${rem}m`;
}

export default function Dashboard() {
  const [h, setH] = useState<HealthResponse | null>(null);
  const [list, setList] = useState<InstanceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  // useToast may throw if no provider - handle gracefully but still require provider for tests
  let toastApi: ReturnType<typeof useToast> | null = null;
  try {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    toastApi = useToast();
  } catch {
    toastApi = null;
  }

  const showError = useCallback(
    (e: unknown, title: string) => {
      const err = e as Error & { correlationId?: string; status?: number };
      const message = err?.message ?? String(e);
      const correlationId = (err as unknown as { correlationId?: string })?.correlationId;
      const description = correlationId ? `${message} (ref: ${correlationId})` : message;
      if (toastApi) {
        toastApi.toast({ tone: 'danger', title, description });
      } else {
        // fallback if provider missing - still surface error
        console.error(title, description);
      }
    },
    [toastApi],
  );

  useEffect(() => {
    let cancelled = false;

    const fetchHealth = async () => {
      try {
        const data = await health.get();
        if (!cancelled) setH(data);
      } catch (e) {
        if (!cancelled) showError(e, 'Failed to load health');
      }
    };

    const fetchScopedInstances = async () => {
      try {
        const me = await users.me();
        if (cancelled) return;
        const resp = await users.instances(me.id);
        if (!cancelled) {
          setList(resp.instances ?? []);
        }
      } catch (e) {
        if (!cancelled) showError(e, 'Failed to load instances');
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    const loadAll = async () => {
      setLoading(true);
      await Promise.all([fetchHealth(), fetchScopedInstances()]);
    };

    void loadAll();

    const id = window.setInterval(() => {
      void fetchHealth();
      void fetchScopedInstances();
    }, 5000);

    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [showError]);

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
        property: 'id',
        title: '',
        sortable: false,
        render: (row) => (
          <Button size="sm" variant="secondary" onClick={() => navigate(`/app/instances/${row.id}`)}>
            Send
          </Button>
        ),
      },
    ],
    [navigate],
  );

  const emptyState = useMemo(
    () => (
      <EmptyState
        title="No instances yet"
        description="You don't have any WhatsApp instances assigned. Contact your admin or create one if you have permission."
        action={
          <Button variant="primary" onClick={() => navigate('/app/instances')}>
            Go to Instances
          </Button>
        }
      />
    ),
    [navigate],
  );

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Dashboard</h1>
        <p className="text-sm text-zinc-500">Campaign and API overview — scoped to your account.</p>
      </div>

      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        <Card>
          <Stat label="Version" value={h?.version ?? '—'} hint={h?.status ?? 'health'} />
        </Card>
        <Card>
          <Stat
            label="Uptime"
            value={h ? formatUptime(h.uptime_seconds) : '—'}
            hint={h ? `${h.uptime_seconds}s total` : 'loading…'}
          />
        </Card>
        <Card>
          <Stat
            label="Browser"
            value={h?.browser_available ? 'available' : h ? 'missing' : '—'}
            hint={h?.browser_available ? 'Ready for automation' : 'Chromium unavailable'}
            delta={h?.browser_available ? 'ok' : 'check'}
            deltaTone={h?.browser_available ? 'success' : 'danger'}
          />
        </Card>
      </div>

      <Card header={<span className="font-medium">My Instances</span>} footer={null}>
        <div className="mb-3 flex items-center justify-between">
          <p className="text-sm text-zinc-500">Instances assigned to your user — quick send access.</p>
          <Link to="/app/instances" className="text-sm text-violet-600 hover:underline">
            Manage →
          </Link>
        </div>
        <DataGrid
          columns={columns}
          rows={list}
          rowKey={(row) => row.id}
          isLoading={loading}
          empty={emptyState}
          ariaLabel="My instances"
        />
        {!loading && list.length > 0 && (
          <div className="mt-3 flex justify-end">
            <Button variant="ghost" size="sm" onClick={() => navigate('/app/instances')}>
              View all
            </Button>
          </div>
        )}
      </Card>
    </div>
  );
}
