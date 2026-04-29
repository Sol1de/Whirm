# Kreios Proxy

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
| `pnpm dev` | Serveur de développement frontend (HMR) |
| `pnpm build` | Build de production frontend |
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
    ├── stores/             # État global (navigation, proxies, connexion)
    ├── pages/              # Dashboard · Proxies · Settings
    └── components/
        ├── layout/         # Sidebar et TopAppBar (partagés)
        ├── dashboard/      # Cartes status, stats, table récente
        ├── proxies/        # Table, filtres, panel "Add Proxy"
        └── settings/       # Formulaire paramètres connexion
src-tauri/                  # Backend Rust (Tauri 2)
```

## Ajouter un composant UI

```bash
pnpm dlx shadcn-svelte@latest add <nom-composant>
```

Les composants sont installés dans `src/lib/components/ui/` et ne doivent pas être modifiés manuellement.
