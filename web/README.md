# WAS Web — React admin

Vite + React 19 + TypeScript + Tailwind + React Router.

Backend is the `WaServer` Rust API (backend-only after htmx removal). Dev proxy forwards `/api` → `http://localhost:3000`.

## Dev

```bash
cd web
npm install
npm run dev # http://localhost:5173
```

Env:

- `VITE_API_BASE` — override API base (default empty → proxy). Example `VITE_API_BASE=http://localhost:3000`
- Defined at build time via `import.meta.env.VITE_API_BASE` in `src/api/client.ts` (`BASE = import.meta.env.VITE_API_BASE || ''` → `fetch(BASE + path)`). No trailing slash required. Empty means relative `/api/*` which dev proxy forwards.

See `.env.example`.

## Routes

| Path | Component | Access | Notes |
|------|-----------|--------|-------|
| `/` | `Home` | public | Landing; links to login/register, API docs |
| `/login` | `Login` | public | `POST /api/v1/auth/login` → stores `was_token` |
| `/register` | `Register` | public | `POST /api/v1/auth/register` (first user → admin) |
| `/app` | `Dashboard` | protected | Health + instances overview, polls 5s (`GET /api/health`, `GET /api/v1/instances`) |
| `/app/instances` | `Instances` | protected | List + create (`GET/POST /api/v1/instances`) |
| `*` | redirect `→ /` | public | SPA fallback via `BrowserRouter` (`<Navigate to="/" replace />`) |

Protected routes are wrapped in `<ProtectedRoute><Layout/></ProtectedRoute>` using `useAuth()` (`GET /api/v1/auth/validate`, token in `localStorage` `was_token`). `BrowserRouter` requires SPA fallback on static hosts (see Build).

Route config: `src/App.tsx`.

## Build

```bash
npm run build   # tsc -b && vite build → dist/
npm run preview # vite preview dist/ on http://localhost:4173
```

- **Output**: `web/dist/` (gitignored). Vite emits `dist/index.html` + `dist/assets/index-*.{js,css}` with content-hash, deterministic for same input. Clean with `rm -rf dist` before rebuild if needed.
- **Typecheck**: `tsc -b` (project refs `tsconfig.app.json` + `tsconfig.node.json`, `noEmit`). CI runs `npx tsc -b` separately.
- **Lint**: `npm run lint` → `oxlint` (`web/.oxlintrc.json`).
- **Preview**: `vite preview` serves `dist/` at `http://localhost:4173` (see `vite.config.ts` `preview.port`). Vite handles SPA history fallback to `index.html` automatically.
- **VITE_API_BASE handling**:
  - Default `""` → relative fetch, dev proxy (`vite.config.ts` `server.proxy: /api → http://localhost:3000`, `/api-docs → 3000`) handles locally.
  - Production: set at build time — `VITE_API_BASE=https://api.example.com npm run build` or `echo 'VITE_API_BASE=https://api.example.com' > .env` then `npm run build`. `src/api/client.ts` concatenates `BASE + path`, so `BASE` should have no trailing slash.
- **SPA fallback** (required for `BrowserRouter`):
  - `vite preview` already does fallback.
  - Nginx: `try_files $uri $uri/ /index.html;`
  - Caddy: `try_files {path} /index.html`
  - `npx serve -s dist` (`-s` = SPA)
  - Vercel: `vercel.json` `{ "rewrites": [{ "source": "/(.*)", "destination": "/index.html" }] }`
  - Netlify: `_redirects` file `/* /index.html 200`
  - Static hosts must serve `index.html` for any unknown path, otherwise deep links (`/app`, `/login`) 404.

Backend must run on `:3000` (`cargo run`) for dev proxy.

## API

See `src/api/endpoints.ts` — typed wrappers for `GET /api/health`, `/api/v1/instances`, `/api/v1/auth/*`, `/api/v1/users/*`. Includes `Authorization: Bearer <token>` via localStorage `was_token`.
