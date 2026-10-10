<script lang="ts">
	import { EntryType } from '../entry';
	import type { OverviewEntry } from './overview_entry';
	import OverviewEntryNav from './OverviewEntryNav.svelte';

	let {
		overview_entries,
		href_base,
		current_path
	}: { overview_entries: OverviewEntry[]; href_base: string; current_path: string } = $props();

	let overview_dirs = $derived(
		overview_entries.filter((e) => e.entry_type === EntryType.DIRECTORY)
	);
</script>

<div class="entry-overview">
	{#each overview_dirs as entry (entry.path)}
		<OverviewEntryNav {entry} {href_base} {current_path} />
	{/each}
</div>
