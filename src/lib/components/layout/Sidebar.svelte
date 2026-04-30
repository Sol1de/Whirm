<script lang="ts">
  import { navigation } from '$lib/stores/navigation.svelte';
  import { connectionStore } from '$lib/stores/connection.svelte';
  import { proxyStore } from '$lib/stores/proxy.svelte';
  import type { Route } from '$lib/types';
  import { LayoutDashboard, Globe, Settings2 } from '@lucide/svelte';
  import type { Component } from 'svelte';

  const navItems: { route: Route; label: string; icon: Component }[] = [
    { route: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { route: 'proxies', label: 'Proxies', icon: Globe },
    { route: 'settings', label: 'Settings', icon: Settings2 }
  ];

  const statusDot: Record<string, string> = {
    connected: 'bg-emerald-500',
    connecting: 'bg-yellow-500',
    disconnected: 'bg-zinc-500'
  };
</script>

<aside class="flex h-full w-[240px] shrink-0 flex-col border-r border-zinc-800 bg-zinc-900">
  <!-- Logo -->
  <div class="flex h-[76px] items-center px-6">
    <span class="font-['Space_Grotesk',sans-serif] text-xl font-bold text-white">ProxyShift</span>
  </div>

  <!-- Navigation -->
  <nav class="flex flex-1 flex-col gap-1 px-4 py-4">
    {#each navItems as item}
      {@const active = navigation.current === item.route}
      <button
        class="flex w-full items-center gap-3 rounded-[2px] py-2 text-sm transition-colors
          {active
          ? 'border-l-2 border-white bg-zinc-800 pl-[14px] pr-3 text-white'
          : 'px-3 text-zinc-500 hover:bg-zinc-800/50 hover:text-zinc-300'}"
        onclick={() => navigation.navigate(item.route)}
      >
        <item.icon class="h-4 w-4 shrink-0" />
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Connection status -->
  <div class="border-t border-zinc-800 px-6 py-4">
    <div class="flex items-center gap-3">
      <span
        class="h-2 w-2 shrink-0 rounded-full {statusDot[connectionStore.state.status] ??
          'bg-zinc-500'}"
      ></span>
      <span class="text-sm capitalize text-zinc-400">{connectionStore.state.status}</span>
    </div>
  </div>
</aside>
