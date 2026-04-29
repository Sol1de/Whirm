<script lang="ts">
  interface Props {
    title: string;
    value?: string;
    subtitle?: string;
  }

  let { title, value, subtitle }: Props = $props();
</script>

<div class="rounded-lg border border-border bg-card p-4">
  <p class="text-sm text-muted-foreground">{title}</p>
  <!-- value + subtitle -->
</div>
