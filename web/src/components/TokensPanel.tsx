import { useCallback, useEffect, useState } from 'react';
import {
  Alert,
  Button,
  Card,
  DataGrid,
  Dialog,
  Field,
  Input,
  useToast,
} from '@devstroop/react-uikit';
import type { GridColumn } from '@devstroop/react-uikit';
import { users } from '../api/endpoints';
import type { AccessTokenInfo } from '../api/types';

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

type Revealed = { token_info: AccessTokenInfo; access_token: string } | null;

export function TokensPanel({ userId }: { userId: string }) {
  const { toast } = useToast();
  const [tokens, setTokens] = useState<AccessTokenInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [name, setName] = useState('');
  const [expiresStr, setExpiresStr] = useState('');
  const [creating, setCreating] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);
  const [revealed, setRevealed] = useState<Revealed>(null);

  const [deleteTarget, setDeleteTarget] = useState<AccessTokenInfo | null>(null);
  const [deleting, setDeleting] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const res = await users.tokens(userId);
      setTokens(res.tokens ?? []);
    } catch (e) {
      setLoadError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    void load();
  }, [load]);

  const handleCreate = useCallback(async () => {
    const trimmed = name.trim();
    if (!trimmed) {
      setCreateError('Name is required');
      return;
    }
    let expires: number | null = null;
    if (expiresStr.trim() !== '') {
      const parsed = Number.parseInt(expiresStr.trim(), 10);
      if (!Number.isFinite(parsed) || parsed <= 0) {
        setCreateError('expires_in_days must be a positive integer');
        return;
      }
      expires = parsed;
    }
    setCreating(true);
    setCreateError(null);
    try {
      const res = await users.createToken(userId, {
        name: trimmed,
        expires_in_days: expires,
      });
      setTokens((prev) => [...prev, res.token_info]);
      setRevealed(res);
      setCreateOpen(false);
      setName('');
      setExpiresStr('');
      toast({
        title: 'Token created',
        description: 'Copy the access token now — it will not be shown again.',
        tone: 'warning',
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setCreateError(msg);
      toast({ title: 'Create failed', description: msg, tone: 'danger' });
    } finally {
      setCreating(false);
    }
  }, [userId, name, expiresStr, toast]);

  const handleDelete = useCallback(async () => {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await users.deleteToken(userId, deleteTarget.id);
      setTokens((prev) => prev.filter((t) => t.id !== deleteTarget.id));
      toast({ title: 'Deleted', description: `Token "${deleteTarget.name}" deleted.`, tone: 'success' });
      setDeleteTarget(null);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast({ title: 'Delete failed', description: msg, tone: 'danger' });
    } finally {
      setDeleting(false);
    }
  }, [userId, deleteTarget, toast]);

  const copyToken = useCallback(async () => {
    if (!revealed?.access_token) return;
    try {
      await navigator.clipboard.writeText(revealed.access_token);
      toast({ title: 'Copied', description: 'Access token copied to clipboard.', tone: 'success' });
    } catch {
      toast({ title: 'Copy failed', description: 'Please copy manually.', tone: 'danger' });
    }
  }, [revealed, toast]);

  const columns: GridColumn<AccessTokenInfo>[] = [
    { property: 'name', title: 'Name', sortable: true },
    {
      property: 'expires_at',
      title: 'Expires',
      render: (row: AccessTokenInfo) => formatDate(row.expires_at),
    },
    {
      property: 'last_used',
      title: 'Last used',
      render: (row: AccessTokenInfo) => formatDate(row.last_used),
    },
    {
      property: 'created_at',
      title: 'Created',
      render: (row: AccessTokenInfo) => formatDate(row.created_at),
    },
    {
      property: '__actions',
      title: 'Actions',
      render: (row: AccessTokenInfo) => (
        <Button
          variant="danger"
          size="sm"
          onClick={(e: React.MouseEvent) => {
            e.stopPropagation();
            setDeleteTarget(row);
          }}
        >
          Delete
        </Button>
      ),
    },
  ];

  return (
    <>
      <Card header="Access tokens">
        <div className="space-y-3 p-4">
          {loadError && (
            <Alert tone="danger" title="Failed to load tokens">
              {loadError}
            </Alert>
          )}
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="text-sm text-zinc-600">Manage access tokens for this user. Tokens are shown only once at creation.</div>
            <Button variant="primary" onClick={() => setCreateOpen(true)}>
              Create token
            </Button>
          </div>
          <DataGrid
            columns={columns}
            rows={tokens}
            rowKey={(row: AccessTokenInfo) => row.id}
            isLoading={loading}
            empty="No tokens yet"
            ariaLabel="Access tokens"
          />
        </div>
      </Card>

      <Dialog
        open={createOpen}
        onClose={() => {
          if (!creating) setCreateOpen(false);
        }}
        title="Create access token"
        description="Create with name + expires_in_days. Token will be shown once."
        size="sm"
        footer={
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setCreateOpen(false)} disabled={creating}>
              Cancel
            </Button>
            <Button variant="primary" onClick={() => void handleCreate()} disabled={creating}>
              {creating ? 'Creating…' : 'Create'}
            </Button>
          </div>
        }
      >
        <div className="space-y-3">
          {createError && (
            <Alert tone="danger" title="Error">
              {createError}
            </Alert>
          )}
          <Field label="Name" htmlFor="token-name" required>
            <Input
              id="token-name"
              value={name}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
              placeholder="my-token"
              disabled={creating}
            />
          </Field>
          <Field label="Expires in days" htmlFor="token-expires" hint="Leave blank for never expires">
            <Input
              id="token-expires"
              type="number"
              min={1}
              value={expiresStr}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setExpiresStr(e.target.value)}
              placeholder="30"
              disabled={creating}
            />
          </Field>
        </div>
      </Dialog>

      <Dialog
        open={!!revealed}
        onClose={() => setRevealed(null)}
        title="Access token"
        description="Copy now — will not be shown again"
        size="md"
        footer={
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={copyToken}>
              Copy
            </Button>
            <Button variant="primary" onClick={() => setRevealed(null)}>
              Done
            </Button>
          </div>
        }
      >
        <div className="space-y-3">
          <Alert tone="warning" title="One-time token">
            Store this access_token securely. It will not be retrievable again. Copy and keep it safe.
          </Alert>
          <Field label="Access token" htmlFor="revealed-token">
            <div className="flex gap-2">
              <Input id="revealed-token" readOnly value={revealed?.access_token ?? ''} />
              <Button variant="secondary" onClick={() => void copyToken()}>
                Copy
              </Button>
            </div>
          </Field>
          {revealed?.token_info && (
            <div className="rounded border bg-zinc-50 p-3 text-xs">
              <div>
                Name: <span className="font-medium">{revealed.token_info.name}</span>
              </div>
              <div>ID: {revealed.token_info.id}</div>
              <div>Expires: {formatDate(revealed.token_info.expires_at)}</div>
            </div>
          )}
        </div>
      </Dialog>

      <Dialog
        open={!!deleteTarget}
        onClose={() => {
          if (!deleting) setDeleteTarget(null);
        }}
        title="Delete token"
        description={deleteTarget ? `Delete "${deleteTarget.name}"? This cannot be undone.` : undefined}
        size="sm"
        footer={
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setDeleteTarget(null)} disabled={deleting}>
              Cancel
            </Button>
            <Button variant="danger" onClick={() => void handleDelete()} disabled={deleting}>
              {deleting ? 'Deleting…' : 'Delete'}
            </Button>
          </div>
        }
      >
        {deleteTarget && (
          <Alert tone="danger" title="Confirm deletion">
            This will permanently delete token <span className="font-medium">{deleteTarget.name}</span> ({deleteTarget.id.slice(0, 8)}).
          </Alert>
        )}
      </Dialog>
    </>
  );
}
