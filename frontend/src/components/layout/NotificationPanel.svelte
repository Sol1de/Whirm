<script lang="ts">
  import { notificationService } from '@services/notification.service.svelte';
  import { CheckCheck, Info, AlertCircle, AlertTriangle, CheckCircle2, X } from '@lucide/svelte';
  import type { AppNotification } from '@types';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  const iconMap: Record<AppNotification['type'], typeof Info> = {
    info: Info,
    success: CheckCircle2,
    error: AlertCircle,
    warning: AlertTriangle,
  };

  const colorMap: Record<AppNotification['type'], string> = {
    info: 'text-zinc-400',
    success: 'text-emerald-500',
    error: 'text-red-400',
    warning: 'text-yellow-400',
  };

  function formatTime(iso: string): string {
    const diff = Date.now() - new Date(iso).getTime();
    const s = Math.floor(diff / 1000);
    if (s < 60) return `${s}s ago`;
    const m = Math.floor(s / 60);
    if (m < 60) return `${m}m ago`;
    const h = Math.floor(m / 60);
    return `${h}h ago`;
  }
</script>

<!-- Click-outside overlay -->
<div class="fixed inset-0 z-40" role="presentation" onclick={onClose}></div>

<div class="relative z-50 w-80 rounded-[2px] border border-zinc-800 bg-zinc-900 shadow-xl">
  <!-- Header -->
  <div class="flex items-center justify-between border-b border-zinc-800 px-4 py-3">
    <span class="font-['Space_Grotesk',sans-serif] text-xs font-medium uppercase tracking-[0.6px] text-zinc-500">
      Notifications
    </span>
    <div class="flex items-center gap-3">
      {#if notificationService.unreadCount > 0}
        <button
          class="flex items-center gap-1 text-xs text-zinc-400 transition-colors hover:text-white"
          onclick={notificationService.markAllRead}
        >
          <CheckCheck class="h-3 w-3" />
          Mark all read
        </button>
      {/if}
      <button
        class="text-zinc-500 transition-colors hover:text-white"
        onclick={onClose}
      >
        <X class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>

  <!-- List -->
  <div class="max-h-80 overflow-y-auto">
    {#if notificationService.notifications.length === 0}
      <div class="flex flex-col items-center justify-center py-10 text-center">
        <p class="text-sm text-zinc-500">No notifications yet</p>
      </div>
    {:else}
      {#each notificationService.notifications as notif (notif.id)}
        {@const Icon = iconMap[notif.type]}
        <div
          class="flex gap-3 border-b border-zinc-800 px-4 py-3 last:border-0 {notif.read ? 'opacity-50' : ''}"
        >
          <Icon class="mt-0.5 h-3.5 w-3.5 shrink-0 {colorMap[notif.type]}" />
          <div class="flex flex-1 flex-col gap-0.5">
            <p class="text-sm text-white">{notif.message}</p>
            <p class="font-['Space_Grotesk',sans-serif] text-xs text-zinc-500">{formatTime(notif.timestamp)}</p>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
