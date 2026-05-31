<script lang="ts">
  import { onMount } from 'svelte';
  import { navigation } from '@stores/navigation.svelte';
  import Sidebar from '@components/layout/Sidebar.svelte';
  import DashboardPage from '@pages/DashboardPage.svelte';
  import ProxiesPage from '@pages/ProxiesPage.svelte';
  import HistoryPage from '@pages/HistoryPage.svelte';
  import SettingsPage from '@pages/SettingsPage.svelte';
  import { Toaster } from '@ui/sonner';
  import { toast } from 'svelte-sonner';
  import { proxyService } from '@services/proxy.service.svelte';
  import { settingsService } from '@services/settings.service.svelte';

  onMount(() => {
    proxyService.getProxies().catch((e: unknown) => {
      toast.error(`Failed to load proxies: ${e instanceof Error ? e.message : String(e)}`);
    });
    settingsService.getSettings().catch((e: unknown) => {
      toast.error(`Failed to load settings: ${e instanceof Error ? e.message : String(e)}`);
    });
  });
</script>

<div class="flex h-screen w-screen overflow-hidden bg-background">
  <Sidebar />

  <div class="flex flex-1 flex-col overflow-hidden">
    {#if navigation.current === 'dashboard'}
      <DashboardPage />
    {:else if navigation.current === 'proxies'}
      <ProxiesPage />
    {:else if navigation.current === 'history'}
      <HistoryPage />
    {:else if navigation.current === 'settings'}
      <SettingsPage />
    {/if}
  </div>
</div>

<Toaster />
