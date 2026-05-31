<script lang="ts">
  import { onMount } from "svelte";
  import TopAppBar from "@components/layout/TopAppBar.svelte";
  import { sessionService } from "@services/session.service";
  import { proxyService } from "@services/proxy.service.svelte";
  import type { ConnectionSession } from "@types";
  import { History } from "@lucide/svelte";
  import { toast } from "svelte-sonner";

  let sessions = $state<ConnectionSession[]>([]);
  let loading = $state(true);

  function proxyName(proxyId: string | null): string {
    if (!proxyId) return "Unknown proxy";
    return proxyService.proxies.find((p) => p.id === proxyId)?.name ?? "Deleted proxy";
  }

  function formatTimestamp(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function formatDuration(connectedAt: string, disconnectedAt: string | null): string {
    if (!disconnectedAt) return "Active";
    const ms = new Date(disconnectedAt).getTime() - new Date(connectedAt).getTime();
    if (ms < 0) return "—";
    const totalSeconds = Math.floor(ms / 1000);
    const h = Math.floor(totalSeconds / 3600);
    const m = Math.floor((totalSeconds % 3600) / 60);
    const s = totalSeconds % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  }

  onMount(async () => {
    try {
      sessions = await sessionService.getAll();
    } catch (error) {
      toast.error(
        `Failed to load history: ${error instanceof Error ? error.message : String(error)}`,
      );
    } finally {
      loading = false;
    }
  });
</script>

<TopAppBar title="Connection History" searchPlaceholder="Search history..." />

<main class="flex-1 overflow-y-auto p-6">
  <div class="rounded-[2px] border border-zinc-800 bg-zinc-900/50 p-px">
    <div class="overflow-x-auto">
      <table class="w-full min-w-[640px]">
        <thead>
          <tr class="border-b border-zinc-800 bg-zinc-900/80">
            {#each ["PROXY", "CONNECTED", "DISCONNECTED", "DURATION", "IP ADDRESS"] as col}
              <th
                class="px-6 py-4 text-left font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500"
              >
                {col}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each sessions as session (session.id)}
            <tr class="border-b border-zinc-800 last:border-0">
              <td class="px-6 py-4 text-sm font-medium text-white">{proxyName(session.proxyId)}</td>
              <td class="px-6 py-4 font-['Space_Grotesk',sans-serif] text-[13px] text-zinc-300"
                >{formatTimestamp(session.connectedAt)}</td
              >
              <td class="px-6 py-4 font-['Space_Grotesk',sans-serif] text-[13px] text-zinc-300">
                {session.disconnectedAt ? formatTimestamp(session.disconnectedAt) : "—"}
              </td>
              <td class="px-6 py-4 text-sm text-zinc-400"
                >{formatDuration(session.connectedAt, session.disconnectedAt)}</td
              >
              <td class="px-6 py-4 font-['Space_Grotesk',sans-serif] text-[13px] text-zinc-300"
                >{session.ipAddress ?? "—"}</td
              >
            </tr>
          {:else}
            <tr>
              <td colspan="5">
                <div class="flex flex-col items-center justify-center py-16 text-center">
                  <History class="mb-4 h-12 w-12 text-zinc-700" />
                  <p class="font-medium text-white">
                    {loading ? "Loading history…" : "No connection history yet"}
                  </p>
                  {#if !loading}
                    <p class="mt-1 text-sm text-zinc-500">
                      Connect to a proxy to start recording sessions
                    </p>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</main>
