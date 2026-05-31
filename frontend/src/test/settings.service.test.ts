import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

const mockedInvoke = vi.mocked(invoke);

const mockSettings = {
  id: 'settings-1',
  globalTimeout: 5000,
  dnsLeakProtection: true,
  proxyProtocol: 'SOCKS5' as const,
};

describe('settingsService', () => {
  let settingsService: Awaited<typeof import('../services/settings.service.svelte')>['settingsService'];

  beforeEach(async () => {
    vi.resetModules();
    mockedInvoke.mockReset();
    const mod = await import('../services/settings.service.svelte');
    settingsService = mod.settingsService;
  });

  describe('getSettings', () => {
    it('loads settings into draft', async () => {
      mockedInvoke.mockResolvedValueOnce(mockSettings);
      await settingsService.getSettings();
      expect(settingsService.draft.globalTimeout).toBe(5000);
      expect(settingsService.draft.dnsLeakProtection).toBe(true);
      expect(settingsService.draft.proxyProtocol).toBe('SOCKS5');
      expect(mockedInvoke).toHaveBeenCalledWith('get_settings');
    });
  });

  describe('update', () => {
    it('partially updates draft', async () => {
      mockedInvoke.mockResolvedValueOnce(mockSettings);
      await settingsService.getSettings();
      settingsService.update({ globalTimeout: 10000 });
      expect(settingsService.draft.globalTimeout).toBe(10000);
      expect(settingsService.draft.dnsLeakProtection).toBe(true);
    });
  });

  describe('save', () => {
    it('calls save_settings with id and current draft', async () => {
      mockedInvoke.mockResolvedValueOnce(mockSettings);
      await settingsService.getSettings();
      mockedInvoke.mockResolvedValueOnce(undefined);
      await settingsService.save();
      expect(mockedInvoke).toHaveBeenCalledWith('save_settings', {
        input: { id: 'settings-1', globalTimeout: 5000, dnsLeakProtection: true, proxyProtocol: 'SOCKS5' },
      });
    });

    it('throws if settings not loaded', async () => {
      await expect(settingsService.save()).rejects.toThrow('Settings not loaded');
    });
  });

  describe('reset', () => {
    it('resets draft to defaults and persists', async () => {
      mockedInvoke.mockResolvedValueOnce(mockSettings);
      await settingsService.getSettings();
      settingsService.update({ globalTimeout: 99999, dnsLeakProtection: false });
      mockedInvoke.mockResolvedValueOnce(undefined);
      await settingsService.reset();
      expect(settingsService.draft.globalTimeout).toBe(5000);
      expect(settingsService.draft.dnsLeakProtection).toBe(true);
    });
  });
});
