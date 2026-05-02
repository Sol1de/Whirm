# Whirm

Application desktop de gestion de proxies. Permet de configurer, organiser et basculer entre des proxies SOCKS5, HTTP et HTTPS depuis une interface unifiée.

**Stack :** Svelte 5 · TypeScript · Tailwind CSS 4 · shadcn-svelte · Tauri 2 (Rust)

---

## Prérequis

- **Node.js** ≥ 18 + **pnpm** (`npm i -g pnpm`)
- **Rust** ≥ 1.77.2 (`rustup` recommandé) — uniquement pour lancer l'app desktop
- Dépendances système Tauri : [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)

---

## Démarrage rapide

```bash
# Installer les dépendances
pnpm install

# Dev — frontend seul (navigateur, port 5173)
pnpm dev

# Dev — app desktop complète (Tauri + Rust + Vite)
pnpm tauri dev
```

---

## Commandes

| Commande | Description |
|---|---|
| `pnpm dev` | Serveur de développement frontend (HMR, port 5173) |
| `pnpm build` | Build de production frontend |
| `pnpm preview` | Prévisualiser le build de production |
| `pnpm check` | Vérification TypeScript + Svelte |
| `pnpm tauri dev` | App desktop en développement |
| `pnpm tauri build` | Bundle desktop (macOS / Windows / Linux) |

---

## Structure du projet

```
src/
├── App.svelte              # Shell principal + routing
└── lib/
    ├── types/              # Interfaces TypeScript partagées
    ├── stores/             # Navigation uniquement
    ├── services/           # État + logique métier (proxies, connexion, settings, sessions)
    ├── pages/              # Dashboard · Proxies · Settings
    └── components/
        ├── layout/         # Sidebar et TopAppBar (partagés)
        ├── dashboard/      # Cartes status, stats, table récente
        ├── proxies/        # Table, filtres, panel "Add Proxy"
        └── settings/       # Formulaire paramètres connexion
src-tauri/
├── src/
│   ├── lib.rs              # Commandes Tauri + AppState
│   └── db/
│       ├── mod.rs           # Init DB + seed
│       ├── entities/        # Modèles SeaORM (proxy, settings, connection_session)
│       └── migrations/      # Migrations SQLite
```

## Backend Rust

### Base de données

SQLite via **SeaORM 2.0-rc**. Le fichier `whirm.db` est créé automatiquement dans le répertoire de données de l'app. Les migrations s'exécutent au démarrage ; une ligne de settings par défaut est insérée au premier lancement.

### Commandes Tauri

Exposées au frontend via `invoke()` :

| Catégorie | Commandes | Rôle |
|---|---|---|
| **Tunnel** | `connect_proxy`, `disconnect_proxy`, `test_proxy` | Connectivité proxy, sauvegarde/restauration du proxy système, mesure de latence |
| **CRUD Proxies** | `get_proxies`, `add_proxy`, `update_proxy`, `delete_proxy` | Gestion persistante des proxies en base |
| **Settings** | `get_settings`, `save_settings` | Lecture/écriture des paramètres globaux |
| **Sessions** | `open_session`, `close_session`, `get_sessions` | Historique des connexions |

À la fermeture de l'app (même en cas de crash), le proxy système est automatiquement restauré et la session active est fermée.

---

## Ajouter un composant UI

```bash
pnpm dlx shadcn-svelte@latest add <nom-composant>
```

Les composants sont installés dans `src/lib/components/ui/` et ne doivent pas être modifiés manuellement.
