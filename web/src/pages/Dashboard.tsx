import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { health, instances } from '../api/endpoints';
import type { HealthResponse, InstanceInfo } from '../api/types';

export default function Dashboard() {
  const [h, setH] = useState<HealthResponse | null>(null);
  const [list, setList] = useState<InstanceInfo[]>([]);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    health
      .get()
      .then(setH)
      .catch((e) => setErr(e.message));
    instances
      .list()
      .then((r) => setList(r.instances))
      .catch(() => {});
    const id = setInterval(() => {
      health.get().then(setH).catch(() => {});
      instances.list().then((r) => setList(r.instances)).catch(() => {});
    }, 5000);
    return () => clearInterval(id);
  }, []);

  if (err) return <div className="rounded bg-red-50 p-3 text-sm text-red-700">{err}</div>;

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
        <Card title="Version" value={h?.version ?? '—'} />
        <Card title="Uptime" value={h ? `${Math.floor(h.uptime_seconds / 60)}m` : '—'} />
        <Card title="Instances" value={h?.instances_count ?? list.length} />
        <Card title="Browser" value={h?.browser_available ? 'available' : 'missing'} />
      </div>

      <div className="rounded border bg-white">
        <div className="flex items-center justify-between border-b px-4 py-3">
          <h2 className="font-medium">Instances</h2>
          <Link to="/instances" className="text-sm text-violet-600">
            Manage →
          </Link>
        </div>
        <div className="divide-y">
          {list.length === 0 && <div className="p-4 text-sm text-zinc-500">No instances yet.</div>}
          {list.map((i) => (
            <Link key={i.id} to={`/instances/${i.id}`} className="flex items-center justify-between px-4 py-3 hover:bg-zinc-50">
              <div>
                <div className="text-sm font-medium">{i.name}</div>
                <div className="text-xs text-zinc-500">{i.phone_number || '—'} · {i.id.slice(0, 8)}</div>
              </div>
              <span className={`rounded-full px-2 py-1 text-xs ${badge(i.status)}`}>{i.status}</span>
            </Link>
          ))}
        </div>
      </div>
    </div>
  );
}

function Card({ title, value }: { title: string; value: string | number }) {
  return (
    <div className="rounded border bg-white p-4">
      <div className="text-xs text-zinc-500">{title}</div>
      <div className="text-lg font-semibold">{value as string}</div>
    </div>
  );
}

function badge(s: string) {
  if (s === 'active') return 'bg-green-100 text-green-700';
  if (s === 'warming_up') return 'bg-amber-100 text-amber-700';
  if (s === 'sleeping') return 'bg-zinc-100 text-zinc-700';
  return 'bg-red-100 text-red-700';
}
