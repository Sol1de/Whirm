<script lang="ts">
  import { connectionService } from '@services/connection.service.svelte';
  import StatCard from './StatCard.svelte';
  import { Globe, Activity, Server } from '@lucide/svelte';

  let state = $derived(connectionService.state);

  let throughputValue = $derived(
    state.downloadSpeed > 0 || state.uploadSpeed > 0
      ? `${state.downloadSpeed.toFixed(1)} MB/S`
      : '—'
  );

  let throughputSubtitle = $derived(
    state.uploadSpeed > 0 ? `${state.uploadSpeed.toFixed(1)} MB/S upload` : undefined
  );
</script>

<div class="grid grid-cols-3 gap-4">
  <StatCard
    label="CURRENT IP"
    value={state.currentIp ?? '—'}
    subtitle={state.activeProxy?.country}
    icon={Globe}
  />
  <StatCard
    label="THROUGHPUT"
    value={throughputValue}
    subtitle={throughputSubtitle}
    icon={Activity}
  />
  <StatCard
    label="ACTIVE PROXY"
    value={state.activeProxy?.name ?? '—'}
    subtitle={state.activeProxy?.category}
    icon={Server}
  />
</div>
