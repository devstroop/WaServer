import { useCallback, useEffect, useState } from 'react';
import { Link, useLocation, useParams } from 'react-router-dom';
import { Alert, Badge, Button, Card } from '@devstroop/react-uikit';
import { instances } from '../api/endpoints';
import type { InstanceInfo, WhatsAppStatus } from '../api/types';
import { LinkPanel } from '../components/LinkPanel';

type BadgeTone = 'neutral' | 'primary' | 'success' | 'warning' | 'danger';

function statusTone(s: string): BadgeTone {
  if (s === 'active' || s === 'connected') return 'success';
  if (s === 'warming_up') return 'warning';
  if (s === 'sleeping') return 'neutral';
  if (s === 'error') return 'danger';
  return 'primary';
}

export default function InstanceDetail() {
  const params = useParams<{ id: string; instanceId: string }>();
  const id = params.id ?? params.instanceId ?? '';
  const location = useLocation();
  const isAdminRoute = location.pathname.startsWith('/admin');
  const backTo = isAdminRoute ? '/admin/instances' : '/dashboard/instances';

  const [instance, setInstance] = useState<InstanceInfo | null>(null);
  const [instanceError, setInstanceError] = useState<string | null>(null);
  const [status, setStatus] = useState<WhatsAppStatus | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [loadingInstance, setLoadingInstance] = useState(true);

  const fetchInstance = useCallback(async () => {
    if (!id) return;
    try {
      const info = await instances.get(id);
      setInstance(info);
      setInstanceError(null);
    } catch (e) {
      setInstanceError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoadingInstance(false);
    }
  }, [id]);

  const fetchStatus = useCallback(async () => {
    if (!id) return;
    try {
      const s = await instances.status(id);
      setStatus(s);
      setStatusError(null);
    } catch (e) {
      setStatusError(e instanceof Error ? e.message : String(e));
    }
  }, [id]);

  useEffect(() => {
    void fetchInstance();
  }, [fetchInstance]);

  useEffect(() => {
    void fetchStatus();
    const t = window.setInterval(() => void fetchStatus(), 5000);
    return () => window.clearInterval(t);
  }, [fetchStatus]);

  useEffect(() => {
    if (status?.authorized) {
      // keep instance authorized flag in sync without refetching whole instance
      setInstance((prev) =>
        prev ? { ...prev, authorized: true, status: status.status } : prev,
      );
    }
  }, [status]);

  const authorized = status?.authorized ?? instance?.authorized ?? false;

  if (!id) {
    return (
      <div className="space-y-4">
        <Alert tone="danger" title="Missing id">
          No instance id in route.
        </Alert>
        <Link to={backTo} className="text-sm text-violet-600 hover:underline">
          ← Back to instances
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <Link
          to={backTo}
          className="rounded border bg-white px-3 py-1.5 text-sm hover:bg-zinc-50"
        >
          ← Back
        </Link>
        <h1 className="text-xl font-semibold tracking-tight">
          {instance?.name ?? 'Instance'}{' '}
          <span className="font-mono text-sm font-normal text-zinc-500">{id.slice(0, 8)}</span>
        </h1>
        {status && <Badge tone={statusTone(status.status)}>{status.status}</Badge>}
        {status && (
          <Badge tone={authorized ? 'success' : 'neutral'} variant="soft">
            {authorized ? 'authorized' : 'not authorized'}
          </Badge>
        )}
      </div>

      {instanceError && <Alert tone="danger">{instanceError}</Alert>}
      {statusError && <Alert tone="danger">{statusError}</Alert>}

      <Card header={<span className="font-medium">Details</span>}>
        {loadingInstance ? (
          <div className="text-sm text-zinc-500">Loading…</div>
        ) : instance ? (
          <div className="grid grid-cols-1 gap-3 text-sm md:grid-cols-3">
            <div>
              <div className="text-xs text-zinc-500">ID</div>
              <div className="font-mono break-all">{instance.id}</div>
            </div>
            <div>
              <div className="text-xs text-zinc-500">Name</div>
              <div className="font-medium">{instance.name}</div>
            </div>
            <div>
              <div className="text-xs text-zinc-500">Phone</div>
              <div>{status?.phone_number ?? instance.phone_number ?? '—'}</div>
            </div>
            <div>
              <div className="text-xs text-zinc-500">Status</div>
              <div className="mt-1">
                <Badge tone={statusTone(status?.status ?? instance.status)}>
                  {status?.status ?? instance.status}
                </Badge>
              </div>
            </div>
            <div>
              <div className="text-xs text-zinc-500">Authorized</div>
              <div className="mt-1">
                <Badge tone={authorized ? 'success' : 'neutral'} variant="soft">
                  {authorized ? 'yes' : 'no'}
                </Badge>
              </div>
            </div>
            <div>
              <div className="text-xs text-zinc-500">Updated</div>
              <div>{instance.updated_at ? new Date(instance.updated_at).toLocaleString() : '—'}</div>
            </div>
          </div>
        ) : (
          <div className="text-sm text-zinc-500">No instance data.</div>
        )}
        <div className="mt-3 flex gap-2">
          <Button variant="secondary" onClick={() => void fetchInstance()} size="sm">
            Refresh details
          </Button>
          <Button variant="ghost" onClick={() => void fetchStatus()} size="sm">
            Refresh status (5s poll active)
          </Button>
        </div>
      </Card>

      <Card header={<span className="font-medium">Status</span>}>
        <div className="space-y-2 text-sm">
          {status ? (
            <>
              <div className="flex flex-wrap gap-2">
                <Badge tone={statusTone(status.status)}>{status.status}</Badge>
                <Badge tone={authorized ? 'success' : 'neutral'}>{authorized ? 'authorized' : 'unauthorized'}</Badge>
                {status.phone_number && <span className="text-zinc-600">{status.phone_number}</span>}
              </div>
              <div className="text-xs text-zinc-500">
                Instance {status.instance_id.slice(0, 8)} · polls every 5s · authorized={String(authorized)}
              </div>
            </>
          ) : (
            <div className="text-zinc-500">Loading status… poll 5s</div>
          )}
          {statusError && <Alert tone="warning">{statusError}</Alert>}
        </div>
      </Card>

      <LinkPanel instanceId={id} authorized={!!authorized} onStatusRefresh={fetchStatus} />
    </div>
  );
}

export { LinkPanel };
