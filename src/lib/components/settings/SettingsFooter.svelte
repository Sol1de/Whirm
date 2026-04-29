<script lang="ts">
  import { Button } from '$lib/components/ui/button';

  interface Props {
    onSave?: () => void;
    onReset?: () => void;
  }

  let { onSave, onReset }: Props = $props();
</script>

<footer class="flex items-center gap-3 border-t border-border bg-background px-6 py-4">
  <Button onclick={onSave}>Save Changes</Button>
  <Button variant="outline" onclick={onReset}>Reset to Defaults</Button>
</footer>
