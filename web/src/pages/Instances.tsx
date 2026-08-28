import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { instances } from '../api/endpoints';
import type { InstanceInfo } from '../api/types';

export default function Instances() {
  const [list, setList] = useState<InstanceInfo[]>([]);
  const [name, setName] = useState('');
  const [phone, setPhone] = useState('');
  const [msg, setMsg] = useState<string | null>(null);

  const load = () => instances.list().then((r) => setList(r.instances)).catch(() => {});
  useEffect(() => {
    load();
    const id = setInterval(load, 5000);
    return () => clearInterval(id);
  }, []);

  const create = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await instances.create({ name: name || 'unknown', phone_number: phone || undefined });
      setName('');
      setPhone('');
      setMsg('Created');
      load();
    } catch (e: unknown) {
      setMsg(e instanceof Error ? e.message : 'Create failed');
    }
  };

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-semibold">Instances</h1>

      <form onSubmit={create} className="flex flex-wrap gap-2 rounded border bg-white p-3">
        <input className="rounded border px-3 py-2 text-sm" placeholder="Name" value={name} onChange={(e) => setName(e.target.value)} required />
        <input className="rounded border px-3 py-2 text-sm" placeholder="+15551234567 (E.164)" value={phone} onChange={(e) => setPhone(e.target.value)} />
        <button className="rounded bg-violet-600 px-4 py-2 text-sm text-white">Create</button>
        {msg && <span className="self-center text-sm text-zinc-600">{msg}</span>}
      </form>

      <div className="overflow-hidden rounded border bg-white">
        <table className="w-full text-sm">
          <thead className="bg-zinc-50 text-left text-xs text-zinc-500">
            <tr>
              <th className="p-2">Name</th>
              <th className="p-2">Phone</th>
              <th className="p-2">Status</th>
              <th className="p-2"></th>
            </tr>
          </thead>
          <tbody>
            {list.map((r) => (
              <tr key={r.id} className="border-t">
                <td className="p-2">{r.name}</td>
                <td className="p-2">{r.phone_number || '—'}</td>
                <td className="p-2">{r.status}</td>
                <td className="p-2 text-right">
                  <Link to={`/instances/${r.id}`} className="rounded border px-2 py-1 text-xs hover:bg-zinc-50">
                    Open
                  </Link>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
