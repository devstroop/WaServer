import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Alert,
  Badge,
  Button,
  Card,
  DataFilter,
  DataGrid,
  Dialog,
  Field,
  Input,
  Password,
  Select,
} from '@devstroop/react-uikit';
import type { GridColumn, DataFilterProperty } from '@devstroop/react-uikit';
import { required, minLength, runValidators } from '@devstroop/react-uikit';
import { users } from '../api/endpoints';
import type { UserInfo } from '../api/types';
import { useAuth } from '../hooks/useAuth';

function formatDate(value: string): string {
  try {
    const d = new Date(value);
    if (Number.isNaN(d.getTime())) return value;
    return d.toLocaleString();
  } catch {
    return value;
  }
}

function roleTone(role: string): 'primary' | 'neutral' | 'success' | 'danger' | 'warning' {
  const r = role.toLowerCase();
  if (r === 'admin') return 'primary';
  return 'neutral';
}

export function UsersList() {
  const { user, loading: authLoading } = useAuth();
  const isAdmin = (user?.role ?? '').toLowerCase() === 'admin';

  const [list, setList] = useState<UserInfo[]>([]);
  const [filtered, setFiltered] = useState<UserInfo[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [dialogOpen, setDialogOpen] = useState(false);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [role, setRole] = useState('user');
  const [formError, setFormError] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<{ username?: string; password?: string }>({});
  const [creating, setCreating] = useState(false);

  const rows = filtered ?? list;

  const filterProperties = useMemo<readonly DataFilterProperty[]>(
    () => [
      { name: 'username', title: 'Username', type: 'string' },
      {
        name: 'role',
        title: 'Role',
        type: 'enum',
        values: [
          { value: 'admin', label: 'admin' },
          { value: 'user', label: 'user' },
        ],
      },
      { name: 'is_active', title: 'Active', type: 'boolean' },
      { name: 'created_at', title: 'Created', type: 'date' },
    ],
    [],
  );

  const columns = useMemo<GridColumn<UserInfo>[]>(
    () => [
      {
        property: 'username',
        title: 'Username',
        sortable: true,
        render: (row) => <span className="font-medium">{row.username}</span>,
      },
      {
        property: 'role',
        title: 'Role',
        sortable: true,
        render: (row) => (
          <Badge tone={roleTone(row.role)} variant="soft">
            {row.role}
          </Badge>
        ),
      },
      {
        property: 'is_active',
        title: 'Active',
        type: 'boolean',
        render: (row) => (
          <Badge tone={row.is_active ? 'success' : 'neutral'} variant="soft">
            {row.is_active ? 'active' : 'inactive'}
          </Badge>
        ),
      },
      {
        property: 'created_at',
        title: 'Created',
        sortable: true,
        type: 'date',
        render: (row) => <span className="text-zinc-600">{formatDate(row.created_at)}</span>,
      },
    ],
    [],
  );

  const load = useCallback(async () => {
    try {
      setError(null);
      const res = await users.list();
      setList(res.users ?? []);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const resetForm = useCallback(() => {
    setUsername('');
    setPassword('');
    setRole('user');
    setFormError(null);
    setFieldErrors({});
  }, []);

  const handleOpen = useCallback(() => {
    resetForm();
    setDialogOpen(true);
  }, [resetForm]);

  const handleClose = useCallback(() => {
    if (creating) return;
    setDialogOpen(false);
    resetForm();
  }, [creating, resetForm]);

  const handleCreate = useCallback(async () => {
    setFormError(null);
    setSuccess(null);

    const trimmedUsername = username.trim();

    const usernameValidators = [required('Username is required')];
    const passwordValidators = [required('Password is required'), minLength(8, 'Password must be at least 8 characters')];

    const uErrs = runValidators(usernameValidators, trimmedUsername);
    const pErrs = runValidators(passwordValidators, password);

    const nextFieldErrors: { username?: string; password?: string } = {};
    if (uErrs.length > 0) nextFieldErrors.username = uErrs[0];
    if (pErrs.length > 0) nextFieldErrors.password = pErrs[0];

    if (Object.keys(nextFieldErrors).length > 0) {
      setFieldErrors(nextFieldErrors);
      return;
    }

    setFieldErrors({});
    setCreating(true);

    try {
      await users.create({ username: trimmedUsername, password, role });
      setSuccess(`User "${trimmedUsername}" created`);
      setDialogOpen(false);
      resetForm();
      await load();
    } catch (e) {
      const raw = e instanceof Error ? e.message : String(e);
      const lower = raw.toLowerCase();
      if (raw.includes('409') || lower.includes('already exists') || lower.includes('already taken') || lower.includes('conflict') || lower.includes('exists')) {
        setFormError('Username already exists (409) — choose another username.');
      } else {
        setFormError(raw || 'Create failed');
      }
    } finally {
      setCreating(false);
    }
  }, [username, password, role, load, resetForm]);

  if (!authLoading && !isAdmin) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-semibold tracking-tight">Users</h1>
        <Card>
          <div className="p-6 text-center">
            <Alert tone="danger" title="Forbidden">
              Admin access required. Your account does not have administrator privileges.
            </Alert>
            <p className="mt-3 text-sm text-zinc-500">Contact an administrator to request access.</p>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Users</h1>
          <p className="text-sm text-zinc-500">Admin — manage accounts via GET /api/v1/users</p>
        </div>
        <Button variant="primary" onClick={handleOpen}>
          Create user
        </Button>
      </div>

      {error && (
        <Alert tone="danger" title="Failed to load users">
          {error}
        </Alert>
      )}

      {success && (
        <Alert tone="success" title="Success" dismissible onDismiss={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      <Card>
        <div className="space-y-3 p-1">
          <DataFilter
            properties={filterProperties}
            items={list}
            viewChanged={(v) => setFiltered(v as UserInfo[])}
          />
          <DataGrid
            columns={columns}
            rows={rows}
            rowKey={(row) => row.id}
            allowSorting
            allowPaging
            pageSize={10}
            isLoading={loading}
            empty={loading ? 'Loading users…' : 'No users found.'}
            ariaLabel="Users"
          />
          {!loading && list.length > 0 && rows.length === 0 && (
            <div className="text-center text-sm text-zinc-500">No results match current filters.</div>
          )}
          <div className="flex justify-between text-xs text-zinc-500">
            <span>
              Showing {rows.length} of {list.length} users
            </span>
          </div>
        </div>
      </Card>

      <Dialog
        open={dialogOpen}
        onClose={handleClose}
        title="Create user"
        description="Create a new account. Username must be unique (409 on conflict)."
        size="md"
        footer={
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={handleClose} disabled={creating}>
              Cancel
            </Button>
            <Button variant="primary" onClick={() => void handleCreate()} disabled={creating}>
              {creating ? 'Creating…' : 'Create'}
            </Button>
          </div>
        }
      >
        <div className="space-y-3">
          {formError && (
            <Alert tone="danger" title="Create failed">
              {formError}
            </Alert>
          )}

          <Field label="Username" required htmlFor="user-username" error={fieldErrors.username}>
            <Input
              id="user-username"
              value={username}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setUsername(e.target.value)}
              placeholder="username"
              invalid={!!fieldErrors.username}
              autoComplete="username"
            />
          </Field>

          <Field label="Password" required htmlFor="user-password" error={fieldErrors.password} hint="Minimum 8 characters">
            <Password
              id="user-password"
              value={password}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPassword(e.target.value)}
              placeholder="••••••••"
              invalid={!!fieldErrors.password}
              autoComplete="new-password"
            />
          </Field>

          <Field label="Role" required htmlFor="user-role">
            <Select
              id="user-role"
              value={role}
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setRole(e.target.value)}
              options={[
                { value: 'user', label: 'user' },
                { value: 'admin', label: 'admin' },
              ]}
            />
          </Field>
        </div>
      </Dialog>
    </div>
  );
}

export default UsersList;
