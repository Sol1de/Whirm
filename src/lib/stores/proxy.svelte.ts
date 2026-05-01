import type { Proxy, ProxyProtocol } from '$lib/types';
import { invoke } from '@tauri-apps/api/core';

function createProxyStore() {
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
    addProxy(proxy: Proxy) {
      proxies = [...proxies, proxy];
    },
    removeProxy(id: string) {
      proxies = proxies.filter((p) => p.id !== id);
    },
    updateProxy(id: string, updates: Partial<Proxy>) {
      proxies = proxies.map((p) => (p.id === id ? { ...p, ...updates } : p));
    },
    testProxy(
      host: string,
      port: number,
      protocol: ProxyProtocol,
      username?: string,
      password?: string,
    ) {
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

export const proxyStore = createProxyStore();
