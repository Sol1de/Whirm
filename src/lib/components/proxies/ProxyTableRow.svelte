<script lang="ts">
  import type { Proxy } from '$lib/types';
  import * as Table from '$lib/components/ui/table';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    proxy: Proxy;
    onEdit?: (proxy: Proxy) => void;
    onDelete?: (id: string) => void;
  }

  let { proxy, onEdit, onDelete }: Props = $props();
</script>

<Table.Row>
  <!-- Name | Host:Port | Protocol (badge) | Country (flag + text) | Speed | Actions (3 buttons) -->
</Table.Row>
