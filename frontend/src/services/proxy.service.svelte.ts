import { invoke } from '@tauri-apps/api/core';
import type { Proxy, ProxyProtocol } from '@types';

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
  let editingProxy = $state<Proxy | null>(null);
  let searchQuery = $state('');
  let regionFilter = $state('all');
  let protocolFilter = $state('all');
  let latencyMap = $state<Map<string, number>>(new Map());

  const EU_CODES = new Set(['AT','BE','BG','CY','CZ','DE','DK','EE','ES','FI','FR','GB','GR','HR','HU','IE','IT','LT','LU','LV','MT','NL','PL','PT','RO','SE','SI','SK','CH','NO','IS']);
  const ASIA_CODES = new Set(['CN','JP','KR','IN','SG','TH','MY','ID','VN','HK','TW','PH','PK','BD','MM','KH','LA','NP','LK']);

  const filteredProxies = $derived(
    proxies.filter((p) => {
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const matches =
          p.name.toLowerCase().includes(q) ||
          p.host.toLowerCase().includes(q) ||
          (p.country ?? '').toLowerCase().includes(q);
        if (!matches) return false;
      }
      if (protocolFilter !== 'all' && p.protocol !== protocolFilter) return false;
      if (regionFilter !== 'all') {
        const code = (p.countryCode ?? '').toUpperCase();
        if (regionFilter === 'us' && code !== 'US') return false;
        if (regionFilter === 'eu' && !EU_CODES.has(code)) return false;
        if (regionFilter === 'asia' && !ASIA_CODES.has(code)) return false;
      }
      return true;
    })
  );

  return {
    get proxies() {
      return proxies;
    },
    get filteredProxies() {
      return filteredProxies;
    },
    get isAddSheetOpen() {
      return isAddSheetOpen;
    },
    get isEditSheetOpen() {
      return editingProxy !== null;
    },
    get editingProxy() {
      return editingProxy;
    },
    get searchQuery() {
      return searchQuery;
    },
    get regionFilter() {
      return regionFilter;
    },
    get protocolFilter() {
      return protocolFilter;
    },
    openAddSheet() {
      isAddSheetOpen = true;
    },
    closeAddSheet() {
      isAddSheetOpen = false;
    },
    openEditSheet(proxy: Proxy) {
      editingProxy = proxy;
    },
    closeEditSheet() {
      editingProxy = null;
    },
    setSearch(q: string) {
      searchQuery = q;
    },
    setRegionFilter(v: string) {
      regionFilter = v;
    },
    setProtocolFilter(v: string) {
      protocolFilter = v;
    },
    getLatency(id: string): number | undefined {
      return latencyMap.get(id);
    },
    setLatency(id: string, ms: number) {
      latencyMap = new Map(latencyMap).set(id, ms);
    },
    async testLatency(proxy: Proxy): Promise<number> {
      // Test by id so the backend decrypts the saved password itself —
      // the ciphertext never round-trips through the frontend.
      const ms = await invoke<number>('test_proxy_by_id', { id: proxy.id });
      latencyMap = new Map(latencyMap).set(proxy.id, ms);
      return ms;
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
      timeout?: number,
    ): Promise<number> {
      return invoke<number>('test_proxy', {
        host,
        port,
        protocol,
        username: username ?? null,
        password: password ?? null,
        timeout: timeout ?? null,
      });
    },
    testById(id: string): Promise<number> {
      return invoke<number>('test_proxy_by_id', { id });
    },
    _reset() {
      proxies = [];
      isAddSheetOpen = false;
      editingProxy = null;
      searchQuery = '';
      regionFilter = 'all';
      protocolFilter = 'all';
      latencyMap = new Map();
    },
  };
}

export const proxyService = createProxyService();
