import { invoke } from '@tauri-apps/api/core';
import type { Proxy, ProxyProtocol } from '$lib/types';

interface AddProxyInput {
  name: string;
  host: string;
  port: number;
  protocol: ProxyProtocol;
  country: string;
  countryCode: string;
  username?: string;
  password?: string;
  category?: string;
}

function createProxyService() {
  let proxies = $state<Proxy[]>([]);
  let isAddSheetOpen = $state(false);

  return {
    get proxies() {
      return proxies;
    },
    get isAddSheetOpen() {
      return isAddSheetOpen;
    },
    openAddSheet() {
      isAddSheetOpen = true;
    },
    closeAddSheet() {
      isAddSheetOpen = false;
    },
    async getProxies() {
      proxies = await invoke<Proxy[]>('get_proxies');
    },
    async add(input: AddProxyInput): Promise<Proxy> {
      const created = await invoke<Proxy>('add_proxy', {
        input: {
          name: input.name,
          host: input.host,
          port: input.port,
          protocol: input.protocol,
          country: input.country,
          countryCode: input.countryCode,
          username: input.username ?? null,
          password: input.password ?? null,
          category: input.category ?? null,
        },
      });
      proxies = [...proxies, created];
      return created;
    },
    async remove(id: string): Promise<void> {
      await invoke<void>('delete_proxy', { id });
      proxies = proxies.filter((p) => p.id !== id);
    },
    async update(id: string, updates: Partial<Proxy>, newPassword?: string): Promise<Proxy> {
      const current = proxies.find((p) => p.id === id);
      if (!current) throw new Error(`Proxy not found: ${id}`);

      const merged = { ...current, ...updates };
      const updated = await invoke<Proxy>('update_proxy', {
        input: {
          id: merged.id,
          name: merged.name,
          host: merged.host,
          port: merged.port,
          protocol: merged.protocol,
          country: merged.country,
          countryCode: merged.countryCode,
          username: merged.username ?? null,
          newPassword: newPassword ?? null,
          status: merged.status,
          category: merged.category ?? null,
        },
      });
      proxies = proxies.map((p) => (p.id === id ? updated : p));
      return updated;
    },
    test(
      host: string,
      port: number,
      protocol: ProxyProtocol,
      username?: string,
      password?: string,
    ): Promise<number> {
      return invoke<number>('test_proxy', {
        host,
        port,
        protocol,
        username: username ?? null,
        password: password ?? null,
      });
    },
  };
}

export const proxyService = createProxyService();
