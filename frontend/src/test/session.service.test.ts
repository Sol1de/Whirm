import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

const mockedInvoke = vi.mocked(invoke);

const mockSession = {
  id: 'session-1',
  proxyId: 'proxy-1',
  connectedAt: '2024-01-01T00:00:00Z',
  disconnectedAt: null,
  ipAddress: '1.2.3.4',
};

describe('sessionService', () => {
  let sessionService: Awaited<typeof import('../services/session.service')>['sessionService'];

  beforeEach(async () => {
    vi.resetModules();
    mockedInvoke.mockReset();
    const mod = await import('../services/session.service');
    sessionService = mod.sessionService;
  });

  describe('open', () => {
    it('invokes open_session and returns session id', async () => {
      mockedInvoke.mockResolvedValueOnce('session-1');
      const id = await sessionService.open('proxy-1', '1.2.3.4');
      expect(id).toBe('session-1');
      expect(mockedInvoke).toHaveBeenCalledWith('open_session', {
        proxyId: 'proxy-1',
        ipAddress: '1.2.3.4',
      });
    });
  });

  describe('close', () => {
    it('invokes close_session with session id', async () => {
      mockedInvoke.mockResolvedValueOnce(undefined);
      await sessionService.close('session-1');
      expect(mockedInvoke).toHaveBeenCalledWith('close_session', { sessionId: 'session-1' });
    });
  });

  describe('getAll', () => {
    it('invokes get_sessions without limit', async () => {
      mockedInvoke.mockResolvedValueOnce([mockSession]);
      const sessions = await sessionService.getAll();
      expect(sessions).toEqual([mockSession]);
      expect(mockedInvoke).toHaveBeenCalledWith('get_sessions', { limit: null });
    });

    it('passes limit when provided', async () => {
      mockedInvoke.mockResolvedValueOnce([mockSession]);
      await sessionService.getAll(10);
      expect(mockedInvoke).toHaveBeenCalledWith('get_sessions', { limit: 10 });
    });
  });
});
