<script lang="ts">
  import type { Component } from 'svelte';

  interface Props {
    label: string;
    value: string;
    subtitle?: string;
    icon: Component;
  }

  let { label, value, subtitle, icon: Icon }: Props = $props();
</script>

<div class="relative rounded-[2px] border border-zinc-800 bg-zinc-900 p-6">
  <!-- Icon top-right -->
  <div class="absolute right-4 top-4 text-zinc-600">
    <Icon class="h-5 w-5" />
  </div>

  <!-- Label -->
  <p
    class="mb-3 font-['Space_Grotesk',sans-serif] text-xs uppercase tracking-[0.6px] text-zinc-500"
  >
    {label}
  </p>

  <!-- Value -->
  <p class="font-['Space_Grotesk',sans-serif] text-2xl text-white">
    {value || '—'}
  </p>

  <!-- Subtitle -->
  {#if subtitle}
    <p class="mt-1 text-sm text-zinc-500">{subtitle}</p>
  {/if}
</div>
