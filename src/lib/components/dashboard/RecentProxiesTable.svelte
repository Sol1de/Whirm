<script lang="ts">
  import * as Table from '$lib/components/ui/table';
  import { Badge } from '$lib/components/ui/badge';
</script>

<div class="rounded-lg border border-border bg-card">
  <div class="flex items-center justify-between border-b border-border px-6 py-4">
    <h3 class="font-medium">Recent Proxies</h3>
    <!-- View all link -->
  </div>

  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head>Proxy Name</Table.Head>
        <Table.Head>Last Used</Table.Head>
        <Table.Head>Status</Table.Head>
        <Table.Head class="text-right">Actions</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      <!-- Rows will be populated during design integration -->
    </Table.Body>
  </Table.Root>
</div>
