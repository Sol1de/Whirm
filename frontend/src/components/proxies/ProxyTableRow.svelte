<script lang="ts">
  import type { Proxy } from '@types';
  import { Plug2, Pencil, Trash2 } from '@lucide/svelte';
  import { proxyService } from '@services/proxy.service.svelte';
  import { connectionService } from '@services/connection.service.svelte';
  import { toast } from 'svelte-sonner';

  interface Props {
    proxy: Proxy;
  }

  let { proxy }: Props = $props();

  function countryCodeToFlag(code: string): string {
    if (!code || code.length !== 2) return '';
    return code
      .toUpperCase()
      .split('')
      .map((c) => String.fromCodePoint(127397 + c.charCodeAt(0)))
      .join('');
  }

  let isActive = $derived(connectionService.state.activeProxy?.id === proxy.id);
  let isConnecting = $derived(connectionService.state.status === 'connecting');

  async function handleDelete() {
    try {
      await proxyService.remove(proxy.id);
    } catch (error) {
      toast.error(`Failed to delete proxy: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async function handleConnect() {
    if (isActive) {
      try {
        await connectionService.disconnect();
      } catch (error) {
        toast.error(`Disconnect failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    } else {
      try {
        await connectionService.connect(proxy.id);
        toast.success(`Connected to ${proxy.name}`);
      } catch (error) {
        toast.error(`Failed to connect to ${proxy.name}: ${error instanceof Error ? error.message : String(error)}`);
      }
    }
  }
</script>

<tr class="border-b border-zinc-800 last:border-0">
  <!-- Name -->
  <td class="px-6 py-[22.5px]">
    <span class="text-base font-medium text-white" style="font-family: Inter, sans-serif;"
      >{proxy.name}</span
    >
  </td>

  <!-- Host:Port -->
  <td class="px-6 py-[22.5px]">
    <span class="font-['Space_Grotesk',sans-serif] text-[13px] text-zinc-300"
      >{proxy.host}:{proxy.port}</span
    >
  </td>

  <!-- Protocol badge -->
  <td class="px-6 py-[22.5px]">
    <span
      class="rounded-[2px] border border-zinc-700 bg-zinc-800 px-2 py-0.5 text-xs text-zinc-300"
      >{proxy.protocol}</span
    >
  </td>

  <!-- Region -->
  <td class="px-6 py-[22.5px]">
    <div class="flex items-center gap-2">
      <span class="text-base">{countryCodeToFlag(proxy.countryCode)}</span>
      <span class="text-sm text-zinc-400">{proxy.country}</span>
    </div>
  </td>

  <!-- Latency -->
  <td class="px-6 py-[22.5px]">
    <span class="text-sm text-zinc-400">—</span>
  </td>

  <!-- Actions -->
  <td class="px-6 py-[22.5px]">
    <div class="flex items-center justify-end gap-2">
      <button
        class="transition-colors disabled:cursor-not-allowed disabled:opacity-40 {isActive
          ? 'text-emerald-500 hover:text-emerald-400'
          : 'text-zinc-500 hover:text-white'}"
        title={isActive ? 'Disconnect' : 'Connect'}
        disabled={isConnecting}
        onclick={handleConnect}
      >
        <Plug2 class="h-4 w-4" />
      </button>
      <button class="text-zinc-500 transition-colors hover:text-white" title="Edit">
        <Pencil class="h-4 w-4" />
      </button>
      <button
        class="text-zinc-500 transition-colors hover:text-red-400"
        title="Delete"
        onclick={handleDelete}
      >
        <Trash2 class="h-4 w-4" />
      </button>
    </div>
  </td>
</tr>
