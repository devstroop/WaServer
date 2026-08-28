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

## Routes

- `/login`, `/register` (public)
- `/` Dashboard (health + instances overview, polls 5s)
- `/instances`, `/instances/:id` (create, QR, warmup, reset, send message with file)
- `/users`, `/users/:id` (tokens)
- `/me` profile

## Build

```bash
npm run build # tsc + vite build → dist/
npm run preview
```

Backend must run on `:3000` (`cargo run`).

## API

See `src/api/endpoints.ts` — typed wrappers for `GET /api/health`, `/api/v1/instances`, `/api/v1/auth/*`, `/api/v1/users/*`. Includes `Authorization: Bearer <token>` via localStorage `was_token`.
