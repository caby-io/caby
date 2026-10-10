<script lang="ts">
	import { page } from '$app/state';
	import { spaces } from '$lib/stores/spaces.svelte';
	import SidebarSpaceSelector from './SidebarSpaceSelector.svelte';

	const current_space = $derived(spaces.list.find((s) => s.name === page.params.space));
</script>

<section class="sidebar-spaces fx fx--col">
	<SidebarSpaceSelector {current_space} spaces={spaces.list} />
</section>

<style lang="scss">
	.sidebar-spaces {
		margin: 1rem 0.75rem 0.5rem;
	}
</style>
