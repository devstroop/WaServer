import { useCallback, useEffect, useState } from 'react';
import { Link, useParams, useNavigate } from 'react-router-dom';
import {
  Alert,
  Button,
  Card,
  Dialog,
  Field,
  Select,
  Switch,
  useToast,
  Password,
} from '@devstroop/react-uikit';
import { users } from '../api/endpoints';
import type { UserInfo } from '../api/types';
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

export default function UserDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { user: authUser, loading: authLoading } = useAuth();
  const { toast } = useToast();

  const isAdmin = (authUser?.role ?? '').toLowerCase() === 'admin';

  const [data, setData] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [role, setRole] = useState<string>('user');
  const [isActive, setIsActive] = useState<boolean>(true);
  const [password, setPassword] = useState<string>('');
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [formSuccess, setFormSuccess] = useState<string | null>(null);

  const [deleteOpen, setDeleteOpen] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const isSelf = !!authUser && !!data && authUser.id === data.id;

  const load = useCallback(async () => {
    if (!id) {
      setLoadError('Missing user id');
      setLoading(false);
      return;
    }
    try {
      setLoadError(null);
      setLoading(true);
      const u = await users.get(id);
      setData(u);
      setRole((u.role ?? 'user').toLowerCase());
      setIsActive(!!u.is_active);
      setPassword('');
      setFormError(null);
    } catch (e) {
      setLoadError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    void load();
  }, [load]);

  const handleSave = useCallback(async () => {
    if (!id || !data) return;
    setFormError(null);
    setFormSuccess(null);

    if (isSelf) {
      const wantsDemote = role !== (data.role ?? '').toLowerCase() && role !== 'admin';
      const wantsDeactivate = isActive === false && data.is_active === true;
      if (wantsDemote || wantsDeactivate) {
        const msg = 'Self-demotion and self-deactivation are blocked.';
        setFormError(msg);
        toast({ title: 'Blocked', description: msg, tone: 'danger' });
        return;
      }
    }

    if (password && password.length > 0 && password.length < 8) {
      const msg = 'Password must be at least 8 characters.';
      setFormError(msg);
      toast({ title: 'Validation error', description: msg, tone: 'danger' });
      return;
    }

    const patch: Partial<{ role: string; is_active: boolean; password: string }> = {};
    const normalizedRole = role.toLowerCase();
    if (normalizedRole !== (data.role ?? '').toLowerCase()) {
      patch.role = normalizedRole;
    }
    if (isActive !== !!data.is_active) {
      patch.is_active = isActive;
    }
    if (password) {
      patch.password = password;
    }

    if (Object.keys(patch).length === 0) {
      setFormError('No changes to save.');
      return;
    }

    setSaving(true);
    try {
      const updated = await users.update(id, patch);
      setData(updated);
      setRole((updated.role ?? 'user').toLowerCase());
      setIsActive(!!updated.is_active);
      setPassword('');
      const msg = password ? 'User updated — password reset revokes sessions.' : 'User updated.';
      setFormSuccess(msg);
      toast({ title: 'Saved', description: msg, tone: 'success' });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setFormError(msg);
      toast({ title: 'Update failed', description: msg, tone: 'danger' });
    } finally {
      setSaving(false);
    }
  }, [id, data, isSelf, role, isActive, password, toast]);

  const handleDelete = useCallback(async () => {
    if (!id || !data) return;
    if (isSelf) {
      const msg = 'Cannot delete your own account.';
      toast({ title: 'Blocked', description: msg, tone: 'danger' });
      setDeleteOpen(false);
      return;
    }
    setDeleting(true);
    try {
      await users.del(id);
      toast({ title: 'Deleted', description: `User "${data.username}" deleted.`, tone: 'success' });
      navigate('/admin/users');
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast({ title: 'Delete failed', description: msg, tone: 'danger' });
    } finally {
      setDeleting(false);
      setDeleteOpen(false);
    }
  }, [id, data, isSelf, navigate, toast]);

  if (!authLoading && !isAdmin) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">User detail</h1>
        <Card>
          <div className="p-6">
            <Alert tone="danger" title="Forbidden">
              Admin access required. Your account does not have administrator privileges.
            </Alert>
          </div>
        </Card>
      </div>
    );
  }

  if (loading || authLoading) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">User detail</h1>
        <Card>
          <div className="p-6 text-sm text-zinc-400">Loading user…</div>
        </Card>
      </div>
    );
  }

  if (loadError) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <Link to="/admin/users" className="text-sm text-primary hover:underline">
            ← Users
          </Link>
          <h1 className="text-xl font-semibold tracking-tight">User detail</h1>
        </div>
        <Alert tone="danger" title="Failed to load user">
          {loadError}
        </Alert>
        <Button variant="secondary" onClick={() => void load()}>
          Retry
        </Button>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">User detail</h1>
        <Alert tone="warning" title="Not found">
          User not found.
        </Alert>
        <Link to="/admin/users" className="text-sm text-primary hover:underline">
          ← Back to users
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <Link to="/admin/users" className="text-sm text-primary hover:underline">
          ← Users
        </Link>
        <h1 className="text-xl font-semibold tracking-tight">User detail</h1>
        <span className="rounded-full bg-zinc-800 px-2 py-0.5 text-xs text-zinc-200">{data.id.slice(0, 8)}</span>
      </div>

      {isSelf && (
        <Alert tone="warning" title="Own account">
          You are viewing your own account — role and active status changes are disabled to prevent self-demotion.
        </Alert>
      )}

      {formSuccess && (
        <Alert tone="success" title="Success" dismissible onDismiss={() => setFormSuccess(null)}>
          {formSuccess}
        </Alert>
      )}

      {formError && (
        <Alert tone="danger" title="Error">
          {formError}
        </Alert>
      )}

      <Card header="Profile">
        <div className="grid grid-cols-1 gap-3 p-4 text-sm md:grid-cols-2">
          <div>
            <div className="text-xs text-zinc-400">ID</div>
            <div className="font-mono text-xs break-all">{data.id}</div>
          </div>
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
          <div>
            <div className="text-xs text-zinc-400">Active</div>
            <div>{data.is_active ? 'Yes' : 'No'}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-400">Created</div>
            <div>{formatDate(data.created_at)}</div>
          </div>
          <div>
            <div className="text-xs text-zinc-400">Updated</div>
            <div>{formatDate(data.updated_at)}</div>
          </div>
        </div>
      </Card>

      <Card header="Edit user">
        <div className="space-y-4 p-4">
          <Field label="Role" htmlFor="user-role" required>
            <Select
              id="user-role"
              value={role}
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setRole(e.target.value)}
              disabled={!!isSelf || saving}
              options={[
                { value: 'user', label: 'user' },
                { value: 'admin', label: 'admin' },
              ]}
            />
          </Field>

          <Field
            label="Active"
            htmlFor="user-active"
            hint={isSelf ? 'Deactivating your own account is blocked.' : 'Toggle to activate or deactivate the account.'}
          >
            <div className="flex items-center gap-2 py-1">
              <Switch
                id="user-active"
                checked={isActive}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setIsActive(e.target.checked)}
                disabled={!!isSelf || saving}
                aria-label="Active toggle"
              />
              <span className="text-sm text-zinc-300">{isActive ? 'Active' : 'Inactive'}</span>
            </div>
          </Field>

          <Field
            label="New password"
            htmlFor="user-password"
            hint="Leave blank to keep current. Resetting password revokes all of the user's web sessions."
            error={password && password.length > 0 && password.length < 8 ? 'Password must be at least 8 characters.' : undefined}
          >
            <Password
              id="user-password"
              value={password}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPassword(e.target.value)}
              placeholder="leave blank to keep current"
              disabled={saving}
              autoComplete="new-password"
            />
          </Field>

          <Alert tone="info" title="Password reset">
            Resetting the password will revoke all web sessions for this user.
          </Alert>

          <div className="flex flex-wrap gap-2">
            <Button variant="primary" onClick={() => void handleSave()} disabled={saving}>
              {saving ? 'Saving…' : 'Save changes'}
            </Button>
            <Button variant="secondary" onClick={() => void load()} disabled={saving}>
              Reset
            </Button>
            <Button variant="danger" onClick={() => setDeleteOpen(true)} disabled={!!isSelf || saving}>
              Delete user
            </Button>
          </div>
        </div>
      </Card>

      <Dialog
        open={deleteOpen}
        onClose={() => setDeleteOpen(false)}
        title="Delete user"
        description={`Delete "${data.username}"? This cannot be undone.`}
        size="sm"
        footer={
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setDeleteOpen(false)} disabled={deleting}>
              Cancel
            </Button>
            <Button variant="danger" onClick={() => void handleDelete()} disabled={deleting}>
              {deleting ? 'Deleting…' : 'Delete'}
            </Button>
          </div>
        }
      >
        <Alert tone="danger" title="Confirm deletion">
          This will permanently delete the user <span className="font-medium">{data.username}</span> ({data.id}).
        </Alert>
      </Dialog>
    </div>
  );
}
