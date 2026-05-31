import { invoke } from '@tauri-apps/api/core';
import type { ConnectionState, Proxy } from '@types';
import { proxyService } from '@services/proxy.service.svelte';
import { sessionService } from '@services/session.service';
import { notificationService } from '@services/notification.service.svelte';

interface Throughput {
  downBytes: number;
  upBytes: number;
}

const BYTES_PER_MB = 1024 * 1024;
const THROUGHPUT_INTERVAL_MS = 1000;

function createConnectionService() {
  let state = $state<ConnectionState>({
    status: 'disconnected',
    activeProxy: null,
    currentIp: null,
    downloadSpeed: 0,
    uploadSpeed: 0,
  });

  let sessionId = $state<string | null>(null);

  let throughputTimer: ReturnType<typeof setInterval> | null = null;
  let lastSample: { down: number; up: number; at: number } | null = null;

  function stopThroughputPolling() {
    if (throughputTimer !== null) {
      clearInterval(throughputTimer);
      throughputTimer = null;
    }
    lastSample = null;
  }

  function startThroughputPolling() {
    stopThroughputPolling();
    throughputTimer = setInterval(async () => {
      try {
        const { downBytes, upBytes } = await invoke<Throughput>('get_throughput');
        const now = Date.now();
        if (lastSample) {
          const seconds = (now - lastSample.at) / 1000;
          if (seconds > 0) {
            const down = Math.max(0, downBytes - lastSample.down) / seconds / BYTES_PER_MB;
            const up = Math.max(0, upBytes - lastSample.up) / seconds / BYTES_PER_MB;
            state = { ...state, downloadSpeed: down, uploadSpeed: up };
          }
        }
        lastSample = { down: downBytes, up: upBytes, at: now };
      } catch {
        // Transient read failure — keep the last known speeds.
      }
    }, THROUGHPUT_INTERVAL_MS);
  }

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
        startThroughputPolling();
        // Reflect the now-active status + refreshed lastUsedAt in the UI.
        proxyService.getProxies().catch(() => {});
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

      stopThroughputPolling();

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

      // Reflect the now-inactive status in the proxy list.
      proxyService.getProxies().catch(() => {});

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
