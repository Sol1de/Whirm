import { invoke } from '@tauri-apps/api/core';
import type { ConnectionState, Proxy } from '@types';
import { proxyService } from '@services/proxy.service.svelte';
import { sessionService } from '@services/session.service';
import { notificationService } from '@services/notification.service.svelte';

function createConnectionService() {
  let state = $state<ConnectionState>({
    status: 'disconnected',
    activeProxy: null,
    currentIp: null,
    downloadSpeed: 0,
    uploadSpeed: 0,
  });

  let sessionId = $state<string | null>(null);

  return {
    get state() {
      return state;
    },

    async connect(proxyId: string) {
      if (state.status === 'connecting' || state.status === 'connected') return;

      const proxy = proxyService.proxies.find((p: Proxy) => p.id === proxyId);
      if (!proxy) throw new Error(`Proxy not found (id: ${proxyId})`);

      state = { ...state, status: 'connecting' };

      try {
        const ip = await invoke<string>('connect_proxy', {
          proxyId: proxy.id,
        });

        try {
          sessionId = await sessionService.open(proxy.id, ip);
        } catch (sessionError) {
          await invoke('disconnect_proxy').catch(() => {});
          throw sessionError;
        }

        state = {
          status: 'connected',
          activeProxy: proxy,
          currentIp: ip,
          downloadSpeed: 0,
          uploadSpeed: 0,
        };
        notificationService.add('success', `Connected to ${proxy.name}`);
      } catch (error) {
        state = {
          status: 'disconnected',
          activeProxy: null,
          currentIp: null,
          downloadSpeed: 0,
          uploadSpeed: 0,
        };
        const msg = error instanceof Error ? error.message : String(error);
        notificationService.add('error', `Connection failed: ${msg}`);
        throw error;
      }
    },

    async disconnect() {
      if (state.status === 'disconnected') return;

      if (sessionId !== null) {
        try {
          await sessionService.close(sessionId);
        } catch (error) {
          console.error('Failed to close session:', error);
        }
        sessionId = null;
      }

      let invokeError: unknown = null;
      try {
        await invoke('disconnect_proxy');
      } catch (error) {
        console.error('Failed to disconnect:', error);
        invokeError = error;
      }

      state = {
        status: 'disconnected',
        activeProxy: null,
        currentIp: null,
        downloadSpeed: 0,
        uploadSpeed: 0,
      };

      if (invokeError) {
        const msg = invokeError instanceof Error ? invokeError.message : String(invokeError);
        notificationService.add('error', `Disconnect failed: ${msg}`);
        throw invokeError;
      } else {
        notificationService.add('info', 'Disconnected from proxy');
      }
    },
  };
}

export const connectionService = createConnectionService();
