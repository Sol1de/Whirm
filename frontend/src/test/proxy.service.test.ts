import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

const mockedInvoke = vi.mocked(invoke);

const mockProxy = {
  id: 'proxy-1',
  name: 'Test Proxy',
  host: '1.2.3.4',
  port: 1080,
  protocol: 'SOCKS5' as const,
  country: 'United States',
  countryCode: 'US',
  status: 'inactive' as const,
};

describe('proxyService', () => {
  let proxyService: Awaited<typeof import('../services/proxy.service.svelte')>['proxyService'];

  beforeEach(async () => {
    vi.resetModules();
    mockedInvoke.mockReset();
    const mod = await import('../services/proxy.service.svelte');
    proxyService = mod.proxyService;
    proxyService._reset();
  });

  describe('getProxies', () => {
    it('populates proxies list', async () => {
      mockedInvoke.mockResolvedValueOnce([mockProxy]);
      await proxyService.getProxies();
      expect(proxyService.proxies).toEqual([mockProxy]);
      expect(mockedInvoke).toHaveBeenCalledWith('get_proxies');
    });
  });

  describe('add', () => {
    it('appends created proxy to list', async () => {
      mockedInvoke.mockResolvedValueOnce(mockProxy);
      const result = await proxyService.add({
        name: 'Test Proxy',
        host: '1.2.3.4',
        port: 1080,
        protocol: 'SOCKS5',
        country: '',
        countryCode: '',
      });
      expect(result).toEqual(mockProxy);
      expect(proxyService.proxies).toEqual(expect.arrayContaining([mockProxy]));
      expect(mockedInvoke).toHaveBeenCalledWith('add_proxy', expect.objectContaining({ input: expect.any(Object) }));
    });
  });

  describe('remove', () => {
    it('removes proxy from list', async () => {
      mockedInvoke.mockResolvedValueOnce([mockProxy]);
      await proxyService.getProxies();
      mockedInvoke.mockResolvedValueOnce(undefined);
      await proxyService.remove('proxy-1');
      expect(proxyService.proxies).toHaveLength(0);
      expect(mockedInvoke).toHaveBeenCalledWith('delete_proxy', { id: 'proxy-1' });
    });
  });

  describe('update', () => {
    it('replaces proxy in list with updated version', async () => {
      mockedInvoke.mockResolvedValueOnce([mockProxy]);
      await proxyService.getProxies();
      const updated = { ...mockProxy, name: 'Updated' };
      mockedInvoke.mockResolvedValueOnce(updated);
      const result = await proxyService.update('proxy-1', { name: 'Updated' });
      expect(result.name).toBe('Updated');
      expect(proxyService.proxies[0].name).toBe('Updated');
    });

    it('throws if proxy not found', async () => {
      await expect(proxyService.update('nonexistent', { name: 'X' })).rejects.toThrow('Proxy not found');
    });
  });

  describe('test', () => {
    it('calls test_proxy with correct args', async () => {
      mockedInvoke.mockResolvedValueOnce(42);
      const latency = await proxyService.test('1.2.3.4', 1080, 'SOCKS5', 'user', 'pass', 5000);
      expect(latency).toBe(42);
      expect(mockedInvoke).toHaveBeenCalledWith('test_proxy', {
        host: '1.2.3.4',
        port: 1080,
        protocol: 'SOCKS5',
        username: 'user',
        password: 'pass',
        timeout: 5000,
      });
    });

    it('passes null for optional fields', async () => {
      mockedInvoke.mockResolvedValueOnce(10);
      await proxyService.test('1.2.3.4', 1080, 'HTTP');
      expect(mockedInvoke).toHaveBeenCalledWith('test_proxy', {
        host: '1.2.3.4',
        port: 1080,
        protocol: 'HTTP',
        username: null,
        password: null,
        timeout: null,
      });
    });
  });

  describe('filteredProxies', () => {
    beforeEach(async () => {
      const proxies = [
        { ...mockProxy, id: '1', name: 'US Proxy', host: '1.1.1.1', country: 'United States', countryCode: 'US', protocol: 'SOCKS5' as const },
        { ...mockProxy, id: '2', name: 'DE Proxy', host: '2.2.2.2', country: 'Germany', countryCode: 'DE', protocol: 'HTTP' as const },
        { ...mockProxy, id: '3', name: 'SG Proxy', host: '3.3.3.3', country: 'Singapore', countryCode: 'SG', protocol: 'HTTPS' as const },
      ];
      mockedInvoke.mockResolvedValueOnce(proxies);
      await proxyService.getProxies();
    });

    it('returns all when no filters active', () => {
      expect(proxyService.filteredProxies).toHaveLength(3);
    });

    it('filters by search query on name', () => {
      proxyService.setSearch('US');
      expect(proxyService.filteredProxies).toHaveLength(1);
      expect(proxyService.filteredProxies[0].id).toBe('1');
    });

    it('filters by search query on host', () => {
      proxyService.setSearch('2.2.2');
      expect(proxyService.filteredProxies).toHaveLength(1);
      expect(proxyService.filteredProxies[0].id).toBe('2');
    });

    it('filters by protocol', () => {
      proxyService.setProtocolFilter('HTTP');
      expect(proxyService.filteredProxies).toHaveLength(1);
      expect(proxyService.filteredProxies[0].id).toBe('2');
    });

    it('filters by US region', () => {
      proxyService.setRegionFilter('us');
      expect(proxyService.filteredProxies).toHaveLength(1);
      expect(proxyService.filteredProxies[0].id).toBe('1');
    });

    it('filters by EU region', () => {
      proxyService.setRegionFilter('eu');
      expect(proxyService.filteredProxies).toHaveLength(1);
      expect(proxyService.filteredProxies[0].id).toBe('2');
    });

    it('filters by Asia region', () => {
      proxyService.setRegionFilter('asia');
      expect(proxyService.filteredProxies).toHaveLength(1);
      expect(proxyService.filteredProxies[0].id).toBe('3');
    });

    it('resets to all when filter cleared', () => {
      proxyService.setSearch('US');
      expect(proxyService.filteredProxies).toHaveLength(1);
      proxyService.setSearch('');
      expect(proxyService.filteredProxies).toHaveLength(3);
    });
  });

  describe('latency', () => {
    it('returns undefined for unknown proxy', () => {
      expect(proxyService.getLatency('nonexistent')).toBeUndefined();
    });

    it('stores and retrieves latency', () => {
      proxyService.setLatency('proxy-1', 42);
      expect(proxyService.getLatency('proxy-1')).toBe(42);
    });
  });

  describe('edit sheet', () => {
    it('opens and closes edit sheet', () => {
      expect(proxyService.isEditSheetOpen).toBe(false);
      proxyService.openEditSheet(mockProxy);
      expect(proxyService.isEditSheetOpen).toBe(true);
      expect(proxyService.editingProxy).toEqual(mockProxy);
      proxyService.closeEditSheet();
      expect(proxyService.isEditSheetOpen).toBe(false);
      expect(proxyService.editingProxy).toBeNull();
    });
  });
});
