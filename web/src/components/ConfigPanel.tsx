import { useCallback, useEffect, useState } from 'react';
import { Alert, Button, Card, Field, Input, Select, useToast } from '@devstroop/react-uikit';
import { instances } from '../api/endpoints';
import { useAuth } from '../hooks/useAuth';

export type ConfigPanelProps = {
  instanceId: string;
};

type InstanceBrowserConfig = {
  headless: boolean;
  timeout_ms: number;
  extra_args: string[];
};

type InstanceRateLimits = {
  messages_per_minute: number;
  requests_per_minute: number;
  message_cooldown_ms: number;
};

type InstanceConfig = {
  instance_id?: string;
  instance_name?: string;
  idle_timeout: number;
  browser: InstanceBrowserConfig;
  rate_limits: InstanceRateLimits;
};

type UpdateBrowserConfig = {
  headless?: boolean;
  timeout_ms?: number;
  extra_args?: string[];
};

type UpdateInstanceConfigRequest = {
  instance_name?: string;
  idle_timeout?: number;
  browser?: UpdateBrowserConfig;
  rate_limits?: Partial<InstanceRateLimits>;
};

export function ConfigPanel({ instanceId }: ConfigPanelProps) {
  const { user } = useAuth();
  const { toast } = useToast();

  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [config, setConfig] = useState<InstanceConfig | null>(null);

  const [instanceName, setInstanceName] = useState('');
  const [idleTimeout, setIdleTimeout] = useState('');
  const [headless, setHeadless] = useState<string>('true');
  const [timeoutMs, setTimeoutMs] = useState('');
  const [extraArgs, setExtraArgs] = useState('');

  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState<string | null>(null);
  const [restartRequired, setRestartRequired] = useState<boolean | null>(null);

  const [headlessError, setHeadlessError] = useState<string | null>(null);
  const [timeoutError, setTimeoutError] = useState<string | null>(null);
  const [idleError, setIdleError] = useState<string | null>(null);

  const role = (user?.role ?? '').toLowerCase();
  const isHidden = role === 'viewer' || role === 'operator';

  const applyConfig = useCallback((c: InstanceConfig) => {
    setConfig(c);
    setInstanceName(c.instance_name ?? '');
    setIdleTimeout(String(c.idle_timeout ?? 300));
    setHeadless(String(c.browser.headless));
    setTimeoutMs(String(c.browser.timeout_ms));
    setExtraArgs((c.browser.extra_args ?? []).join(', '));
  }, []);

  const fetchConfig = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const raw = await instances.getConfig(instanceId);
      const c = raw as unknown as InstanceConfig;
      // Normalize defaults if backend omits fields
      const normalized: InstanceConfig = {
        instance_id: c.instance_id,
        instance_name: c.instance_name,
        idle_timeout: c.idle_timeout ?? 300,
        browser: {
          headless: c.browser?.headless ?? true,
          timeout_ms: c.browser?.timeout_ms ?? 30000,
          extra_args: c.browser?.extra_args ?? [],
        },
        rate_limits: {
          messages_per_minute: c.rate_limits?.messages_per_minute ?? 60,
          requests_per_minute: c.rate_limits?.requests_per_minute ?? 120,
          message_cooldown_ms: c.rate_limits?.message_cooldown_ms ?? 1000,
        },
      };
      applyConfig(normalized);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setLoadError(msg);
    } finally {
      setLoading(false);
    }
  }, [instanceId, applyConfig]);

  useEffect(() => {
    void fetchConfig();
  }, [fetchConfig]);

  if (isHidden) return null;

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaveError(null);
    setSaveSuccess(null);
    setRestartRequired(null);
    setHeadlessError(null);
    setTimeoutError(null);
    setIdleError(null);

    let hasError = false;
    const parsedIdle = idleTimeout.trim() === '' ? undefined : Number(idleTimeout);
    if (idleTimeout.trim() !== '' && (!Number.isFinite(parsedIdle) || (parsedIdle as number) < 0)) {
      setIdleError('idle_timeout must be a non-negative integer');
      hasError = true;
    }
    const parsedTimeout = timeoutMs.trim() === '' ? undefined : Number(timeoutMs);
    if (timeoutMs.trim() !== '' && (!Number.isFinite(parsedTimeout) || (parsedTimeout as number) <= 0)) {
      setTimeoutError('timeout_ms must be a positive integer');
      hasError = true;
    }
    if (headless !== 'true' && headless !== 'false') {
      setHeadlessError('headless must be true or false');
      hasError = true;
    }
    if (hasError) return;

    const payload: UpdateInstanceConfigRequest = {
      instance_name: instanceName.trim() || undefined,
      idle_timeout: parsedIdle,
      browser: {
        headless: headless === 'true',
        timeout_ms: parsedTimeout,
        extra_args: extraArgs.trim() === '' ? [] : extraArgs.split(',').map((s) => s.trim()).filter(Boolean),
      },
    };

    setSaving(true);
    try {
      const raw = await instances.updateConfig(instanceId, payload);
      const res = raw as unknown as { message?: string; config?: InstanceConfig; restart_required?: boolean };
      const msg = res.message ?? 'Configuration updated';
      const next = res.config as InstanceConfig | undefined;
      if (next) applyConfig(next);
      else {
        // refetch to reflect persisted state
        void fetchConfig();
      }
      if (typeof res.restart_required === 'boolean') setRestartRequired(res.restart_required);
      setSaveSuccess(msg);
      toast({ title: 'Config updated', description: msg, tone: 'success' });
      if (res.restart_required) {
        toast({ title: 'Restart required', description: 'Browser restart required for changes to take effect', tone: 'warning' });
      }
    } catch (err) {
      const m = err instanceof Error ? err.message : String(err);
      setSaveError(m);
      toast({ title: 'Update failed', description: m, tone: 'danger' });
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card header={<span className="font-medium">Configuration</span>}>
      <div className="space-y-3">
        {loadError && <Alert tone="danger">{loadError}</Alert>}
        {saveError && <Alert tone="danger">{saveError}</Alert>}
        {saveSuccess && <Alert tone="success">{saveSuccess}</Alert>}
        {restartRequired !== null && (
          <Alert tone={restartRequired ? 'warning' : 'info'} title={restartRequired ? 'Restart required' : 'No restart needed'}>
            {restartRequired ? 'Browser restart required for these changes.' : 'Changes applied without restart.'}
          </Alert>
        )}

        {loading ? (
          <div className="text-sm text-zinc-400">Loading config…</div>
        ) : config ? (
          <form onSubmit={handleSave} className="space-y-4">
            <Field label="Instance name" htmlFor="cfg-name" hint="Friendly name">
              <Input
                id="cfg-name"
                placeholder="my-instance"
                value={instanceName}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => setInstanceName(ev.target.value)}
              />
            </Field>

            <Field label="Idle timeout (seconds)" htmlFor="cfg-idle" error={idleError} hint="Default 300">
              <Input
                id="cfg-idle"
                type="number"
                min={0}
                step={1}
                placeholder="300"
                value={idleTimeout}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => setIdleTimeout(ev.target.value)}
                invalid={!!idleError}
              />
            </Field>

            <Field label="Browser headless" htmlFor="cfg-headless" error={headlessError} hint="InstanceBrowserConfig.headless">
              <Select
                id="cfg-headless"
                value={headless}
                onChange={(ev: React.ChangeEvent<HTMLSelectElement>) => setHeadless(ev.target.value)}
                options={[
                  { value: 'true', label: 'true (headless)' },
                  { value: 'false', label: 'false (headed)' },
                ]}
                invalid={!!headlessError}
              />
            </Field>

            <Field label="Browser timeout (ms)" htmlFor="cfg-timeout" error={timeoutError} hint="InstanceBrowserConfig.timeout_ms">
              <Input
                id="cfg-timeout"
                type="number"
                min={1}
                step={1}
                placeholder="30000"
                value={timeoutMs}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => setTimeoutMs(ev.target.value)}
                invalid={!!timeoutError}
              />
            </Field>

            <Field label="Extra args" htmlFor="cfg-extra" hint="Comma-separated Chrome args">
              <Input
                id="cfg-extra"
                placeholder="--no-sandbox, --disable-gpu"
                value={extraArgs}
                onChange={(ev: React.ChangeEvent<HTMLInputElement>) => setExtraArgs(ev.target.value)}
              />
            </Field>

            <div className="flex flex-wrap gap-2">
              <Button type="submit" variant="primary" disabled={saving}>
                {saving ? 'Saving…' : 'Save config'}
              </Button>
              <Button type="button" variant="secondary" onClick={() => void fetchConfig()} disabled={loading || saving}>
                Reload
              </Button>
            </div>
            <div className="text-xs text-zinc-400">GET /instances/:id/config then PUT with InstanceBrowserConfig</div>
          </form>
        ) : (
          <div className="text-sm text-zinc-400">No config loaded.</div>
        )}
      </div>
    </Card>
  );
}

export default ConfigPanel;
