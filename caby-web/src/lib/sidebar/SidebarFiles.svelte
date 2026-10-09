<script lang="ts">
	import { page } from '$app/state';
	import EntriesOverviewNav from '$lib/files/overview/EntriesOverviewNav.svelte';
	import { expandToPath, overview } from '$lib/stores/overview.svelte';
	import SidebarSection from './SidebarSection.svelte';

	const space = $derived(page.params.space!);
	const href_base = $derived(`/files/${space}`);
	const current_path = $derived(page.params.path ?? '');

	$effect(() => {
		expandToPath(href_base, current_path);
	});
</script>

<SidebarSection title="Files">
	{#if overview.space === space}
		<EntriesOverviewNav overview_entries={overview.entries} {href_base} {current_path} />
	{/if}
</SidebarSection>
