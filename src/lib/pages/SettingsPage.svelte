<script lang="ts">
  import TopAppBar from "$lib/components/layout/TopAppBar.svelte";
  import ConnectionSettings from "$lib/components/settings/ConnectionSettings.svelte";
  import SettingsFooter from "$lib/components/settings/SettingsFooter.svelte";
  import { settingsService } from "$lib/services/settings.service.svelte";
  import { toast } from "svelte-sonner";

  async function handleSave() {
    try {
      await settingsService.save();
      toast.success("Settings saved");
    } catch {
      toast.error("Failed to save settings");
    }
  }

  async function handleReset() {
    try {
      await settingsService.reset();
      toast.info("Settings reset to defaults");
    } catch {
      toast.error("Failed to reset settings");
    }
  }
</script>

<TopAppBar title="System Settings" searchPlaceholder="Search parameters..." />

<main class="relative flex flex-1 flex-col overflow-y-auto p-8">
  <ConnectionSettings />
</main>

<SettingsFooter onSave={handleSave} onReset={handleReset} />
