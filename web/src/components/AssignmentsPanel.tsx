import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Alert,
  Badge,
  Button,
  Card,
  DataGrid,
  Dialog,
  Field,
  Select,
  Selectbar,
} from '@devstroop/react-uikit';
import type { GridColumn } from '@devstroop/react-uikit';
import { instances, users } from '../api/endpoints';
import type { InstanceInfo, InstanceOwnerRecord, InstancePermission } from '../api/types';
import { useAuth } from '../hooks/useAuth';

type Perm = InstancePermission;

const PERM_OPTIONS = [
  { value: 'viewer', label: 'Viewer' },
  { value: 'operator', label: 'Operator' },
  { value: 'owner', label: 'Owner' },
] as const;

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

function permTone(p: string): 'neutral' | 'primary' | 'success' {
  const v = p.toLowerCase();
  if (v === 'owner') return 'success';
  if (v === 'operator') return 'primary';
  return 'neutral';
}

export function AssignmentsPanel({ userId }: { userId: string }) {
  const { user: authUser, loading: authLoading } = useAuth();
  const isAdmin = (authUser?.role ?? '').toLowerCase() === 'admin';

  const [assignments, setAssignments] = useState<InstanceOwnerRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [allInstances, setAllInstances] = useState<InstanceInfo[]>([]);
  const [instancesLoading, setInstancesLoading] = useState(true);
  const [instancesError, setInstancesError] = useState<string | null>(null);

  const [selectedInstance, setSelectedInstance] = useState<string>('');
  const [permission, setPermission] = useState<Perm>('viewer');
  const [assigning, setAssigning] = useState(false);
  const [assignError, setAssignError] = useState<string | null>(null);
  const [assignSuccess, setAssignSuccess] = useState<string | null>(null);

  const [deleteTarget, setDeleteTarget] = useState<InstanceOwnerRecord | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [removeError, setRemoveError] = useState<string | null>(null);

  const loadAssignments = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const res = await users.instances(userId);
      setAssignments(res.instances ?? []);
    } catch (e) {
      setLoadError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [userId]);

  const loadInstances = useCallback(async () => {
    setInstancesLoading(true);
    setInstancesError(null);
    try {
      const res = await instances.list();
      setAllInstances(res.instances ?? []);
    } catch (e) {
      setInstancesError(e instanceof Error ? e.message : String(e));
    } finally {
      setInstancesLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadAssignments();
    void loadInstances();
  }, [loadAssignments, loadInstances]);

  const instanceMap = useMemo(() => {
    const m = new Map<string, InstanceInfo>();
    for (const inst of allInstances) m.set(inst.id, inst);
    return m;
  }, [allInstances]);

  const availableInstances = useMemo(() => {
    const assignedIds = new Set(assignments.map((a) => a.instance_id));
    return allInstances.filter((inst) => !assignedIds.has(inst.id));
  }, [allInstances, assignments]);

  const selectOptions = useMemo(() => {
    if (availableInstances.length === 0) {
      return [{ value: '', label: 'No available instances' }];
    }
    return [
      { value: '', label: 'Select instance…' },
      ...availableInstances.map((inst) => ({
        value: inst.id,
        label: `${inst.name} (${inst.id.slice(0, 8)})`,
      })),
    ];
  }, [availableInstances]);

  const handleAssign = useCallback(async () => {
    setAssignError(null);
    setAssignSuccess(null);
    setRemoveError(null);
    if (!selectedInstance) {
      setAssignError('Select an instance to assign.');
      return;
    }
    if (!PERM_OPTIONS.some((o) => o.value === permission)) {
      setAssignError('Invalid permission.');
      return;
    }
    setAssigning(true);
    try {
      const rec = await users.assign({
        user_id: userId,
        instance_id: selectedInstance,
        permission,
      });
      setAssignments((prev) => {
        const exists = prev.find((p) => p.instance_id === rec.instance_id);
        if (exists) {
          return prev.map((p) => (p.instance_id === rec.instance_id ? rec : p));
        }
        return [...prev, rec];
      });
      setSelectedInstance('');
      setAssignSuccess(`Assigned "${instanceMap.get(rec.instance_id)?.name ?? rec.instance_id}" as ${rec.permission}.`);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setAssignError(msg);
    } finally {
      setAssigning(false);
    }
  }, [userId, selectedInstance, permission, instanceMap]);

  const handleRemove = useCallback(async () => {
    if (!deleteTarget) return;
    setDeleting(true);
    setRemoveError(null);
    try {
      await users.removeInstance(userId, deleteTarget.instance_id);
      setAssignments((prev) => prev.filter((p) => p.instance_id !== deleteTarget.instance_id));
      setDeleteTarget(null);
      setAssignSuccess(`Removed instance ${deleteTarget.instance_id.slice(0, 8)}.`);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setRemoveError(msg);
    } finally {
      setDeleting(false);
    }
  }, [userId, deleteTarget]);

  const columns: GridColumn<InstanceOwnerRecord>[] = useMemo(
    () => [
      {
        property: 'instance_id',
        title: 'Instance',
        sortable: true,
        render: (row: InstanceOwnerRecord) => {
          const inst = instanceMap.get(row.instance_id);
          const name = inst?.name ?? '—';
          const short = row.instance_id.slice(0, 8);
          return (
            <div className="flex flex-col">
              <span className="font-medium">{name}</span>
              <span className="font-mono text-xs text-zinc-500">{short} · {row.instance_id}</span>
            </div>
          );
        },
      },
      {
        property: 'permission',
        title: 'Permission',
        sortable: true,
        render: (row: InstanceOwnerRecord) => (
          <Badge tone={permTone(String(row.permission))} variant="soft">
            {String(row.permission).charAt(0).toUpperCase() + String(row.permission).slice(1).toLowerCase()}
          </Badge>
        ),
      },
      {
        property: 'created_at',
        title: 'Assigned',
        sortable: true,
        render: (row: InstanceOwnerRecord) => <span className="text-xs text-zinc-600">{formatDate(row.created_at)}</span>,
      },
      {
        property: '__actions',
        title: 'Actions',
        render: (row: InstanceOwnerRecord) => (
          <Button
            variant="danger"
            size="sm"
            onClick={(e: React.MouseEvent) => {
              e.stopPropagation();
              setDeleteTarget(row);
              setRemoveError(null);
            }}
            disabled={deleting && deleteTarget?.instance_id === row.instance_id}
          >
            Remove
          </Button>
        ),
      },
    ],
    [instanceMap, deleting, deleteTarget],
  );

  if (!authLoading && !isAdmin) {
    return (
      <Card>
        <div className="p-6">
          <Alert tone="danger" title="Forbidden">
            Admin access required. Your account does not have administrator privileges.
          </Alert>
        </div>
      </Card>
    );
  }

  return (
    <>
      <Card header="Instance assignments">
        <div className="space-y-4 p-4">
          {loadError && (
            <Alert tone="danger" title="Failed to load assignments">
              {loadError}
            </Alert>
          )}
          {instancesError && (
            <Alert tone="warning" title="Failed to load instances">
              {instancesError}
            </Alert>
          )}
          {removeError && (
            <Alert tone="danger" title="Remove failed">
              {removeError}
            </Alert>
          )}
          {assignSuccess && (
            <Alert tone="success" title="Success" dismissible onDismiss={() => setAssignSuccess(null)}>
              {assignSuccess}
            </Alert>
          )}

          <div className="flex flex-wrap items-end gap-3 rounded border bg-zinc-50 p-3">
            <Field label="Instance" htmlFor="assign-instance" hint={availableInstances.length === 0 ? 'All instances are assigned or none exist.' : `${availableInstances.length} available`}>
              <Select
                id="assign-instance"
                value={selectedInstance}
                onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setSelectedInstance(e.target.value)}
                disabled={assigning || instancesLoading || availableInstances.length === 0}
                options={selectOptions}
              />
            </Field>

            <Field label="Permission" htmlFor="assign-permission" hint="Viewer can read, Operator can send, Owner full control">
              <Selectbar
                options={[...PERM_OPTIONS]}
                value={permission}
                onChange={(v: string) => setPermission(v as Perm)}
                aria-label="Permission"
              />
            </Field>

            <Button
              variant="primary"
              onClick={() => void handleAssign()}
              disabled={assigning || !selectedInstance || availableInstances.length === 0}
            >
              {assigning ? 'Assigning…' : 'Assign'}
            </Button>

            <Button variant="secondary" onClick={() => void loadAssignments()} disabled={loading || assigning}>
              Refresh
            </Button>
          </div>

          {assignError && (
            <Alert tone="danger" title="Assign failed">
              {assignError}
            </Alert>
          )}

          <DataGrid
            columns={columns}
            rows={assignments}
            rowKey={(row: InstanceOwnerRecord) => row.instance_id}
            isLoading={loading || authLoading}
            empty={loading ? 'Loading assignments…' : 'No instances assigned.'}
            ariaLabel="Instance assignments"
          />

          {!loading && assignments.length === 0 && !loadError && (
            <div className="text-center text-sm text-zinc-500">No assignments yet — assign an instance above.</div>
          )}

          <div className="flex justify-between text-xs text-zinc-500">
            <span>
              {assignments.length} assigned · {allInstances.length} total instances
            </span>
            <span className="hidden md:inline">Admin-only · permission enforced via RBAC</span>
          </div>
        </div>
      </Card>

      <Dialog
        open={!!deleteTarget}
        onClose={() => {
          if (!deleting) setDeleteTarget(null);
        }}
        title="Remove assignment"
        description={
          deleteTarget
            ? `Remove instance "${instanceMap.get(deleteTarget.instance_id)?.name ?? deleteTarget.instance_id.slice(0, 8)}" from this user?`
            : undefined
        }
        size="sm"
        footer={
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setDeleteTarget(null)} disabled={deleting}>
              Cancel
            </Button>
            <Button variant="danger" onClick={() => void handleRemove()} disabled={deleting}>
              {deleting ? 'Removing…' : 'Remove'}
            </Button>
          </div>
        }
      >
        {deleteTarget && (
          <Alert tone="danger" title="Confirm removal">
            This will revoke access to instance <span className="font-medium font-mono">{deleteTarget.instance_id}</span> with permission{' '}
            <span className="font-medium">{String(deleteTarget.permission)}</span>. User will lose access immediately.
          </Alert>
        )}
        {removeError && (
          <Alert tone="danger" title="Error">
            {removeError}
          </Alert>
        )}
      </Dialog>
    </>
  );
}
