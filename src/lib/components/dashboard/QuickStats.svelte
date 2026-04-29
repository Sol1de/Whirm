<script lang="ts">
  import StatCard from './StatCard.svelte';
  import { connectionStore } from '$lib/stores/connection.svelte';
</script>

<div class="grid grid-cols-3 gap-4">
  <StatCard title="Current IP" />
  <StatCard title="Network Speed" />
  <StatCard title="Active Proxy" />
</div>
