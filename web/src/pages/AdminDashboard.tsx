import { useEffect, useState } from 'react';
import { Alert, Badge, Card, Stat, Table } from '@devstroop/react-uikit';
import type { TableColumn } from '@devstroop/react-uikit';
import { health } from '../api/endpoints';
import type { HealthResponse, InstanceMetrics, MetricsResponse } from '../api/types';

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const m = Math.floor(seconds / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  const rem = m % 60;
  return `${h}h ${rem}m`;
}

function statusTone(s: string): 'success' | 'warning' | 'neutral' | 'danger' | 'primary' {
  if (s === 'active') return 'success';
  if (s === 'warming_up') return 'warning';
  if (s === 'sleeping') return 'neutral';
  if (s === 'error') return 'danger';
  return 'primary';
}

export default function AdminDashboard() {
  const [h, setH] = useState<HealthResponse | null>(null);
  const [metrics, setMetrics] = useState<MetricsResponse | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const fetchAll = async () => {
      try {
        const [healthData, metricsData] = await Promise.all([health.get(), health.metrics()]);
        if (cancelled) return;
        setH(healthData);
        setMetrics(metricsData);
        setErr(null);
      } catch (e) {
        if (!cancelled) setErr(e instanceof Error ? e.message : String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    void fetchAll();
    const id = setInterval(() => {
      void Promise.all([health.get().then((v) => !cancelled && setH(v)).catch(() => {}), health.metrics().then((v) => !cancelled && setMetrics(v)).catch(() => {})]);
    }, 5000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  const columns: readonly TableColumn<InstanceMetrics>[] = [
    {
      key: 'id',
      header: 'Instance ID',
      render: (row) => <span className="font-mono text-xs" title={row.id}>{row.id.slice(0, 8)}…</span>,
    },
    {
      key: 'status',
      header: 'Status',
      render: (row) => <Badge tone={statusTone(row.status)} variant="soft">{row.status}</Badge>,
    },
    {
      key: 'authorized',
      header: 'Authorized',
      render: (row) => (
        <Badge tone={row.authorized ? 'success' : 'neutral'} variant="soft">
          {row.authorized ? 'yes' : 'no'}
        </Badge>
      ),
    },
    {
      key: 'total_messages_sent',
      header: 'Sent',
      render: (row) => <span>{row.total_messages_sent}</span>,
    },
    {
      key: 'error_count',
      header: 'Errors',
      render: (row) => <span>{row.error_count}</span>,
    },
    {
      key: 'warmups',
      header: 'Warmups',
      render: (row) => <span>{row.warmups ?? '—'}</span>,
    },
  ];

  if (err) {
    return (
      <div className="space-y-4">
        <Alert tone="danger" title="Failed to load admin metrics">{err}</Alert>
      </div>
    );
  }

  const instancesCount = h?.instances_count ?? metrics?.instances_count ?? (metrics?.instances.length ?? 0);
  const browserAvailable = h?.browser_available;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Admin Dashboard</h1>
        <p className="text-sm text-zinc-500">Global control plane — health &amp; metrics overview.</p>
      </div>

      {browserAvailable === false && (
        <Alert tone="warning" title="Browser unavailable">
          Chromium is not available on this host — instance warmup and QR flows will fail. Install browser dependencies.
        </Alert>
      )}

      <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
        <Card>
          <Stat label="Version" value={h?.version ?? (loading ? '—' : '—')} hint={h?.status ?? 'health'} />
        </Card>
        <Card>
          <Stat
            label="Uptime"
            value={h ? formatUptime(h.uptime_seconds) : '—'}
            hint={h ? `${h.uptime_seconds}s total` : loading ? 'loading…' : '—'}
          />
        </Card>
        <Card>
          <Stat
            label="Instances"
            value={String(instancesCount)}
            hint={metrics ? `${metrics.instances.length} in metrics` : 'global count'}
          />
        </Card>
        <Card>
          <Stat
            label="Browser"
            value={browserAvailable == null ? '—' : browserAvailable ? 'available' : 'missing'}
            hint={browserAvailable == null ? 'checking…' : browserAvailable ? 'Ready for automation' : 'Chromium unavailable'}
            delta={browserAvailable == null ? undefined : browserAvailable ? 'ok' : 'check'}
            deltaTone={browserAvailable ? 'success' : browserAvailable === false ? 'danger' : 'neutral'}
          />
        </Card>
      </div>

      <Card header={<span className="font-medium">Instance Metrics</span>}>
        <div className="mb-3 flex items-center justify-between">
          <p className="text-sm text-zinc-500">
            Per-instance observability from <code className="rounded bg-zinc-100 px-1 py-0.5 text-xs">GET /api/metrics</code>
            {metrics ? ` — uptime ${formatUptime(metrics.uptime_seconds)}, memory ${(metrics.memory_usage_bytes / (1024 * 1024)).toFixed(1)} MB` : ''}
          </p>
          {metrics && (
            <Badge tone="neutral" variant="soft">{metrics.instances_count} total</Badge>
          )}
        </div>
        <Table
          columns={columns}
          rows={metrics?.instances ?? []}
          rowKey={(row) => row.id}
          empty={loading ? 'Loading metrics…' : 'No instances recorded.'}
        />
      </Card>
    </div>
  );
}
