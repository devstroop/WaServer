import { useCallback, useEffect, useRef, useState } from 'react';
import { Alert, Button, Card } from '@devstroop/react-uikit';
import { instances } from '../api/endpoints';

export type LinkPanelProps = {
  instanceId: string;
  authorized: boolean;
  onStatusRefresh?: () => void;
};

export function LinkPanel({ instanceId, authorized, onStatusRefresh }: LinkPanelProps) {
  const [qrUrl, setQrUrl] = useState<string | null>(null);
  const [qrError, setQrError] = useState<string | null>(null);
  const [linkingCode, setLinkingCode] = useState<string | null>(null);
  const [phoneError, setPhoneError] = useState<string | null>(null);
  const [phoneLoading, setPhoneLoading] = useState(false);
  const [unlinkError, setUnlinkError] = useState<string | null>(null);
  const [unlinkSuccess, setUnlinkSuccess] = useState<string | null>(null);
  const [unlinkLoading, setUnlinkLoading] = useState(false);

  const qrUrlRef = useRef<string | null>(null);
  useEffect(() => {
    qrUrlRef.current = qrUrl;
  }, [qrUrl]);

  const fetchQr = useCallback(async () => {
    if (!instanceId || authorized) return;
    try {
      const blob = await instances.qr(instanceId);
      if (blob instanceof Blob) {
        const url = URL.createObjectURL(blob);
        setQrUrl((prev) => {
          if (prev) URL.revokeObjectURL(prev);
          return url;
        });
        setQrError(null);
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes('already_authorized') || msg.toLowerCase().includes('authorized')) {
        setQrError(null);
        return;
      }
      setQrError(msg);
    }
  }, [instanceId, authorized]);

  useEffect(() => {
    if (authorized) {
      if (qrUrlRef.current) {
        URL.revokeObjectURL(qrUrlRef.current);
        setQrUrl(null);
      }
      return;
    }
    void fetchQr();
    const t = window.setInterval(() => void fetchQr(), 2000);
    return () => window.clearInterval(t);
  }, [fetchQr, authorized]);

  useEffect(() => {
    return () => {
      if (qrUrlRef.current) URL.revokeObjectURL(qrUrlRef.current);
    };
  }, []);

  const handleLinkPhone = async () => {
    setPhoneError(null);
    setLinkingCode(null);
    setPhoneLoading(true);
    try {
      const res = await instances.linkPhone(instanceId);
      const raw = res as unknown as Record<string, unknown>;
      const code =
        (raw['linking_code'] as string | null | undefined) ??
        (raw['code'] as string | null | undefined) ??
        null;
      if (code) setLinkingCode(String(code));
      else setPhoneError('No linking code returned');
    } catch (e) {
      setPhoneError(e instanceof Error ? e.message : String(e));
    } finally {
      setPhoneLoading(false);
    }
  };

  const handleUnlink = async () => {
    setUnlinkError(null);
    setUnlinkSuccess(null);
    setUnlinkLoading(true);
    try {
      const res = await instances.unlink(instanceId);
      setUnlinkSuccess((res as { message?: string }).message ?? 'Unlinked');
      onStatusRefresh?.();
      setTimeout(() => void fetchQr(), 300);
    } catch (e) {
      setUnlinkError(e instanceof Error ? e.message : String(e));
    } finally {
      setUnlinkLoading(false);
    }
  };

  if (authorized) {
    return (
      <Card header={<span className="font-medium">Link device</span>}>
        <div className="space-y-3">
          <Alert tone="success" title="Linked">
            This device is authorized.
          </Alert>
          {unlinkSuccess && <Alert tone="success">{unlinkSuccess}</Alert>}
          {unlinkError && <Alert tone="danger">{unlinkError}</Alert>}
          <Button variant="danger" onClick={handleUnlink} disabled={unlinkLoading}>
            {unlinkLoading ? 'Unlinking…' : 'Unlink'}
          </Button>
        </div>
      </Card>
    );
  }

  return (
    <Card header={<span className="font-medium">Link device</span>}>
      <div className="space-y-3">
        {qrError && <Alert tone="danger">{qrError}</Alert>}
        {phoneError && <Alert tone="danger">{phoneError}</Alert>}
        {unlinkError && <Alert tone="danger">{unlinkError}</Alert>}
        {linkingCode && (
          <Alert tone="info" title="Linking code">
            {linkingCode}
          </Alert>
        )}
        <div className="flex flex-wrap items-start gap-6">
          <div className="flex h-[200px] w-[200px] items-center justify-center rounded border bg-zinc-900">
            {qrUrl ? (
              <img
                src={qrUrl}
                alt="WhatsApp link QR code"
                className="h-[200px] w-[200px] rounded"
              />
            ) : (
              <span className="px-4 text-center text-sm text-zinc-400">Loading QR… poll 2s</span>
            )}
          </div>
          <div className="max-w-sm space-y-2">
            <p className="text-sm text-zinc-300">
              Open WhatsApp on the phone → Settings → Linked devices → Link a device, then scan
              this code. This panel refreshes automatically.
            </p>
            <div className="flex flex-wrap gap-2">
              <Button variant="secondary" onClick={() => void fetchQr()}>
                Refresh QR
              </Button>
              <Button variant="primary" onClick={handleLinkPhone} disabled={phoneLoading}>
                {phoneLoading ? 'Linking…' : 'Link via Phone'}
              </Button>
              <Button variant="ghost" onClick={handleUnlink} disabled={unlinkLoading}>
                {unlinkLoading ? 'Unlinking…' : 'Unlink'}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </Card>
  );
}

export default LinkPanel;
