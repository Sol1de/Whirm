import type { ProxyProtocol } from '$lib/types';

interface Settings {
  globalTimeout: number;
  dnsLeakProtection: boolean;
  proxyProtocol: ProxyProtocol;
}

const STORAGE_KEY = 'whirm-settings';

const defaults: Settings = {
  globalTimeout: 5000,
  dnsLeakProtection: true,
  proxyProtocol: 'SOCKS5'
};

function createSettingsStore() {
  const stored =
    typeof localStorage !== 'undefined'
      ? (JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null') as Settings | null)
      : null;

  let draft = $state<Settings>(stored ?? { ...defaults });

  return {
    get draft() {
      return draft;
    },
    update(patch: Partial<Settings>) {
      draft = { ...draft, ...patch };
    },
    save() {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(draft));
    },
    reset() {
      draft = { ...defaults };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(defaults));
    }
  };
}

export const settingsStore = createSettingsStore();
