import { Link } from 'react-router-dom';
import { Button, Icon } from '@devstroop/react-uikit';

export default function Home() {
  return (
    <div className="mx-auto w-full max-w-7xl px-4 sm:px-6 lg:px-8">
      <div className="grid gap-8 py-12 md:grid-cols-2 md:gap-12 md:py-16">
        <div className="text-left">
          <div className="mb-3 inline-flex items-center gap-2 rounded-full border bg-white px-3 py-1 text-xs text-zinc-600">
            <span className="h-2 w-2 rounded-full bg-emerald-500" />
            API v0.6.0 • WhatsApp Web automation
          </div>
          <h1 className="text-4xl font-bold tracking-tight md:text-5xl">WAS — WhatsApp Server</h1>
          <p className="mt-3 max-w-xl text-base text-zinc-600 md:text-lg">Send WhatsApp via simple REST API — instances, QR/phone link, rate-limited send. Lean React admin, no bloat.</p>
          <div className="mt-6 flex flex-wrap gap-3">
            <Link to="/auth/login">
              <Button variant="primary" size="lg">
                Sign in
              </Button>
            </Link>
            <Link to="/auth/register">
              <Button variant="secondary" size="lg">
                Create account
              </Button>
            </Link>
            <a href="/api-docs/" target="_blank">
              <Button variant="ghost" size="lg">
                <Icon name="file" size={16} />
                API Docs
              </Button>
            </a>
          </div>
        </div>
        <div className="rounded-xl border bg-white p-4 shadow-sm">
          <div className="flex items-center justify-between border-b pb-3">
            <div className="text-sm font-medium">Instance: WAS-01</div>
            <span className="rounded-full bg-emerald-100 px-2 py-0.5 text-xs text-emerald-700">active</span>
          </div>
          <div className="mt-4 flex flex-col gap-4 sm:flex-row">
            <div className="flex h-32 w-full items-center justify-center rounded-lg border border-dashed bg-zinc-50 text-center text-xs leading-tight text-zinc-500 sm:w-32 sm:shrink-0">QR<br />2s poll</div>
            <div className="min-w-0 flex-1 space-y-2">
              <div className="text-xs font-medium">POST /send</div>
              <pre className="overflow-x-auto rounded bg-zinc-900 p-2 text-[11px] leading-relaxed text-zinc-100">curl -X POST /api/v1/instances/abc/send?phone=+1555...&text=Hello -F file=@img.jpg</pre>
              <div className="text-xs text-zinc-500">authorized • polls every 5s</div>
            </div>
          </div>
        </div>
      </div>
      <div className="mt-4 grid grid-cols-1 gap-4 md:grid-cols-3">
        <div className="rounded-lg border bg-white p-4">
          <div className="text-sm font-medium">Multi-instance</div>
          <div className="mt-1 text-xs text-zinc-500">Run 1..N WhatsApp accounts, isolated sessions</div>
        </div>
        <div className="rounded-lg border bg-white p-4">
          <div className="text-sm font-medium">QR & Phone link</div>
          <div className="mt-1 text-xs text-zinc-500">2s QR poll, phone pairing, status 5s</div>
        </div>
        <div className="rounded-lg border bg-white p-4">
          <div className="text-sm font-medium">Send API</div>
          <div className="mt-1 text-xs text-zinc-500">Text + media via POST /send, rate-limited</div>
        </div>
      </div>
    </div>
  );
}
