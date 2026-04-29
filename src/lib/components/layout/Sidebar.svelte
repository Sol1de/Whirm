<script lang="ts">
  import { navigation } from '$lib/stores/navigation.svelte';
  import { connectionStore } from '$lib/stores/connection.svelte';
  import type { Route } from '$lib/types';

  const navItems: { route: Route; label: string }[] = [
    { route: 'dashboard', label: 'Dashboard' },
    { route: 'proxies', label: 'Proxies' },
    { route: 'settings', label: 'Settings' }
  ];
</script>

<aside class="flex h-full w-60 shrink-0 flex-col border-r border-border bg-sidebar">
  <!-- Logo -->
  <div class="flex h-14 items-center px-4">
    <!-- Logo ProxyShift -->
  </div>

  <!-- Nav links -->
  <nav class="flex flex-1 flex-col gap-1 px-2 py-4">
    {#each navItems as item}
      <button
        class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors
          {navigation.current === item.route
          ? 'bg-sidebar-accent text-sidebar-accent-foreground'
          : 'text-sidebar-foreground hover:bg-sidebar-accent/50'}"
        onclick={() => navigation.navigate(item.route)}
      >
        <!-- Icon slot -->
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Connection status -->
  <div class="border-t border-border px-4 py-3">
    <div class="flex items-center gap-2 text-sm">
      <span
        class="h-2 w-2 rounded-full {connectionStore.state.status === 'connected'
          ? 'bg-green-500'
          : connectionStore.state.status === 'connecting'
            ? 'bg-yellow-500'
            : 'bg-muted-foreground'}"
      ></span>
      <span class="text-muted-foreground capitalize">{connectionStore.state.status}</span>
    </div>
  </div>
</aside>
