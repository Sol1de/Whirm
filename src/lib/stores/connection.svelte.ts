import type { ConnectionState } from '$lib/types';

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
    // Future: invoke Tauri command to connect to a proxy
    async connect(_proxyId: string) {
      state = { ...state, status: 'connecting' };
      // TODO: await invoke('connect_proxy', { proxyId })
    },
    // Future: invoke Tauri command to disconnect
    async disconnect() {
      state = { ...state, status: 'disconnected', activeProxy: null, currentIp: null };
      // TODO: await invoke('disconnect_proxy')
    }
  };
}

export const connectionStore = createConnectionStore();
