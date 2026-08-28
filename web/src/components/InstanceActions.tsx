import { useEffect, useRef, useState } from 'react';
import { Alert, Button, Card, Dialog, Field, Input, Select, useToast } from '@devstroop/react-uikit';
import { instances } from '../api/endpoints';
import { useAuth } from '../hooks/useAuth';

export type InstanceActionsProps = {
  instanceId: string;
  onDeleted?: () => void;
};

export function InstanceActions({ instanceId, onDeleted }: InstanceActionsProps) {
  const { user } = useAuth();
  const { toast } = useToast();

  const [warmupLoading, setWarmupLoading] = useState(false);
  const [warmupError, setWarmupError] = useState<string | null>(null);
  const [warmupSuccess, setWarmupSuccess] = useState<string | null>(null);

  const [screenshotLoading, setScreenshotLoading] = useState(false);
  const [screenshotError, setScreenshotError] = useState<string | null>(null);
  const [screenshotUrl, setScreenshotUrl] = useState<string | null>(null);
  const [screenshotOpen, setScreenshotOpen] = useState(false);

  const [resetLoading, setResetLoading] = useState(false);
  const [resetError, setResetError] = useState<string | null>(null);
  const [resetSuccess, setResetSuccess] = useState<string | null>(null);
  const [resetConfirmOpen, setResetConfirmOpen] = useState(false);

  const [deleteLoading, setDeleteLoading] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deleteSuccess, setDeleteSuccess] = useState<string | null>(null);
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [deleteData, setDeleteData] = useState<string>('false');
  const [deleteConfirmText, setDeleteConfirmText] = useState('');

  const screenshotRef = useRef<string | null>(null);

  useEffect(() => {
    screenshotRef.current = screenshotUrl;
  }, [screenshotUrl]);

  useEffect(() => {
    return () => {
      if (screenshotRef.current) URL.revokeObjectURL(screenshotRef.current);
    };
  }, []);

  const closeScreenshot = () => {
    setScreenshotOpen(false);
    if (screenshotUrl) {
      URL.revokeObjectURL(screenshotUrl);
      setScreenshotUrl(null);
    }
  };

  const role = (user?.role ?? '').toLowerCase();
  const isHidden = role === 'viewer' || role === 'operator';
  if (isHidden) return null;

  const handleWarmup = async () => {
    setWarmupLoading(true);
    setWarmupError(null);
    setWarmupSuccess(null);
    try {
      const res = await instances.warmup(instanceId);
      const msg = (res as { message?: string }).message ?? 'Instance warmed up';
      setWarmupSuccess(msg);
      toast({ title: 'Warmed up', description: msg, tone: 'success' });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setWarmupError(msg);
      toast({ title: 'Warmup failed', description: msg, tone: 'danger' });
    } finally {
      setWarmupLoading(false);
    }
  };

  const handleScreenshot = async () => {
    setScreenshotLoading(true);
    setScreenshotError(null);
    try {
      const blob = await instances.screenshot(instanceId);
      if (blob instanceof Blob) {
        if (screenshotUrl) URL.revokeObjectURL(screenshotUrl);
        const url = URL.createObjectURL(blob);
        setScreenshotUrl(url);
        setScreenshotOpen(true);
        toast({ title: 'Screenshot captured', tone: 'success' });
      } else {
        throw new Error('Invalid screenshot response');
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setScreenshotError(msg);
      toast({ title: 'Screenshot failed', description: msg, tone: 'danger' });
    } finally {
      setScreenshotLoading(false);
    }
  };

  const handleResetConfirm = async () => {
    setResetLoading(true);
    setResetError(null);
    setResetSuccess(null);
    try {
      const res = await instances.reset(instanceId);
      const msg = (res as { message?: string }).message ?? 'Instance reset';
      setResetSuccess(msg);
      toast({ title: 'Instance reset', description: msg, tone: 'success' });
      setResetConfirmOpen(false);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setResetError(msg);
      toast({ title: 'Reset failed', description: msg, tone: 'danger' });
    } finally {
      setResetLoading(false);
    }
  };

  const handleDeleteConfirm = async () => {
    setDeleteLoading(true);
    setDeleteError(null);
    setDeleteSuccess(null);
    try {
      const delData = deleteData === 'true';
      const res = await instances.del(instanceId, delData);
      const msg = (res as { message?: string }).message ?? 'Instance deleted';
      setDeleteSuccess(msg);
      toast({ title: 'Instance deleted', description: msg, tone: 'success' });
      setDeleteConfirmOpen(false);
      onDeleted?.();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setDeleteError(msg);
      toast({ title: 'Delete failed', description: msg, tone: 'danger' });
    } finally {
      setDeleteLoading(false);
    }
  };

  return (
    <>
      <Card header={<span className="font-medium">Instance actions</span>}>
        <div className="space-y-3">
          {warmupSuccess && <Alert tone="success">{warmupSuccess}</Alert>}
          {warmupError && <Alert tone="danger">{warmupError}</Alert>}
          {screenshotError && <Alert tone="danger">{screenshotError}</Alert>}
          {resetSuccess && <Alert tone="success">{resetSuccess}</Alert>}
          {resetError && <Alert tone="danger">{resetError}</Alert>}
          {deleteSuccess && <Alert tone="success">{deleteSuccess}</Alert>}
          {deleteError && <Alert tone="danger">{deleteError}</Alert>}

          <div className="flex flex-wrap gap-2">
            <Button variant="secondary" size="sm" onClick={() => void handleWarmup()} disabled={warmupLoading}>
              {warmupLoading ? 'Warming…' : 'Warmup'}
            </Button>
            <Button variant="secondary" size="sm" onClick={() => void handleScreenshot()} disabled={screenshotLoading}>
              {screenshotLoading ? 'Capturing…' : 'Screenshot'}
            </Button>
            <Button variant="danger" size="sm" onClick={() => setResetConfirmOpen(true)} disabled={resetLoading}>
              Reset
            </Button>
            <Button variant="danger" size="sm" onClick={() => setDeleteConfirmOpen(true)} disabled={deleteLoading}>
              Delete
            </Button>
          </div>
          <p className="text-xs text-zinc-500">
            Owner only — warmup pre-warms browser, screenshot opens a dialog, reset clears session, delete removes instance.
          </p>
        </div>
      </Card>

      <Dialog
        open={screenshotOpen}
        onClose={closeScreenshot}
        title="Screenshot"
        description="Live browser capture"
        size="lg"
        footer={
          <Button variant="secondary" onClick={closeScreenshot}>
            Close
          </Button>
        }
      >
        <div className="flex justify-center">
          {screenshotUrl ? (
            <img src={screenshotUrl} alt="Instance screenshot" className="max-h-[70vh] max-w-full rounded border" />
          ) : (
            <div className="text-sm text-zinc-500">No image</div>
          )}
        </div>
      </Dialog>

      <Dialog
        open={resetConfirmOpen}
        onClose={() => setResetConfirmOpen(false)}
        title="Reset instance"
        description="This will stop the browser and wipe session data. Instance is preserved."
        footer={
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => setResetConfirmOpen(false)} disabled={resetLoading}>
              Cancel
            </Button>
            <Button variant="danger" onClick={() => void handleResetConfirm()} disabled={resetLoading}>
              {resetLoading ? 'Resetting…' : 'Confirm reset'}
            </Button>
          </div>
        }
      >
        <Alert tone="warning">Reset clears Chrome profile, sessions and media.</Alert>
      </Dialog>

      <Dialog
        open={deleteConfirmOpen}
        onClose={() => setDeleteConfirmOpen(false)}
        title="Delete instance"
        description="Permanently delete this instance."
        footer={
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => setDeleteConfirmOpen(false)} disabled={deleteLoading}>
              Cancel
            </Button>
            <Button
              variant="danger"
              onClick={() => void handleDeleteConfirm()}
              disabled={deleteLoading || (deleteConfirmText.trim() !== '' && deleteConfirmText.trim().toLowerCase() !== 'delete')}
            >
              {deleteLoading ? 'Deleting…' : 'Confirm delete'}
            </Button>
          </div>
        }
      >
        <div className="space-y-3">
          <Alert tone="danger">This action cannot be undone.</Alert>
          <Field label="Delete data" hint="Remove instance data directory as well">
            <Select
              value={deleteData}
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setDeleteData(e.target.value)}
              options={[
                { value: 'false', label: 'Preserve data' },
                { value: 'true', label: 'Delete data' },
              ]}
            />
          </Field>
          <Field label="Confirm" hint="Type DELETE to confirm (optional)">
            <Input
              placeholder="DELETE"
              value={deleteConfirmText}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDeleteConfirmText(e.target.value)}
            />
          </Field>
          <div className="text-xs text-zinc-500">DELETE /instances/:id?delete_data={deleteData}</div>
        </div>
      </Dialog>
    </>
  );
}

export default InstanceActions;
