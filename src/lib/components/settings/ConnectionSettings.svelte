<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import * as Select from '$lib/components/ui/select';

  let port = $state(5000);
  let enabled = $state(true);
  let protocol = $state('SOCKS5');
</script>

<section class="rounded-lg border border-border bg-card">
  <div class="flex items-center gap-2 border-b border-border px-6 py-4">
    <!-- Section icon -->
    <h2 class="font-medium">Connection Settings</h2>
  </div>

  <div class="divide-y divide-border">
    <!-- Port row -->
    <div class="flex items-center justify-between px-6 py-5">
      <div>
        <!-- Label + description slot -->
      </div>
      <Input type="number" bind:value={port} class="w-32 text-right" />
    </div>

    <!-- Enable toggle row -->
    <div class="flex items-center justify-between px-6 py-5">
      <div>
        <!-- Label + description slot -->
      </div>
      <Switch bind:checked={enabled} />
    </div>

    <!-- Protocol row -->
    <div class="flex items-center justify-between px-6 py-5">
      <div>
        <!-- Label + description slot -->
      </div>
      <!-- Select SOCKS5 / HTTP / HTTPS slot -->
    </div>
  </div>
</section>
