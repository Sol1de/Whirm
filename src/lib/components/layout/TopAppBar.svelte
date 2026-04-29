<script lang="ts">
  interface Props {
    title: string;
    searchPlaceholder?: string;
    onSearch?: (query: string) => void;
  }

  let { title, searchPlaceholder = 'Search...', onSearch }: Props = $props();

  let searchQuery = $state('');

  function handleSearch() {
    onSearch?.(searchQuery);
  }
</script>

<header class="flex h-14 shrink-0 items-center gap-4 border-b border-border bg-background px-6">
  <h1 class="text-lg font-semibold">{title}</h1>

  <div class="ml-auto flex items-center gap-2">
    <!-- Search input -->
    <div class="relative">
      <!-- Icon slot (search icon) -->
      <input
        type="text"
        placeholder={searchPlaceholder}
        bind:value={searchQuery}
        oninput={handleSearch}
        class="h-8 w-64 rounded-md border border-input bg-background px-3 py-1 pl-9 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      />
    </div>

    <!-- Action buttons slot -->
  </div>
</header>
