# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
pnpm dev          # Start Vite dev server (frontend only, port 5173)
pnpm build        # Production build (frontend only)
pnpm check        # TypeScript + Svelte type-check (run before committing)
pnpm preview      # Preview production build

# Tauri (desktop app) — requires Rust toolchain
pnpm tauri dev    # Start full desktop app (launches Vite + Tauri window)
pnpm tauri build  # Bundle desktop app for distribution
```

There are no tests in this project yet.

## Architecture

**Vite + Svelte 5 + Tauri 2** — this is NOT a SvelteKit project. There is no file-based routing, no SSR, and no server-side code in the frontend.

### Routing
Navigation is handled entirely via a `$state` store in `src/lib/stores/navigation.svelte.ts`. The `Sidebar` component calls `navigation.navigate(route)`, and `App.svelte` renders one of three page components via `{#if}` blocks. Routes are typed as `'dashboard' | 'proxies' | 'settings'`.

### State management & services
Frontend state and backend communication live in `src/lib/services/`. Services that hold reactive state use the **`.svelte.ts` extension** (mandatory for Svelte 5 runes in non-component files — plain `.ts` causes a compiler error). Stateless services can use plain `.ts`.

Services expose state via getter properties and action methods. Components read state through getters and never write directly to service internals. All `invoke()` calls are placed exclusively in services (never inside components).

Current services:
- `proxyService` (`proxy.service.svelte.ts`) — proxy list + `isAddSheetOpen` flag; `getProxies()` / `add()` / `remove()` / `update()` / `test()` / `openAddSheet()` / `closeAddSheet()`
- `connectionService` (`connection.service.svelte.ts`) — tunnel status, active proxy, IP, speeds; `connect(proxyId)` / `disconnect()`
- `settingsService` (`settings.service.svelte.ts`) — persists to SQLite via Tauri; exposes `draft`, `update(patch)`, `save()`, `reset()`. Settings fields: `globalTimeout` (default 5000 ms), `dnsLeakProtection` (boolean), `proxyProtocol` (default `'SOCKS5'`)
- `sessionService` (`session.service.ts`) — stateless; `open()` / `close()` / `getAll()` for connection session history

Navigation remains in `src/lib/stores/navigation.svelte.ts` (only store left).

### Database (SeaORM + SQLite)
The backend uses **SeaORM 2.0-rc** with SQLite. The database file (`whirm.db`) is created in the Tauri app data directory. Migrations run automatically on startup via `db::init()`.

- **Entities** live in `src-tauri/src/db/entities/` — `proxy`, `settings`, `connection_session`, plus enums in `entities/enums/proxy.rs`
- **Migrations** live in `src-tauri/src/db/migrations/` — named `m20240101_NNNNNN_description.rs`, registered in `migrations/mod.rs`
- A default settings row is seeded on first run if none exists
- Entity models derive `Serialize` with `#[serde(rename_all = "camelCase")]` so field names match TypeScript types over the Tauri bridge

When adding a new table: create a migration file, register it in `migrations/mod.rs`, create an entity module, and register it in `entities/mod.rs`.

### Tauri integration
The Rust backend lives in `src-tauri/`. Frontend-to-Rust calls use `invoke()` from `@tauri-apps/api/core`.

Tauri commands in `src-tauri/src/lib.rs`:
- **Proxy tunnel**: `connect_proxy`, `disconnect_proxy`, `test_proxy`
- **Proxy CRUD**: `get_proxies`, `add_proxy`, `update_proxy`, `delete_proxy`
- **Settings**: `get_settings`, `save_settings`
- **Sessions**: `open_session`, `close_session`, `get_sessions`

Commands that accept structured input use dedicated `*Input` structs with `#[serde(rename_all = "camelCase")]`. On the frontend, pass inputs as `{ input: { ... } }` to match Tauri's argument naming.

`AppState` (Tauri managed state) holds: `saved_proxy: Mutex<Option<Sysproxy>>`, `db: DatabaseConnection`, `active_session_id: Mutex<Option<String>>`, `active_proxy_id: Mutex<Option<String>>`. The exit handler restores the original system proxy and closes the active session on app exit — this prevents stale proxy settings if the app crashes.

Always use typed generics on `invoke()` calls: `invoke<string>(...)`, `invoke<number>(...)`, etc.

Key Rust deps: `sysproxy` (system proxy reads/writes), `reqwest` with SOCKS5 support (connectivity test), `sea-orm` + `sea-orm-migration` (database), `chrono` (timestamps), `uuid` (primary keys).

**Password encryption**: proxy passwords are encrypted at rest via AES-256-GCM with a machine-specific key derived using HKDF-SHA256 (`machine-uid` crate, salt `"whirm-field-v1"`). Implementation in `src-tauri/src/crypto.rs`. `add_proxy`/`update_proxy` encrypt before DB write; `connect_proxy` decrypts before use. Never compare or forward the raw DB `password` field — it is ciphertext.

**System proxy bypass**: `connect_proxy` hardcodes bypass list `"localhost,127.0.0.1,<local>"` — not currently user-configurable.

### Component structure
- `src/lib/components/layout/` — `Sidebar` and `TopAppBar` are shared across all three pages
- `src/lib/components/{dashboard,proxies,settings}/` — feature-specific components
- `src/lib/components/ui/` — auto-generated by shadcn-svelte (do not edit manually)
- `src/lib/pages/` — page-level components that compose layout + feature components

### UI components
shadcn-svelte is configured with the **`vega` style** and **neutral base color**. Add components with:
```bash
pnpm dlx shadcn-svelte@latest add <component-name>
```
Note: the `toast` component does not exist in the vega registry — use `sonner` instead (`$lib/components/ui/sonner`).

Icons come from `@lucide/svelte` (already installed). Import as named exports: `import { Settings } from '@lucide/svelte'`.

### Styling
- Tailwind CSS 4 (Vite plugin, no `tailwind.config.js`)
- Dark theme only — all CSS tokens in `src/app.css` under `:root` use `oklch` color space
- shadcn tokens (`--background`, `--foreground`, `--card`, etc.) are the only custom tokens; `--radius` is set to `0.125rem` (2 px, nearly square corners)
- The `cn()` utility from `$lib/utils` merges Tailwind classes (clsx + tailwind-merge)
- Two fonts: **Inter Variable** (body/UI, `font-sans`) and **Space Grotesk** (logo, section labels, monospace values, `font-grotesk` CSS var). Use `font-['Space_Grotesk',sans-serif]` or the `font-grotesk` Tailwind utility
- Design palette: `zinc-950` bg · `zinc-900` cards/sidebar · `zinc-800` borders · `emerald-500` connected state · `violet-300` accent dot · `rounded-[2px]` on all interactive elements

### Types
Core types live in `src/lib/types/index.ts`:
- `Route` — `'dashboard' | 'proxies' | 'settings'`
- `ProxyProtocol` — `'SOCKS5' | 'HTTP' | 'HTTPS'`
- `ProxyStatus` — `'active' | 'inactive' | 'error'`
- `ConnectionStatus` — `'connected' | 'disconnected' | 'connecting'`
- `Proxy` — main entity: `id`, `name`, `host`, `port`, `protocol`, `country`, `countryCode`, `status`, optional `username`, `category`, `lastUsedAt`
- `ConnectionState` — `status`, `activeProxy`, `currentIp`, `downloadSpeed`, `uploadSpeed`
- `ConnectionSession` — `id`, `proxyId`, `connectedAt`, `disconnectedAt`, `ipAddress`
- `Settings` — **not yet defined in this file**; fields used by `settingsService`: `globalTimeout: number`, `dnsLeakProtection: boolean`, `proxyProtocol: ProxyProtocol`

### Svelte 5 patterns
This codebase uses Svelte 5 runes exclusively:
- `$state()` for reactive state
- `$derived()` for computed values  
- `$props()` for component props (with TypeScript interfaces defined as `interface Props {}` above the destructure)
- `$effect()` for side effects
- No legacy `export let`, no `$:` reactive statements, no Svelte stores (`writable`/`readable`)
