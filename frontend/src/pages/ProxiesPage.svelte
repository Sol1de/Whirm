<script lang="ts">
  import TopAppBar from '@components/layout/TopAppBar.svelte';
  import ProxyFiltersBar from '@components/proxies/ProxyFiltersBar.svelte';
  import ProxyTable from '@components/proxies/ProxyTable.svelte';
  import AddProxySheet from '@components/proxies/AddProxySheet.svelte';
  import { proxyService } from '@services/proxy.service.svelte';
</script>

<TopAppBar title="Proxies" searchPlaceholder="Search saved proxies..." />

<main class="flex-1 overflow-y-auto p-6 space-y-4">
  <ProxyFiltersBar />
  <ProxyTable />
</main>

<AddProxySheet open={proxyService.isAddSheetOpen} onClose={proxyService.closeAddSheet} />
