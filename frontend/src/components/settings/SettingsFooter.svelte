<script lang="ts">
  interface Props {
    onSave?: () => void;
    onReset?: () => void;
  }

  let { onSave, onReset }: Props = $props();
</script>

<footer class="flex items-center gap-3 border-t border-zinc-800 px-6 py-4">
  <button
    class="rounded-[2px] border border-zinc-800 px-4 py-2 text-sm text-white transition-colors hover:bg-zinc-800"
    onclick={onReset}
  >
    Reset to Defaults
  </button>
  <button
    class="rounded-[2px] bg-white px-4 py-2 text-sm font-semibold text-black transition-opacity hover:opacity-90"
    onclick={onSave}
  >
    Save Changes
  </button>
</footer>
