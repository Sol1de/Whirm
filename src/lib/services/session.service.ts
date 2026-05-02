import { invoke } from '@tauri-apps/api/core';
import type { ConnectionSession } from '$lib/types';

export const sessionService = {
  open(proxyId: string, ipAddress: string): Promise<string> {
    return invoke<string>('open_session', { proxyId, ipAddress });
  },
  close(sessionId: string): Promise<void> {
    return invoke<void>('close_session', { sessionId });
  },
  getAll(limit?: number): Promise<ConnectionSession[]> {
    return invoke<ConnectionSession[]>('get_sessions', { limit: limit ?? null });
  },
};
