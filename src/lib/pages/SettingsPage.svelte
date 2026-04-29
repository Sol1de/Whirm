<script lang="ts">
  import TopAppBar from '$lib/components/layout/TopAppBar.svelte';
  import ConnectionSettings from '$lib/components/settings/ConnectionSettings.svelte';
  import SettingsFooter from '$lib/components/settings/SettingsFooter.svelte';
</script>

<TopAppBar title="Settings" searchPlaceholder="Search parameters..." />

<main class="flex flex-1 flex-col overflow-y-auto p-6">
  <ConnectionSettings />
</main>

<SettingsFooter />
