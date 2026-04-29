<script lang="ts">
  import TopAppBar from '$lib/components/layout/TopAppBar.svelte';
  import ProxyFiltersBar from '$lib/components/proxies/ProxyFiltersBar.svelte';
  import ProxyTable from '$lib/components/proxies/ProxyTable.svelte';
  import AddProxySheet from '$lib/components/proxies/AddProxySheet.svelte';
  import { proxyStore } from '$lib/stores/proxy.svelte';
</script>

<TopAppBar title="Proxies" searchPlaceholder="Search saved proxies..." />

<main class="flex-1 overflow-y-auto p-6 space-y-4">
  <ProxyFiltersBar />
  <ProxyTable />
</main>

<AddProxySheet open={proxyStore.isAddSheetOpen} onClose={proxyStore.closeAddSheet} />
