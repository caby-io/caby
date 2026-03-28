<script lang="ts">
	import type { Entry } from '../entry';
	import OverviewEntrySelect from './OverviewEntrySelect.svelte';

	let {
		selected = $bindable(undefined),
		overview_entries,
		space
	}: { selected?: Entry; overview_entries: any; space: string } = $props();

	let overview_dirs = $derived(overview_entries?.filter((e: any) => e.entry_type === 'directory'));

	let expand = $state(false);
</script>

<div class="entry-overview">
	{#each overview_dirs as _, i}
		<OverviewEntrySelect bind:entry={overview_dirs[i]} {space} />
	{/each}
</div>

<style lang="scss">
</style>
