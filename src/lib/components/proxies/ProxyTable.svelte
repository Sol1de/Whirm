<script lang="ts">
  import { proxyService } from "$lib/services/proxy.service.svelte";
  import ProxyTableRow from "./ProxyTableRow.svelte";
  import { Network, Plus } from "@lucide/svelte";
</script>

<div class="rounded-[2px] border border-zinc-800 bg-zinc-900/50 p-px">
  <div class="overflow-x-auto">
  <table class="w-full min-w-[640px]">
    <!-- Header -->
    <thead>
      <tr class="border-b border-zinc-800 bg-zinc-900/80">
        {#each ["NAME", "HOST:PORT", "PROTOCOL", "REGION", "LATENCY", "ACTIONS"] as col, i}
          <th
            class="px-6 py-4 font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500
              {i === 5 ? 'text-right' : 'text-left'}"
          >
            {col}
          </th>
        {/each}
      </tr>
    </thead>

    <!-- Body -->
    <tbody>
      {#each proxyService.proxies as proxy (proxy.id)}
        <ProxyTableRow {proxy} />
      {:else}
        <tr>
          <td colspan="6">
            <div
              class="flex flex-col items-center justify-center py-16 text-center"
            >
              <Network class="mb-4 h-12 w-12 text-zinc-700" />
              <p class="font-medium text-white">No proxies added</p>
              <p class="mt-1 text-sm text-zinc-500">
                Add your first proxy to get started
              </p>
              <button
                class="mt-4 rounded-[2px] bg-white px-4 py-2 text-sm font-semibold text-black transition-opacity hover:opacity-90"
                onclick={proxyService.openAddSheet}
              >
                Add Proxy
              </button>
            </div>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  </div>
</div>
