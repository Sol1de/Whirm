import type { Proxy } from '$lib/types';

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
    }
  };
}

export const proxyStore = createProxyStore();
