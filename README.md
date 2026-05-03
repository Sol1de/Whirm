# Whirm

A desktop app for managing proxies without the headache. Add your SOCKS5, HTTP, or HTTPS proxies, switch between them in one click, and let Whirm handle the system-level wiring in the background.

Built with Svelte 5, TypeScript, Tailwind CSS 4, and Tauri 2 (Rust) under the hood.

---

## Getting started

### Prerequisites

- **Node.js ≥ 18** + **pnpm**
- **Rust ≥ 1.85**
- **Tauri system dependencies** for your OS quick list at [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)

### Quick start

```bash
pnpm install

# Frontend only
pnpm dev

# Desktop app
pnpm tauri dev
```

---

## Commands

| Command | What it does |
|---|---|
| `pnpm dev` | Frontend dev server with HMR on port 5173 |
| `pnpm build` | Production build of the frontend |
| `pnpm preview` | Preview that production build locally |
| `pnpm check` | TypeScript + Svelte type-check |
| `pnpm tauri dev` | Desktop app in dev mode (Vite + Tauri window) |
| `pnpm tauri build` | Bundle the app for distribution |

---

## Project structure

```
frontend/                   # UI — Svelte 5 + TypeScript
├── App.svelte              # Root shell + routing
├── main.ts                 # Entry point
├── app.css                 # Tailwind + theme tokens
└── src/
    ├── types/              # Shared TypeScript interfaces
    ├── stores/             # Navigation store
    ├── services/           # All state + business logic (proxies, connection, settings, sessions)
    ├── pages/              # Dashboard · Proxies · Settings
    └── components/
        ├── layout/         # Sidebar and TopAppBar
        ├── dashboard/      # Status cards, stats, recent sessions
        ├── proxies/        # Proxy table, filters, add panel
        ├── settings/       # Settings form
        └── ui/             # shadcn-svelte components — don't edit manually

desktop/                    # Native layer — Rust + Tauri 2
└── src/
    ├── lib.rs              # App setup + global state
    ├── crypto.rs           # AES-256-GCM password encryption
    ├── commands/           # Tauri command handlers
    └── db/
        ├── mod.rs          # DB init + first-run seeding
        ├── entities/       # SeaORM models
        └── migrations/     # SQLite migrations
```

---

## Import aliases

Rather than relative paths everywhere, imports use semantic aliases configured in `vite.config.ts` and `tsconfig.json`:

| Alias | Points to |
|---|---|
| `@components` | `frontend/src/components/` |
| `@ui` | `frontend/src/components/ui/` |
| `@pages` | `frontend/src/pages/` |
| `@services` | `frontend/src/services/` |
| `@stores` | `frontend/src/stores/` |
| `@types` | `frontend/src/types/` |
| `@utils` | `frontend/src/utils.ts` |

---

## Rust backend

### Database

The app uses **SeaORM 2.0-rc** on top of SQLite. The database file (`whirm.db`) is created automatically in the OS app data directory the first time you launch. Migrations run on startup, and a default settings row is seeded if none exists yet.

### Tauri commands

Everything the frontend needs from the native layer goes through `invoke()`:

| Category | Commands |
|---|---|
| Proxy connection | `connect_proxy`, `disconnect_proxy`, `test_proxy` |
| Proxy management | `get_proxies`, `add_proxy`, `update_proxy`, `delete_proxy` |
| Settings | `get_settings`, `save_settings` |
| Session history | `open_session`, `close_session`, `get_sessions` |

When the app closes — even if it crashes — the original system proxy is restored and the active session is properly closed.

---

## Adding a UI component

```bash
pnpm dlx shadcn-svelte@latest add <component-name>
```

Components land in `frontend/src/components/ui/`. The project uses the **vega** style with neutral base color. Note: `toast` isn't in the vega registry, use `sonner` instead.
