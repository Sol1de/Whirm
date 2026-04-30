<script lang="ts">
  import TopAppBar from "$lib/components/layout/TopAppBar.svelte";
  import ConnectionSettings from "$lib/components/settings/ConnectionSettings.svelte";
  import SettingsFooter from "$lib/components/settings/SettingsFooter.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { toast } from "svelte-sonner";

  function handleSave() {
    settingsStore.save();
    toast.success("Settings saved");
  }

  function handleReset() {
    settingsStore.reset();
    toast.info("Settings reset to defaults");
  }
</script>

<TopAppBar title="System Settings" searchPlaceholder="Search parameters..." />

<main class="relative flex flex-1 flex-col overflow-y-auto p-8">
  <ConnectionSettings />
</main>

<SettingsFooter onSave={handleSave} onReset={handleReset} />
