<script lang="ts">
  import { proxyService } from '@services/proxy.service.svelte';
  import { navigation } from '@stores/navigation.svelte';
  import { connectionService } from '@services/connection.service.svelte';
  import { toast } from 'svelte-sonner';

  let recentProxies = $derived(
    [...proxyService.proxies]
      .filter((p) => p.lastUsedAt != null)
      .sort((a, b) => new Date(b.lastUsedAt!).getTime() - new Date(a.lastUsedAt!).getTime())
      .slice(0, 5)
  );
  let isConnecting = $derived(connectionService.state.status === 'connecting');

  async function handleConnect(proxyId: string) {
    try {
      await connectionService.connect(proxyId);
    } catch (err: unknown) {
      toast.error(`Connection failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }
</script>

<div class="rounded-[2px] border border-zinc-800 bg-zinc-900">
  <!-- Header -->
  <div class="flex items-center justify-between border-b border-zinc-800 px-6 py-4">
    <h3
      class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-400"
    >
      RECENT PROXIES
    </h3>
    <button
      class="text-xs text-zinc-500 transition-colors hover:text-white"
      onclick={() => navigation.navigate('history')}
    >
      View All History ›
    </button>
  </div>

  {#if recentProxies.length === 0}
    <!-- Empty state -->
    <div class="flex flex-col items-center justify-center py-12 text-center">
      <p class="text-sm text-zinc-500">No proxies added yet</p>
      <p class="mt-1 text-xs text-zinc-600">Add your first proxy to see it here</p>
    </div>
  {:else}
    <!-- Table -->
    <table class="w-full">
      <thead>
        <tr class="border-b border-zinc-800">
          <th
            class="px-6 py-3 text-left font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            >NODE NAME</th
          >
          <th
            class="px-6 py-3 text-left font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            >HOST/ENDPOINT</th
          >
          <th
            class="px-6 py-3 text-left font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            >TYPE</th
          >
          <th
            class="px-6 py-3 text-right font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            >ACTION</th
          >
        </tr>
      </thead>
      <tbody>
        {#each recentProxies as proxy (proxy.id)}
          <tr class="border-b border-zinc-800/50 last:border-0">
            <td class="px-6 py-4">
              <div class="flex items-center gap-3">
                <span class="h-1.5 w-1.5 shrink-0 rounded-full bg-zinc-500"></span>
                <span class="text-sm text-white">{proxy.name}</span>
              </div>
            </td>
            <td class="px-6 py-4 font-mono text-xs text-zinc-400"
              >{proxy.host}:{proxy.port}</td
            >
            <td class="px-6 py-4">
              <span
                class="rounded-[2px] border border-zinc-700 bg-zinc-800 px-2 py-0.5 text-xs text-zinc-300"
                >{proxy.protocol}</span
              >
            </td>
            <td class="px-6 py-4 text-right">
              <button
                class="text-xs font-medium tracking-wide text-zinc-400 transition-colors hover:text-white disabled:cursor-not-allowed disabled:opacity-40"
                disabled={isConnecting}
                onclick={() => handleConnect(proxy.id)}
              >
                RECONNECT
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
