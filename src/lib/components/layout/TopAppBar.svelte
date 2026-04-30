<script lang="ts">
  import { Search, Plus, RotateCw, Bell } from '@lucide/svelte';
  import { navigation } from '$lib/stores/navigation.svelte';
  import { proxyStore } from '$lib/stores/proxy.svelte';
  import { toast } from 'svelte-sonner';

  interface Props {
    title: string;
    searchPlaceholder?: string;
  }

  let { title, searchPlaceholder = 'Search...' }: Props = $props();

  let searchQuery = $state('');

  function handleAdd() {
    navigation.navigate('proxies');
    proxyStore.openAddSheet();
  }

  function handleRefresh() {
    toast.success('Refreshed');
  }

  function handleBell() {
    toast.info('No new notifications');
  }
</script>

<header
  class="flex h-[56px] shrink-0 items-center justify-between border-b border-zinc-800 bg-[#09090b] px-6"
>
  <!-- Title -->
  <h1
    class="text-lg font-semibold tracking-[-0.18px] text-white"
    style="font-family: Inter, sans-serif;"
  >
    {title}
  </h1>

  <!-- Right zone -->
  <div class="flex items-center gap-6">
    <!-- Search -->
    <div class="relative">
      <Search class="absolute left-3 top-1/2 h-[10.5px] w-[10.5px] -translate-y-1/2 text-zinc-500" />
      <input
        type="text"
        placeholder={searchPlaceholder}
        bind:value={searchQuery}
        class="h-9 w-64 rounded-[2px] border border-zinc-800 bg-[#09090b] pl-9 pr-3 text-sm text-white placeholder:text-[#6b7280] focus:outline-none focus:ring-1 focus:ring-zinc-600"
      />
    </div>

    <!-- Action buttons -->
    <div class="flex items-center gap-4">
      <button
        class="text-zinc-400 transition-colors hover:text-white"
        onclick={handleAdd}
        title="Add proxy"
      >
        <Plus class="h-5 w-5" />
      </button>

      <button
        class="text-zinc-400 transition-colors hover:text-white"
        onclick={handleRefresh}
        title="Refresh"
      >
        <RotateCw class="h-4 w-4" />
      </button>

      <button
        class="relative text-zinc-400 transition-colors hover:text-white"
        onclick={handleBell}
        title="Notifications"
      >
        <Bell class="h-4 w-4" />
        <span class="absolute -right-0.5 -top-0.5 h-1.5 w-1.5 rounded-full bg-violet-300"></span>
      </button>
    </div>
  </div>
</header>
