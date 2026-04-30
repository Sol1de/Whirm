<script lang="ts">
  import * as Sheet from "$lib/components/ui/sheet";
  import { Input } from "$lib/components/ui/input";
  import { proxyStore } from "$lib/stores/proxy.svelte";
  import type { ProxyProtocol } from "$lib/types";
  import { X, Info } from "@lucide/svelte";
  import { toast } from "svelte-sonner";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let proxyName = $state("");
  let host = $state("");
  let port = $state("");
  let protocol = $state<ProxyProtocol>("SOCKS5");
  let username = $state("");
  let password = $state("");

  const protocols: ProxyProtocol[] = ["HTTPS", "SOCKS5", "HTTP"];

  function handleSave() {
    if (!proxyName.trim() || !host.trim() || !port.trim()) {
      toast.error("Please fill in all required fields (name, host, port)");
      return;
    }
    const portNum = parseInt(port, 10);
    if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
      toast.error("Port must be a number between 1 and 65535");
      return;
    }
    proxyStore.addProxy({
      id: crypto.randomUUID(),
      name: proxyName.trim(),
      host: host.trim(),
      port: portNum,
      protocol,
      country: "",
      countryCode: "",
      status: "inactive",
      username: username.trim() || undefined,
      password: password.trim() || undefined,
    });
    toast.success("Proxy added");
    // Reset form
    proxyName = "";
    host = "";
    port = "";
    protocol = "SOCKS5";
    username = "";
    password = "";
    onClose();
  }

  function handleTest() {
    toast.info("Connection test not yet implemented");
  }
</script>

<Sheet.Root {open} onOpenChange={(v) => !v && onClose()}>
  <Sheet.Content
    side="right"
    class="flex w-[400px] flex-col border-zinc-800 bg-zinc-900 p-0 [&>button]:hidden"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between border-b border-zinc-800 px-6 py-5"
    >
      <h2 class="text-base font-semibold text-white">Add New Proxy</h2>
      <button
        class="text-zinc-400 transition-colors hover:text-white"
        onclick={onClose}
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <!-- Form body -->
    <div class="flex flex-1 flex-col gap-6 overflow-y-auto px-6 py-6">
      <!-- Proxy Name -->
      <div class="flex flex-col gap-2">
        <label
          class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
          for="add-proxy-name">PROXY NAME</label
        >
        <Input
          name="add-proxy-name"
          bind:value={proxyName}
          placeholder="e.g. Frankfurt Data Center"
          class="rounded-[2px] border-zinc-800 bg-[#09090b] text-white placeholder:text-zinc-600"
        />
      </div>

      <!-- Host + Port -->
      <div class="flex gap-4">
        <div class="flex flex-1 flex-col gap-2">
          <label
            class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            for="add-proxy-host">HOST (IP OR DOMAIN)</label
          >
          <Input
            name="add-proxy-host"
            bind:value={host}
            placeholder="127.0.0.1"
            class="rounded-[2px] border-zinc-800 bg-[#09090b] text-white placeholder:text-zinc-600"
          />
        </div>
        <div class="flex w-28 flex-col gap-2">
          <label
            class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            for="add-proxy-port">PORT</label
          >
          <Input
            name="add-proxy-port"
            bind:value={port}
            placeholder="8080"
            class="rounded-[2px] border-zinc-800 bg-[#09090b] text-white placeholder:text-zinc-600"
          />
        </div>
      </div>

      <!-- Protocol -->
      <div class="flex flex-col gap-2">
        <label
          class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
          for="add-proxy-protocol">PROTOCOL</label
        >
        <div class="flex">
          {#each protocols as p}
            <button
              class="flex-1 border px-3 py-2 text-sm transition-colors first:rounded-l-[2px] last:rounded-r-[2px]
                {protocol === p
                ? 'border-white bg-white text-black font-medium'
                : 'border-zinc-700 bg-zinc-800 text-zinc-300 hover:bg-zinc-700'}"
              onclick={() => (protocol = p)}
            >
              {p}
            </button>
          {/each}
        </div>
      </div>

      <!-- Authentication (optional) -->
      <div class="flex flex-col gap-4 border-t border-zinc-800 pt-4">
        <p
          class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
        >
          AUTHENTICATION (OPTIONAL)
        </p>
        <div class="flex flex-col gap-2">
          <label
            class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            for="add-proxy-username">USERNAME</label
          >
          <Input
            name="add-proxy-username"
            bind:value={username}
            class="rounded-[2px] border-zinc-800 bg-[#09090b] text-white"
          />
        </div>
        <div class="flex flex-col gap-2">
          <label
            class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
            for="add-proxy-password">PASSWORD</label
          >
          <Input
            type="password"
            name="add-proxy-password"
            bind:value={password}
            class="rounded-[2px] border-zinc-800 bg-[#09090b] text-white"
          />
        </div>
      </div>

      <!-- Info box -->
      <div
        class="flex gap-3 rounded-[2px] border border-zinc-800 bg-zinc-800/30 p-4"
      >
        <Info class="mt-0.5 h-4 w-4 shrink-0 text-zinc-500" />
        <p class="text-sm leading-relaxed text-zinc-500">
          Ensure your firewall allows incoming connections from these addresses.
          Test connection after saving.
        </p>
      </div>
    </div>

    <!-- Footer -->
    <div class="flex gap-3 border-t border-zinc-800 px-6 py-4">
      <button
        class="flex-1 rounded-[2px] bg-white py-2 text-sm font-semibold text-black transition-opacity hover:opacity-90"
        onclick={handleSave}
      >
        Save Proxy
      </button>
      <button
        class="rounded-[2px] border border-zinc-700 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-800"
        onclick={handleTest}
      >
        Test
      </button>
    </div>
  </Sheet.Content>
</Sheet.Root>
