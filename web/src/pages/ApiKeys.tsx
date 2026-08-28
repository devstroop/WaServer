import { useEffect, useState } from 'react';
import { Alert, Card } from '@devstroop/react-uikit';
import { users } from '../api/endpoints';
import type { UserInfo } from '../api/types';
import { TokensPanel } from '../components/TokensPanel';

export default function ApiKeys() {
  const [me, setMe] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setLoading(true);
        const u = await users.me();
        if (!cancelled) {
          setMe(u);
          setError(null);
        }
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (loading) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">API Keys</h1>
        <Card>
          <div className="p-6 text-sm text-zinc-400">Loading…</div>
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">API Keys</h1>
        <Alert tone="danger" title="Failed to load profile">
          {error}
        </Alert>
      </div>
    );
  }

  if (!me) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">API Keys</h1>
        <Alert tone="warning" title="Not found">
          Could not determine current user.
        </Alert>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <h1 className="text-xl font-semibold tracking-tight">API Keys</h1>
        <span className="rounded-full bg-zinc-800 px-2 py-0.5 text-xs text-zinc-200">{me.username}</span>
        <span className="rounded-full bg-zinc-800 px-2 py-0.5 text-xs text-zinc-200">{me.role}</span>
      </div>
      <Alert tone="info" title="Self-managed tokens">
        Create and revoke your own access tokens. Tokens are shown only once at creation — copy and store securely.
      </Alert>
      <TokensPanel userId={me.id} />
    </div>
  );
}
