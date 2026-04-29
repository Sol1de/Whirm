<script lang="ts">
  import * as Sheet from '$lib/components/ui/sheet';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as ToggleGroup from '$lib/components/ui/toggle-group';
  import type { ProxyProtocol } from '$lib/types';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let proxyName = $state('');
  let host = $state('');
  let port = $state('');
  let protocol = $state<ProxyProtocol>('SOCKS5');
  let username = $state('');
  let password = $state('');

  function handleSave() {
    // TODO: validate + add proxy via proxyStore
    onClose();
  }

  function handleCancel() {
    onClose();
  }
</script>

<Sheet.Root {open} onOpenChange={(v) => !v && onClose()}>
  <Sheet.Content side="right" class="w-[400px] p-0">
    <!-- Header -->
    <Sheet.Header class="border-b border-border px-6 py-5">
      <Sheet.Title>Add Proxy</Sheet.Title>
      <Sheet.Close />
    </Sheet.Header>

    <!-- Form body -->
    <div class="flex flex-col gap-6 overflow-y-auto px-6 py-6">
      <!-- Proxy Name -->
      <div class="flex flex-col gap-2">
        <Label>Proxy Name</Label>
        <Input bind:value={proxyName} placeholder="e.g. Frankfurt Data Center" />
      </div>

      <!-- Host + Port -->
      <div class="flex gap-4">
        <div class="flex flex-1 flex-col gap-2">
          <Label>Host (IP or Domain)</Label>
          <Input bind:value={host} placeholder="127.0.0.1" />
        </div>
        <div class="flex w-28 flex-col gap-2">
          <Label>Port</Label>
          <Input bind:value={port} placeholder="8080" />
        </div>
      </div>

      <!-- Protocol -->
      <div class="flex flex-col gap-2">
        <Label>Protocol</Label>
        <ToggleGroup.Root type="single" bind:value={protocol} class="justify-start">
          <ToggleGroup.Item value="SOCKS5">SOCKS5</ToggleGroup.Item>
          <ToggleGroup.Item value="HTTP">HTTP</ToggleGroup.Item>
          <ToggleGroup.Item value="HTTPS">HTTPS</ToggleGroup.Item>
        </ToggleGroup.Root>
      </div>

      <!-- Authentication (optional) -->
      <div class="flex flex-col gap-4 border-t border-border pt-4">
        <p class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Authentication (Optional)
        </p>
        <div class="flex flex-col gap-2">
          <Label>Username</Label>
          <Input bind:value={username} />
        </div>
        <div class="flex flex-col gap-2">
          <Label>Password</Label>
          <Input type="password" bind:value={password} />
        </div>
      </div>

      <!-- Info notice -->
      <div class="rounded-md border border-border bg-muted/30 p-4 text-sm text-muted-foreground">
        <!-- Notice text slot -->
      </div>
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-between border-t border-border px-6 py-4">
      <Button class="flex-1" onclick={handleSave}>Save Proxy</Button>
      <Button variant="outline" class="ml-3" onclick={handleCancel}>Cancel</Button>
    </div>
  </Sheet.Content>
</Sheet.Root>
