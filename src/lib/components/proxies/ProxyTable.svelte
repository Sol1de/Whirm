<script lang="ts">
  import { proxyStore } from '$lib/stores/proxy.svelte';
  import ProxyTableRow from './ProxyTableRow.svelte';
  import * as Table from '$lib/components/ui/table';
</script>

<div class="rounded-lg border border-border bg-card">
  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head>Name</Table.Head>
        <Table.Head>Host : Port</Table.Head>
        <Table.Head>Protocol</Table.Head>
        <Table.Head>Country</Table.Head>
        <Table.Head>Speed</Table.Head>
        <Table.Head class="text-right">Actions</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each proxyStore.proxies as proxy (proxy.id)}
        <ProxyTableRow {proxy} />
      {/each}
    </Table.Body>
  </Table.Root>
</div>
