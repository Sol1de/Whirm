<script lang="ts">
  import { proxyStore } from '$lib/stores/proxy.svelte';
  import * as Select from '$lib/components/ui/select';
  import { Button } from '$lib/components/ui/button';

  let regionFilter = $state('all');
  let protocolFilter = $state('all');
</script>

<div class="flex items-center gap-3">
  <!-- Select Region -->
  <!-- Select Protocol -->
  <div class="ml-auto">
    <Button onclick={proxyStore.openAddSheet}>
      Add Proxy
    </Button>
  </div>
</div>
