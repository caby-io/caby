<script lang="ts">
	import { getSpaces } from '$lib/api/api_spaces';
	import type { Space } from '$lib/space';
	import { client } from '$lib/stores/client.svelte';

	let spaces: Space[] = $state([]);

	const updateSpaces = async () => {
		let resp = await getSpaces(client);

		if (resp.status != 'success') {
			console.error('could not fetch spaces');
			// todo
			return;
		}

		spaces = resp.data!;
		console.log(spaces);
	};

	$effect(() => {
		updateSpaces();
	});
</script>

<button class="fx fx--cc button spaces-button">{spaces[0]?.name}</button>

<style lang="scss">
	.spaces-button {
		margin: 1rem;
	}
</style>
