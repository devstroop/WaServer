import { useEffect, useState } from 'react';
import { Alert, Button, Card, DropZone, Field, Input, Textarea, Upload, useToast } from '@devstroop/react-uikit';
import { instances } from '../api/endpoints';
import { useAuth } from '../hooks/useAuth';

export type SendPanelProps = {
  instanceId: string;
  permission?: string;
};

const E164_RE = /^\+[1-9]\d{1,14}$/;

export function SendPanel({ instanceId, permission }: SendPanelProps) {
  const { user } = useAuth();
  const { toast } = useToast();

  const [phone, setPhone] = useState('');
  const [text, setText] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [phoneError, setPhoneError] = useState<string | null>(null);
  const [textError, setTextError] = useState<string | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Sync file from Upload component's hidden input (UIKit Upload uses data-testid="upload-input")
  useEffect(() => {
    const el = document.querySelector('[data-testid="upload-input"]') as HTMLInputElement | null;
    if (!el) return;
    const handler = () => {
      if (el.files && el.files[0]) {
        setFile(el.files[0]);
      }
    };
    el.addEventListener('change', handler);
    return () => el.removeEventListener('change', handler);
  }, []);

  const handleDrop = (files: FileList) => {
    if (files.length > 0) {
      setFile(files[0]);
    }
  };

  // Permission gate: Viewer hidden, only Operator|Owner (and admin) can send
  const effectivePermission = (permission ?? user?.role ?? '').toLowerCase();
  const isViewer = effectivePermission === 'viewer';
  if (isViewer) {
    return null;
  }
  if (permission) {
    const p = permission.toLowerCase();
    if (p !== 'owner' && p !== 'operator' && p !== 'admin') {
      return null;
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setPhoneError(null);
    setTextError(null);
    setSubmitError(null);
    setSuccess(null);

    let hasError = false;
    const trimmedPhone = phone.trim();
    if (!E164_RE.test(trimmedPhone)) {
      setPhoneError('Phone must be E.164 format, e.g. +15551234567');
      hasError = true;
    }
    const trimmedText = text.trim();
    if (!trimmedText && !file) {
      setTextError('Provide text or attach a file');
      hasError = true;
    }
    if (hasError) return;

    setLoading(true);
    try {
      const res = await instances.send(
        instanceId,
        { phone: trimmedPhone, text: trimmedText || undefined },
        file ?? undefined,
      );
      const msg = `Sent to ${res.phone} · ${res.message_id.slice(0, 8)}`;
      setSuccess(msg);
      toast({ title: 'Message sent', description: msg, tone: 'success' });
    } catch (err) {
      const m = err instanceof Error ? err.message : String(err);
      setSubmitError(m);
      toast({ title: 'Send failed', description: m, tone: 'danger' });
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card header={<span className="font-medium">Send message</span>}>
      <div className="space-y-3">
        {success && (
          <Alert tone="success" title="Sent">
            {success}
          </Alert>
        )}
        {submitError && <Alert tone="danger">{submitError}</Alert>}

        <form onSubmit={handleSubmit} className="space-y-4">
          <Field label="Phone" htmlFor="send-phone" required hint="E.164 format" error={phoneError}>
            <Input
              id="send-phone"
              placeholder="+15551234567"
              value={phone}
              onChange={(ev: React.ChangeEvent<HTMLInputElement>) => setPhone(ev.target.value)}
              pattern="\\+[1-9][0-9]{1,14}"
              required
              invalid={!!phoneError}
            />
          </Field>

          <Field label="Message" htmlFor="send-text" hint="Text or caption (at least one of text/file required)" error={textError}>
            <Textarea
              id="send-text"
              placeholder="Hello..."
              value={text}
              onChange={(ev: React.ChangeEvent<HTMLTextAreaElement>) => setText(ev.target.value)}
              rows={3}
            />
          </Field>

          <Field label="Attachment" hint="Optional file via upload or drag & drop">
            <div className="space-y-2">
              <DropZone
                onDrop={handleDrop}
                label="Drop file here or browse"
                dragLabel="Drop to attach"
                browseText="Browse"
              />
              {file && (
                <div className="flex items-center gap-2 text-sm">
                  <span className="truncate font-medium">{file.name}</span>
                  <span className="text-zinc-400">{Math.max(1, Math.round(file.size / 1024))} KB</span>
                  <Button variant="ghost" size="sm" type="button" onClick={() => setFile(null)}>
                    Remove
                  </Button>
                </div>
              )}
              {/* UIKit Upload for compliance — syncs via effect above */}
              <div className="pt-1">
                <Upload multiple={false} auto={false} chooseText="Choose file" />
              </div>
            </div>
          </Field>

          <Button type="submit" variant="primary" disabled={loading}>
            {loading ? 'Sending…' : 'Send'}
          </Button>
        </form>
      </div>
    </Card>
  );
}

export default SendPanel;
