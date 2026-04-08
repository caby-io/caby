<script lang="ts">
	import { goto } from '$app/navigation';
	import { getSpaces } from '$lib/api/api_spaces';
	import { client } from '$lib/stores/client.svelte';
	import { onMount } from 'svelte';

	const onLoad = async () => {
		let resp = await getSpaces(client);

		if (resp.status != 'success') {
			console.error('could not fetch spaces');
			// todo
			return;
		}

		goto(`/files/${resp.data![0].name}`);
	};

	onMount(() => {
		onLoad();
	});
</script>
