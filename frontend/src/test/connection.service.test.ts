import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

const mockedInvoke = vi.mocked(invoke);

const mockProxy = {
  id: 'proxy-1',
  name: 'Test Proxy',
  host: '1.2.3.4',
  port: 1080,
  protocol: 'SOCKS5' as const,
  country: 'US',
  countryCode: 'US',
  status: 'inactive' as const,
};

describe('connectionService', () => {
  let connectionService: Awaited<typeof import('../services/connection.service.svelte')>['connectionService'];
  let proxyService: Awaited<typeof import('../services/proxy.service.svelte')>['proxyService'];

  beforeEach(async () => {
    vi.resetModules();
    mockedInvoke.mockReset();
    const connMod = await import('../services/connection.service.svelte');
    const proxyMod = await import('../services/proxy.service.svelte');
    connectionService = connMod.connectionService;
    proxyService = proxyMod.proxyService;
    proxyService._reset();
  });

  async function seedProxy() {
    mockedInvoke.mockResolvedValueOnce([mockProxy]);
    await proxyService.getProxies();
    mockedInvoke.mockReset();
  }

  describe('connect', () => {
    it('transitions to connected state on success', async () => {
      await seedProxy();
      expect(connectionService.state.status).toBe('disconnected');
      mockedInvoke.mockResolvedValueOnce('5.5.5.5');
      mockedInvoke.mockResolvedValueOnce('session-1');
      await connectionService.connect('proxy-1');
      expect(connectionService.state.status).toBe('connected');
      expect(connectionService.state.currentIp).toBe('5.5.5.5');
      expect(connectionService.state.activeProxy?.id).toBe('proxy-1');
    });

    it('reverts to disconnected on connect failure', async () => {
      await seedProxy();
      mockedInvoke.mockRejectedValueOnce(new Error('Proxy unreachable'));
      await expect(connectionService.connect('proxy-1')).rejects.toThrow('Proxy unreachable');
      expect(connectionService.state.status).toBe('disconnected');
      expect(connectionService.state.activeProxy).toBeNull();
    });

    it('throws if proxy not found', async () => {
      await expect(connectionService.connect('nonexistent')).rejects.toThrow('Proxy not found');
    });

    it('is a no-op when already connecting or connected', async () => {
      await seedProxy();
      mockedInvoke.mockResolvedValueOnce('5.5.5.5');
      mockedInvoke.mockResolvedValueOnce('session-1');
      await connectionService.connect('proxy-1');
      mockedInvoke.mockReset();
      await connectionService.connect('proxy-1');
      expect(mockedInvoke).not.toHaveBeenCalled();
    });
  });

  describe('disconnect', () => {
    it('resets state on success', async () => {
      await seedProxy();
      mockedInvoke.mockResolvedValueOnce('5.5.5.5');
      mockedInvoke.mockResolvedValueOnce('session-1');
      await connectionService.connect('proxy-1');
      mockedInvoke.mockResolvedValueOnce(undefined);
      mockedInvoke.mockResolvedValueOnce(undefined);
      await connectionService.disconnect();
      expect(connectionService.state.status).toBe('disconnected');
      expect(connectionService.state.activeProxy).toBeNull();
      expect(connectionService.state.currentIp).toBeNull();
    });

    it('is a no-op when already disconnected', async () => {
      mockedInvoke.mockReset();
      await connectionService.disconnect();
      expect(mockedInvoke).not.toHaveBeenCalled();
    });
  });
});
