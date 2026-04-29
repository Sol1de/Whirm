<script lang="ts">
  import { navigation } from '$lib/stores/navigation.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import DashboardPage from '$lib/pages/DashboardPage.svelte';
  import ProxiesPage from '$lib/pages/ProxiesPage.svelte';
  import SettingsPage from '$lib/pages/SettingsPage.svelte';
  import { Toaster } from '$lib/components/ui/sonner';
</script>

<div class="flex h-screen w-screen overflow-hidden bg-background">
  <Sidebar />

  <div class="flex flex-1 flex-col overflow-hidden">
    {#if navigation.current === 'dashboard'}
      <DashboardPage />
    {:else if navigation.current === 'proxies'}
      <ProxiesPage />
    {:else if navigation.current === 'settings'}
      <SettingsPage />
    {/if}
  </div>
</div>

<Toaster />
