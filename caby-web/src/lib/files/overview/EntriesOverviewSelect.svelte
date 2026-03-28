<script lang="ts">
	import type { OverviewEntry } from './overview_entry';
	import OverviewEntrySelect from './OverviewEntrySelect.svelte';

	let {
		overview_entries,
		space,
		onSelect
	}: { overview_entries: any; space: string; onSelect?: (entry: OverviewEntry) => void } = $props();

	let overview_dirs = $derived(overview_entries?.filter((e: any) => e.entry_type === 'directory'));

	let root_entry: OverviewEntry = $state({
		entry_type: 'directory',
		icon: '🗄️',
		name: space,
		path: '',
		children: [],
		is_expanded: true,
		is_selected: false
	});

	$effect(() => {
		root_entry.children = overview_dirs ?? [];
	});
</script>

<div class="entry-overview">
	<OverviewEntrySelect entry={root_entry} {space} {onSelect} />
</div>

<style lang="scss">
</style>
