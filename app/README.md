# WAS Web UI

React + TypeScript + shadcn/ui frontend for WAS (WhatsApp Server).

## Features

- **Dashboard** - Monitor server health, connection status, and uptime
- **Authentication** - QR code scanning and phone number pairing
- **Chat Interface** - Send and receive messages with WhatsApp-style UI
- **Settings** - Configure API token and appearance

## Development

```bash
# Install dependencies
npm install

# Start development server (proxies to WAS backend on :3000)
npm run dev

# Build for production
npm run build
```

## Production

Build the frontend and the Rust server will automatically serve it:

```bash
# Build frontend
cd app
npm install
npm run build

# The 'app/dist' folder will be served at root when running WAS
cd ..
cargo run --release
```

## Tech Stack

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **shadcn/ui** - Component library
- **React Query** - Data fetching
- **Zustand** - State management
- **React Router** - Routing
