<script lang="ts">
  import { Network } from "@lucide/svelte";
  import { Switch } from "@ui/switch";
  import * as Select from "@ui/select";
  import { settingsService } from "@services/settings.service.svelte";
</script>

<section class="rounded-[2px] border border-zinc-800 bg-zinc-900">
  <!-- Section header -->
  <div class="flex items-center gap-2 border-b border-zinc-800 px-6 py-4">
    <Network class="h-4 w-4 text-zinc-400" />
    <span
      class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-400"
      >CONNECTION PROTOCOL</span
    >
  </div>

  <div class="divide-y divide-zinc-800">
    <!-- Row 1: Global Timeout -->
    <div class="flex items-center justify-between px-6 py-6">
      <div class="flex flex-col gap-1">
        <p class="text-sm text-white" style="font-family: Inter, sans-serif;">
          Global Timeout
        </p>
        <p class="text-xs text-zinc-500">
          Maximum latency allowed before session drop (ms)
        </p>
      </div>
      <input
        type="number"
        value={settingsService.draft.globalTimeout}
        oninput={(e) =>
          settingsService.update({
            globalTimeout:
              parseInt((e.target as HTMLInputElement).value, 10) || 0,
          })}
        class="w-32 rounded-[2px] border border-zinc-800 bg-[#09090b] px-3 py-2 text-right font-['Space_Grotesk',sans-serif] text-sm text-white focus:outline-none focus:ring-1 focus:ring-zinc-600"
      />
    </div>

    <!-- Row 2: DNS Leak Protection -->
    <div class="flex items-center justify-between px-6 py-6">
      <div class="flex flex-col gap-1">
        <p class="text-sm text-white" style="font-family: Inter, sans-serif;">
          Remote DNS Resolution
        </p>
        <p class="text-xs text-zinc-500">
          Resolve hostnames at the upstream proxy instead of locally for relayed
          traffic. Not full OS-level DNS-leak protection.
        </p>
      </div>
      <Switch
        checked={settingsService.draft.dnsLeakProtection}
        onCheckedChange={(v) => settingsService.update({ dnsLeakProtection: v })}
      />
    </div>

    <!-- Row 3: Proxy Protocol -->
    <div class="flex items-center justify-between px-6 py-6">
      <div class="flex flex-col gap-1">
        <p class="text-sm text-white" style="font-family: Inter, sans-serif;">
          Proxy Protocol
        </p>
        <p class="text-xs text-zinc-500">
          Select the primary tunneling architecture
        </p>
      </div>
      <Select.Root
        type="single"
        value={settingsService.draft.proxyProtocol}
        onValueChange={(v) =>
          v &&
          settingsService.update({
            proxyProtocol: v as "SOCKS5" | "HTTP" | "HTTPS",
          })}
      >
        <Select.Trigger
          class="w-48 rounded-[2px] border-zinc-800 bg-[#09090b] text-sm text-white"
        >
          {settingsService.draft.proxyProtocol === "SOCKS5"
            ? "SOCKS5"
            : settingsService.draft.proxyProtocol}
        </Select.Trigger>
        <Select.Content class="rounded-[2px] border-zinc-800 bg-zinc-900">
          <Select.Item value="SOCKS5" class="text-sm text-white"
            >SOCKS5 (Recommended)</Select.Item
          >
          <Select.Item value="HTTP" class="text-sm text-white">HTTP</Select.Item
          >
          <Select.Item value="HTTPS" class="text-sm text-white"
            >HTTPS</Select.Item
          >
        </Select.Content>
      </Select.Root>
    </div>
  </div>
</section>
