<script lang="ts">
  import TopAppBar from '$lib/components/layout/TopAppBar.svelte';
  import ConnectionCard from '$lib/components/dashboard/ConnectionCard.svelte';
  import QuickStats from '$lib/components/dashboard/QuickStats.svelte';
  import RecentProxiesTable from '$lib/components/dashboard/RecentProxiesTable.svelte';
</script>

<TopAppBar title="Dashboard" searchPlaceholder="Search parameters..." />

<main class="flex-1 overflow-y-auto p-6 space-y-6">
  <ConnectionCard />
  <QuickStats />
  <RecentProxiesTable />
</main>
