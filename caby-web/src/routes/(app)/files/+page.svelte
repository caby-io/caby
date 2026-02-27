<script lang="ts">
	import { getSpaces } from '$lib/api/api_spaces';
	import { client } from '$lib/stores/client.svelte';
	import { redirect } from '@sveltejs/kit';

	const onLoad = async () => {
		let resp = await getSpaces(client);

		if (resp.status != 'success') {
			console.error('could not fetch spaces');
			// todo
			return;
		}

		window.location.href = `/files/${resp.data![0].name}`;
	};

	$effect(() => {
		onLoad();
	});
</script>
