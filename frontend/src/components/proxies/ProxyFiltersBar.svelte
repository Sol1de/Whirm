<script lang="ts">
  import { proxyService } from '@services/proxy.service.svelte';
  import * as Select from '@ui/select';
  import { Plus } from '@lucide/svelte';

  let regionFilter = $state('all');
  let protocolFilter = $state('all');
</script>

<div class="flex items-end justify-between">
  <!-- Filters -->
  <div class="flex items-end gap-3">
    <!-- Region filter -->
    <div class="flex flex-col gap-1">
      <span
        class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
        >REGION</span
      >
      <Select.Root type="single" bind:value={regionFilter}>
        <Select.Trigger
          class="h-9 w-auto rounded-[2px] border-zinc-800 bg-zinc-900 text-sm text-white"
        >
          {regionFilter === 'all' ? 'All Regions' : regionFilter}
        </Select.Trigger>
        <Select.Content class="rounded-[2px] border-zinc-800 bg-zinc-900">
          <Select.Item value="all" class="text-sm text-white hover:bg-zinc-800">All Regions</Select.Item>
          <Select.Item value="us" class="text-sm text-white hover:bg-zinc-800">United States</Select.Item>
          <Select.Item value="eu" class="text-sm text-white hover:bg-zinc-800">Europe</Select.Item>
          <Select.Item value="asia" class="text-sm text-white hover:bg-zinc-800">Asia</Select.Item>
        </Select.Content>
      </Select.Root>
    </div>

    <!-- Protocol filter -->
    <div class="flex flex-col gap-1">
      <span
        class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
        >PROTOCOL</span
      >
      <Select.Root type="single" bind:value={protocolFilter}>
        <Select.Trigger
          class="h-9 w-auto rounded-[2px] border-zinc-800 bg-zinc-900 text-sm text-white"
        >
          {protocolFilter === 'all' ? 'All Protocols' : protocolFilter}
        </Select.Trigger>
        <Select.Content class="rounded-[2px] border-zinc-800 bg-zinc-900">
          <Select.Item value="all" class="text-sm text-white hover:bg-zinc-800">All Protocols</Select.Item>
          <Select.Item value="SOCKS5" class="text-sm text-white hover:bg-zinc-800">SOCKS5</Select.Item>
          <Select.Item value="HTTP" class="text-sm text-white hover:bg-zinc-800">HTTP</Select.Item>
          <Select.Item value="HTTPS" class="text-sm text-white hover:bg-zinc-800">HTTPS</Select.Item>
        </Select.Content>
      </Select.Root>
    </div>
  </div>

  <!-- Add Proxy button -->
  <button
    class="flex items-center gap-2 rounded-[2px] bg-white px-4 py-2 font-bold text-[#09090b] transition-opacity hover:opacity-90"
    onclick={proxyService.openAddSheet}
  >
    <Plus class="h-4 w-4" />
    Add Proxy
  </button>
</div>
