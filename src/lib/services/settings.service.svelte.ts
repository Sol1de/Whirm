import { invoke } from '@tauri-apps/api/core';
import type { ProxyProtocol } from '$lib/types';

interface Settings {
  id: string;
  globalTimeout: number;
  dnsLeakProtection: boolean;
  proxyProtocol: ProxyProtocol;
}

const defaults: Omit<Settings, 'id'> = {
  globalTimeout: 5000,
  dnsLeakProtection: true,
  proxyProtocol: 'SOCKS5',
};

function createSettingsService() {
  let id = '';
  let draft = $state<Omit<Settings, 'id'>>({ ...defaults });

  return {
    get draft() {
      return draft;
    },
    async getSettings() {
      const { id: fetchedId, ...rest } = await invoke<Settings>('get_settings');
      id = fetchedId;
      draft = rest;
    },
    update(patch: Partial<Omit<Settings, 'id'>>) {
      draft = { ...draft, ...patch };
    },
    async save() {
      if (!id) throw new Error('Settings not loaded');
      await invoke<void>('save_settings', { input: { id, ...draft } });
    },
    async reset() {
      if (!id) throw new Error('Settings not loaded');
      draft = { ...defaults };
      await invoke<void>('save_settings', { input: { id, ...draft } });
    },
  };
}

export const settingsService = createSettingsService();
