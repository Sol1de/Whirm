import { invoke } from '@tauri-apps/api/core';
import type { ConnectionState, Proxy } from '$lib/types';
import { proxyStore } from '$lib/stores/proxy.svelte';

function createConnectionStore() {
  let state = $state<ConnectionState>({
    status: 'disconnected',
    activeProxy: null,
    currentIp: null,
    downloadSpeed: 0,
    uploadSpeed: 0
  });

  return {
    get state() {
      return state;
    },

    async connect(proxyId: string) {
      if (state.status === 'connecting' || state.status === 'connected') return;

      const proxy = proxyStore.proxies.find((p: Proxy) => p.id === proxyId);
      if (!proxy) {
        throw new Error(`Proxy introuvable (id: ${proxyId})`);
      }

      state = { ...state, status: 'connecting' };

      try {
        const ip = await invoke<string>('connect_proxy', {
          host: proxy.host,
          port: proxy.port,
          protocol: proxy.protocol,
          username: proxy.username ?? null,
          password: proxy.password ?? null
        });

        state = {
          status: 'connected',
          activeProxy: proxy,
          currentIp: ip,
          downloadSpeed: 0,
          uploadSpeed: 0
        };
        proxyStore.updateProxy(proxy.id, { lastUsed: new Date() });
      } catch (error) {
        console.error('Échec de la connexion au proxy :', error);
        state = {
          status: 'disconnected',
          activeProxy: null,
          currentIp: null,
          downloadSpeed: 0,
          uploadSpeed: 0
        };
        throw error;
      }
    },

    async disconnect() {
      if (state.status === 'disconnected') return;

      let invokeError: unknown = null;
      try {
        await invoke('disconnect_proxy');
      } catch (error) {
        console.error('Échec de la déconnexion :', error);
        invokeError = error;
      }
      state = {
        status: 'disconnected',
        activeProxy: null,
        currentIp: null,
        downloadSpeed: 0,
        uploadSpeed: 0
      };
      if (invokeError) throw invokeError;
    }
  };
}

export const connectionStore = createConnectionStore();
