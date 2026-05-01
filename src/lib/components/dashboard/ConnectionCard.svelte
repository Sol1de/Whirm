<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { proxyStore } from "$lib/stores/proxy.svelte";
  import { toast } from "svelte-sonner";
  import { CodeXml } from "@lucide/svelte";

  const statusConfig = {
    connected: {
      dot: "bg-emerald-500",
      overlay: "bg-emerald-500/20",
      label: "Connected",
      subtitle: "Traffic routing through proxy",
      subtitleColor: "text-emerald-500",
    },
    connecting: {
      dot: "bg-yellow-500",
      overlay: "bg-yellow-500/20",
      label: "Connecting…",
      subtitle: "Establishing tunnel…",
      subtitleColor: "text-yellow-500",
    },
    disconnected: {
      dot: "bg-zinc-500",
      overlay: "bg-zinc-500/20",
      label: "Disconnected",
      subtitle: "No active tunnel",
      subtitleColor: "text-zinc-500",
    },
  };

  let config = $derived(
    statusConfig[connectionStore.state.status] ?? statusConfig.disconnected,
  );
  let isConnected = $derived(connectionStore.state.status === "connected");
  let isConnecting = $derived(connectionStore.state.status === "connecting");

  let targetProxy = $derived(
    connectionStore.state.activeProxy ?? proxyStore.proxies[0] ?? null,
  );

  let isDisabled = $derived(
    isConnecting || (!isConnected && targetProxy === null),
  );

  let proxyLabel = $derived(targetProxy?.name ?? "No proxy configured");

  async function handleToggle() {
    if (isConnected) {
      try {
        await connectionStore.disconnect();
      } catch (error) {
        toast.error(`Disconnect failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    } else if (targetProxy) {
      try {
        await connectionStore.connect(targetProxy.id);
      } catch (error) {
        toast.error(`Connection failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    }
  }
</script>

<div
  class="flex items-center justify-between rounded-[2px] border border-zinc-800 bg-zinc-900 p-[33px]"
>
  <!-- Left: indicator + text -->
  <div class="flex items-center gap-6">
    <!-- Status dot with glow overlay -->
    <div class="relative flex items-center justify-center">
      <div class="absolute h-12 w-12 rounded-[2px] {config.overlay}"></div>
      <div
        class="relative h-4 w-4 rounded-[2px] border-2 border-zinc-900 {config.dot}"
      ></div>
    </div>

    <!-- Text -->
    <div class="flex flex-col gap-1">
      <p
        class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[1.2px] text-zinc-500"
      >
        TUNNEL STATUS
      </p>
      <p
        class="font-['Space_Grotesk',sans-serif] text-2xl font-medium text-white"
      >
        {config.label}
      </p>
      <p class="text-sm {config.subtitleColor}">{config.subtitle}</p>
    </div>
  </div>

  <!-- Right: buttons -->
  <div class="flex items-center gap-3">
    <span class="text-sm text-zinc-400">{proxyLabel}</span>
    <button
      class="rounded-[2px] bg-white px-5 py-2 text-sm font-semibold text-black transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-40"
      disabled={isDisabled}
      onclick={handleToggle}
    >
      {isConnected
        ? "⏻  DISCONNECT"
        : isConnecting
          ? "⏻  CONNECTING…"
          : "⏻  CONNECT"}
    </button>
    <button
      class="rounded-[2px] border border-zinc-800 p-2 text-zinc-400 transition-colors hover:text-white"
    >
      <CodeXml class="h-4 w-4" />
    </button>
  </div>
</div>
